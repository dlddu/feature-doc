//! Stage 2 of the analysis pipeline.
//!
//! The input is the file-path list stage 1 measured, plus the heads of a few
//! entry-point and manifest files ([`key_files`]). Paths alone cannot show what
//! decides an architecture answer — a binary that embeds its frontend, handlers
//! that call a client library directly — but reading the whole tree would put it
//! all through the LLM boundary. So only an allowlist of well-known file names is
//! read, and each only up to [`MAX_EXCERPT_BYTES`]. Evidence stays paths.

use serde_json::{json, Value};

use crate::llm::{self, Ask};
use crate::repo_scan::FileExcerpt;

/// In PRD order — the screen renders the axes in this order, not the document's.
pub const AXES: [(&str, &str); 5] = [
    ("infrastructure", "인프라 (배포 매니페스트 · IaC)"),
    ("repository_structure", "저장소 구조 (monorepo 여부 · 모듈 구분)"),
    ("architecture", "아키텍처 (레이어 구분 · DDD/MVC/헥사고날 등 패턴)"),
    ("framework", "프레임워크 · 런타임"),
    ("middleware", "미들웨어 (인증 · 로깅 · 캐시 등)"),
];

/// What each axis asks for, as the model reads it — the labels in [`AXES`] are screen copy.
/// Examples name kinds of evidence, never concrete paths: a concrete path in the
/// prompt is one the model can cite even when the tree does not contain it.
const AXIS_GUIDE: [(&str, &str); 5] = [
    (
        "infrastructure",
        "deployment manifests and infrastructure-as-code \
         (e.g. Kubernetes / Helm / Kustomize manifests, Dockerfiles, compose files, \
         Terraform or other IaC, CI/CD deploy workflows)",
    ),
    (
        "repository_structure",
        "whether the repository is a monorepo and how it is divided into modules, \
         packages or services (e.g. workspace manifests, per-module build files, \
         top-level directory split)",
    ),
    (
        "architecture",
        "layering and architectural patterns such as DDD, MVC or hexagonal \
         (e.g. controller / service / repository directories, domain / ports / adapters, \
         separate API and worker processes)",
    ),
    (
        "framework",
        "frameworks and runtimes (e.g. web or UI framework, language runtime and \
         toolchain, as shown by build manifests, lockfiles and entry points)",
    ),
    (
        "middleware",
        "concerns applied across requests such as authentication, sessions, \
         logging / tracing, caching and rate limiting",
    ),
];

/// How many files [`key_files`] picks, and how much of each is read. Together they
/// bound what the excerpts add to the prompt (≈ 32 KB).
pub const MAX_KEY_FILES: usize = 8;
pub const MAX_EXCERPT_BYTES: usize = 4000;

/// File names read for context, most telling first. Names only — never a pattern
/// that could match a credentials file.
const KEY_FILE_NAMES: [&str; 32] = [
    // entry points
    "main.go", "main.rs", "lib.rs", "main.py", "app.py", "manage.py", "main.ts",
    "main.tsx", "main.js", "index.ts", "server.ts", "server.js", "app.ts", "app.js",
    "Program.cs", "main.kt",
    // build and workspace manifests
    "go.mod", "Cargo.toml", "package.json", "pyproject.toml", "requirements.txt",
    "pom.xml", "build.gradle", "build.gradle.kts", "Gemfile", "composer.json",
    "pnpm-workspace.yaml", "go.work",
    // how it is shipped
    "Dockerfile", "docker-compose.yml", "docker-compose.yaml", "kustomization.yaml",
];

/// Directories whose files describe someone else's code or a test fixture, not
/// this repository's structure.
const SKIPPED_DIRS: [&str; 10] = [
    "vendor", "node_modules", "third_party", "testdata", "fixtures", "test", "tests",
    "e2e", "examples", "docs",
];

/// Deeper than this and a manifest is usually a sub-package's, not the project's.
const MAX_KEY_FILE_DEPTH: usize = 4;

/// How many paths are handed to the model. A cap keeps the prompt bounded on large
/// repositories; taking the *first* N of a sorted list rather than a sample keeps
/// it deterministic.
const MAX_PATHS: usize = 400;

const SYSTEM: &str = "\
You analyze a source repository's file tree and extract its cross-cutting concerns.
Work only from the paths you are given: never invent a file that is not in the list.
Some files also come with the head of their content. Use it to judge how the code
is structured and wired together, but cite only paths from the list.
Every item must cite at least one path from the list as its evidence.
If an axis has no supporting evidence in the tree, return an empty item list for it
rather than guessing.";

/// Nothing but JSON Schema goes in here — it is sent to the provider verbatim.
fn schema() -> Value {
    let axis_keys: Vec<&str> = AXES.iter().map(|(key, _)| *key).collect();
    json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["categories"],
        "properties": {
            "categories": {
                "type": "array",
                "items": {
                    "type": "object",
                    "additionalProperties": false,
                    "required": ["axis", "items"],
                    "properties": {
                        "axis": { "type": "string", "enum": axis_keys },
                        "items": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "additionalProperties": false,
                                "required": ["name", "evidence"],
                                "properties": {
                                    "name": { "type": "string" },
                                    "evidence": {
                                        "type": "array",
                                        "items": { "type": "string" },
                                    },
                                },
                            },
                        },
                    },
                },
            },
        },
    })
}

/// Derived from the actual path list rather than hard-coded, so a test can assert
/// the real evidence property instead of asserting a fixed string.
fn stub_answer(paths: &[String]) -> Value {
    let categories: Vec<Value> = AXES
        .iter()
        .enumerate()
        .map(|(i, (key, label))| {
            // Chosen by position so different axes cite different files.
            let items: Vec<Value> = paths
                .get(i % paths.len().max(1))
                .map(|path| {
                    vec![json!({
                        "name": format!("{label} (stub 추출)"),
                        "evidence": [path],
                    })]
                })
                .unwrap_or_default();
            json!({ "axis": key, "items": items })
        })
        .collect();
    json!({ "categories": categories })
}

fn prompt(
    owner: &str,
    name: &str,
    branch: &str,
    paths: &[String],
    excerpts: &[FileExcerpt],
) -> String {
    let listed = paths.join("\n");
    let mut text = format!(
        "Repository: {owner}/{name}@{branch}\n\
         Files ({shown} of {total}):\n{listed}\n\n\
         Extract the cross-cutting concerns for each of these axes:\n{axes}\n\n\
         The examples describe kinds of evidence, not files in this repository: \
         cite only paths from the list above.",
        shown = paths.len(),
        total = paths.len(),
        axes = AXIS_GUIDE
            .iter()
            .map(|(key, guide)| format!("- {key}: {guide}"))
            .collect::<Vec<_>>()
            .join("\n"),
    );
    if !excerpts.is_empty() {
        text.push_str("\n\nHeads of key files (entry points and manifests):");
        for e in excerpts {
            let cut = if e.truncated { " (truncated)" } else { "" };
            text.push_str(&format!(
                "\n\n----- {}{cut} -----\n{}",
                e.path, e.body
            ));
        }
    }
    text
}

/// Which files' heads [`extract`] should be given, from the same list the prompt
/// shows — so every excerpt belongs to a path the model may cite.
pub fn key_files(paths: &[String]) -> Vec<String> {
    let mut picked: Vec<(usize, usize, String)> = input_paths(paths)
        .into_iter()
        .filter_map(|p| {
            let parts: Vec<&str> = p.split('/').collect();
            let (file, dirs) = parts.split_last()?;
            if parts.len() > MAX_KEY_FILE_DEPTH || dirs.iter().any(|d| SKIPPED_DIRS.contains(d)) {
                return None;
            }
            let rank = KEY_FILE_NAMES.iter().position(|n| n == file)?;
            let depth = parts.len();
            Some((rank, depth, p))
        })
        .collect();
    // Most telling name first, then the shallowest copy of it, then by path — a
    // total order, so the same tree always yields the same excerpts.
    picked.sort();
    // One copy of each name before any second copy: eight `main.go`s under `cmd/`
    // would otherwise crowd out the `go.mod` that says how they fit together.
    let (first, rest): (Vec<_>, Vec<_>) = picked
        .iter()
        .enumerate()
        .partition(|(i, (rank, _, _))| *i == 0 || picked[i - 1].0 != *rank);
    first
        .into_iter()
        .chain(rest)
        .take(MAX_KEY_FILES)
        .map(|(_, (_, _, p))| p.clone())
        .collect()
}

pub fn input_paths(paths: &[String]) -> Vec<String> {
    let mut sorted = paths.to_vec();
    sorted.sort();
    sorted.dedup();
    sorted.truncate(MAX_PATHS);
    sorted
}

pub async fn extract(
    http: &reqwest::Client,
    mode: crate::config::Mode,
    provider: llm::Provider,
    key: Option<&str>,
    language: Option<llm::Language>,
    owner: &str,
    name: &str,
    branch: &str,
    paths: &[String],
    excerpts: &[FileExcerpt],
) -> Result<llm::Answer, String> {
    let paths = input_paths(paths);
    if paths.is_empty() {
        return Err("repository tree is empty; nothing to analyze".to_string());
    }
    llm::ask(
        http,
        mode,
        provider,
        key,
        Ask {
            system: SYSTEM,
            language,
            user: prompt(owner, name, branch, &paths, excerpts),
            schema: schema(),
            // mock-exception: LLM-01 — 실 LLM 산출물에 대한 결정적 단정을 위해 고정 답을 공급
            stub: stub_answer(&paths),
        },
    )
    .await
}

/// Shaped to match stage 1's own detail line, which the same row renders.
pub fn detail(doc: &Value) -> String {
    let items: usize = doc
        .get("categories")
        .and_then(Value::as_array)
        .map(|cats| {
            cats.iter()
                .filter_map(|c| c.get("items").and_then(Value::as_array))
                .map(Vec::len)
                .sum()
        })
        .unwrap_or(0);
    format!("{} categories · {} items", AXES.len(), items)
}

// mock-exception: LLM-01 — 단위 테스트 모듈: 결정적 더블로 검증(E2E 예외가 아닌 지문 등재 — 정책: docs/e2e-mocking-policy.md)
#[cfg(test)]
mod tests {
    #[test]
    fn schema_is_accepted_by_openai_strict_mode() {
        crate::llm::assert_strict_schema(&schema());
    }

    use super::*;

    fn tree() -> Vec<String> {
        vec![
            "src/main.rs".to_string(),
            "deploy/base/deployment.yaml".to_string(),
            "Cargo.toml".to_string(),
            "src/middleware/auth.rs".to_string(),
            "src/main.rs".to_string(), // duplicate — must be collapsed
        ]
    }

    #[test]
    fn the_prompt_describes_every_axis_in_screen_order() {
        let text = prompt("acme", "widgets", "main", &input_paths(&tree()), &[]);
        let keys: Vec<&str> = AXES.iter().map(|(key, _)| *key).collect();
        let guided: Vec<&str> = AXIS_GUIDE.iter().map(|(key, _)| *key).collect();
        assert_eq!(guided, keys, "every schema axis needs a guide line, in order");
        for (key, guide) in AXIS_GUIDE {
            assert!(text.contains(&format!("- {key}: {guide}")), "{key} not described");
        }
    }

    fn owned(paths: &[&str]) -> Vec<String> {
        paths.iter().map(|p| p.to_string()).collect()
    }

    #[test]
    fn key_files_picks_entry_points_and_manifests_but_not_tests_or_vendored_code() {
        let tree = owned(&[
            "README.md",
            "go.mod",
            "main.go",
            "handlers/pods.go",
            "frontend/package.json",
            "frontend/src/main.tsx",
            "vendor/github.com/x/y/main.go",
            "e2e/k8s/kustomization.yaml",
            "test/fixtures/Dockerfile",
            "a/b/c/d/main.go",
            ".env",
        ]);
        assert_eq!(
            key_files(&tree),
            owned(&["main.go", "frontend/src/main.tsx", "go.mod", "frontend/package.json"])
        );
    }

    #[test]
    fn key_files_takes_one_of_each_name_before_a_second_copy() {
        let mut tree: Vec<String> = (0..10).map(|i| format!("cmd/tool{i}/main.go")).collect();
        tree.push("go.mod".to_string());
        let picked = key_files(&tree);
        assert_eq!(picked.len(), MAX_KEY_FILES);
        assert!(picked.contains(&"go.mod".to_string()), "{picked:?}");
        assert_eq!(key_files(&tree), picked, "the choice must be deterministic");
    }

    #[test]
    fn the_prompt_carries_each_excerpt_under_its_path_and_marks_a_cut() {
        let excerpts = vec![
            FileExcerpt {
                path: "main.go".to_string(),
                body: "//go:embed frontend/dist".to_string(),
                truncated: true,
            },
            FileExcerpt {
                path: "go.mod".to_string(),
                body: "module x".to_string(),
                truncated: false,
            },
        ];
        let text = prompt("acme", "widgets", "main", &input_paths(&tree()), &excerpts);
        assert!(text.contains("----- main.go (truncated) -----\n//go:embed frontend/dist"));
        assert!(text.contains("----- go.mod -----\nmodule x"));
        let bare = prompt("acme", "widgets", "main", &input_paths(&tree()), &[]);
        assert!(!bare.contains("Heads of key files"), "no section without excerpts");
    }

    #[test]
    fn input_paths_is_sorted_deduped_and_stable() {
        let a = input_paths(&tree());
        let b = input_paths(&tree());
        assert_eq!(a, b);
        assert_eq!(a.len(), 4, "the duplicate path should collapse");
        let mut sorted = a.clone();
        sorted.sort();
        assert_eq!(a, sorted);
    }

    #[tokio::test]
    async fn every_extracted_item_cites_a_path_from_the_tree() {
        let http = reqwest::Client::new();
        let paths = tree();
        let answer = extract(
            &http,
            crate::config::Mode::Stub,
            llm::Provider::Anthropic,
            None,
            None,
            "acme",
            "widgets",
            "main",
            &paths,
            &[],
        )
        .await
        .unwrap();

        let known = input_paths(&paths);
        let cats = answer.content["categories"].as_array().unwrap();
        assert_eq!(cats.len(), AXES.len(), "one entry per AC1.2 axis");
        let mut items_seen = 0;
        for cat in cats {
            for item in cat["items"].as_array().unwrap() {
                items_seen += 1;
                let evidence = item["evidence"].as_array().unwrap();
                assert!(!evidence.is_empty(), "item has no evidence");
                for e in evidence {
                    let path = e.as_str().unwrap().to_string();
                    assert!(known.contains(&path), "cited a path not in the tree: {path}");
                }
            }
        }
        assert!(items_seen > 0);
    }

    #[tokio::test]
    async fn the_same_tree_reproduces_the_same_document() {
        let http = reqwest::Client::new();
        let paths = tree();
        let run = || {
            extract(
                &http,
                crate::config::Mode::Stub,
                llm::Provider::Anthropic,
                None,
                None,
                "acme",
                "widgets",
                "main",
                &paths,
                &[],
            )
        };
        assert_eq!(run().await.unwrap().content, run().await.unwrap().content);
    }

    #[tokio::test]
    async fn an_empty_tree_fails_rather_than_inventing_a_document() {
        let http = reqwest::Client::new();
        let err = extract(
            &http,
            crate::config::Mode::Stub,
            llm::Provider::Anthropic,
            None,
            None,
            "acme",
            "widgets",
            "main",
            &[],
            &[],
        )
        .await
        .unwrap_err();
        assert!(err.contains("empty"), "{err}");
    }

    /// The schema travels to the provider verbatim, so the stub answer must not
    /// be inside it — OpenAI rejects unknown keywords under `strict`.
    #[test]
    fn the_schema_is_only_a_schema() {
        let s = schema();
        assert!(s.get("stub_answer").is_none());
        assert_eq!(s["additionalProperties"], false);
        assert_eq!(s["required"][0], "categories");
    }

    #[test]
    fn detail_reads_like_an_s04_step() {
        let doc = json!({ "categories": [
            { "axis": "infrastructure", "items": [{ "name": "x", "evidence": ["a"] }] },
            { "axis": "framework", "items": [] },
        ]});
        assert_eq!(detail(&doc), "5 categories · 1 items");
    }
}
