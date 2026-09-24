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

/// A concrete path here is one the model can cite even when the tree does not contain it.
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
    "pom.xml", "build.gradle", "build.gradle.kts", "pnpm-workspace.yaml", "go.work",
    // how the services run together, ahead of the rarer manifests below
    "docker-compose.yml", "docker-compose.yaml",
    "Gemfile", "composer.json",
    // how it is shipped
    "Dockerfile", "kustomization.yaml",
];

/// Directories whose files describe someone else's code or a test fixture, not
/// this repository's structure.
const SKIPPED_DIRS: [&str; 10] = [
    "vendor", "node_modules", "third_party", "testdata", "fixtures", "test", "tests",
    "e2e", "examples", "docs",
];

/// Code that is not the product's structure — helper scripts and dev tooling —
/// kept out of the excerpts, though still in the path list.
const NOT_STRUCTURE_DIRS: [&str; 3] = ["scripts", "tools", "hack"];

/// Directory names that mark the main one of several same-named entry points.
const ENTRY_DIR_HINTS: [&str; 4] = ["server", "api", "app", "web"];

/// Deeper than this and a manifest is usually a sub-package's, not the project's.
const MAX_KEY_FILE_DEPTH: usize = 4;

/// How many paths are handed to the model. A cap keeps the prompt bounded on large
/// repositories; [`input_paths`] decides which ones make it.
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
    total: usize,
    excerpts: &[FileExcerpt],
) -> String {
    let listed = paths.join("\n");
    let sampled = if total > paths.len() {
        ", sampled across directories; tests, docs and vendored files go first"
    } else {
        ""
    };
    let mut text = format!(
        "Repository: {owner}/{name}@{branch}\n\
         Files ({shown} of {total}{sampled}):\n{listed}\n\n\
         Extract the cross-cutting concerns for each of these axes:\n{axes}\n\n\
         The examples describe kinds of evidence, not files in this repository: \
         cite only paths from the list above.",
        shown = paths.len(),
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
    let mut picked: Vec<((usize, bool, usize, String), String)> = input_paths(paths)
        .into_iter()
        .filter_map(|p| {
            let parts: Vec<&str> = p.split('/').collect();
            let (file, dirs) = parts.split_last()?;
            if parts.len() > MAX_KEY_FILE_DEPTH
                || dirs
                    .iter()
                    .any(|d| SKIPPED_DIRS.contains(d) || NOT_STRUCTURE_DIRS.contains(d))
            {
                return None;
            }
            let rank = KEY_FILE_NAMES.iter().position(|n| n == file)?;
            let parent = dirs.last().copied().unwrap_or("");
            // An `index.ts` deep in a tree re-exports a folder; only a package's
            // own `index` (at its root or directly under `src`) is an entry point.
            if file.starts_with("index.") && dirs.len() > 1 && parent != "src" {
                return None;
            }
            // Among several `cmd/*/main.go`, the one named for a server or for its
            // own service is the one that shows how the thing is wired. A file at
            // the root of the repository or of a top-level directory is that
            // directory's own, and counts the same.
            let plain = !(dirs.len() <= 1
                || ENTRY_DIR_HINTS.contains(&parent)
                || Some(&parent) == dirs.first());
            let top = if dirs.is_empty() { String::new() } else { dirs[0].to_string() };
            Some(((rank, plain, parts.len(), top), p))
        })
        .collect();
    // Most telling name first, then an entry-looking directory, then the shallowest,
    // then by path — a total order, so the same tree always yields the same excerpts.
    picked.sort_by(|a, b| (&a.0 .0, a.0 .1, a.0 .2, &a.1).cmp(&(&b.0 .0, b.0 .1, b.0 .2, &b.1)));
    // One copy of each name per top-level directory before any further copy: eight
    // `main.go`s under `cmd/` would otherwise crowd out the `go.mod` that says how
    // they fit together, while a backend and a worker each keep their own.
    let mut seen = std::collections::BTreeSet::new();
    let (first, rest): (Vec<_>, Vec<_>) = picked
        .into_iter()
        .partition(|((rank, _, _, top), _)| seen.insert((*rank, top.clone())));
    first
        .into_iter()
        .chain(rest)
        .take(MAX_KEY_FILES)
        .map(|(_, p)| p)
        .collect()
}

/// The paths every LLM stage sees, capped at [`MAX_PATHS`] and returned sorted.
///
/// Over the cap, the first N of a sorted list is the wrong cut: it drops whole
/// top-level directories that happen to sort last, and those are often a
/// project's own services (`worker/`, `k8s/`) rather than its tests. So the
/// sample goes round-robin across directories — each directory's entry points
/// and manifests first — and takes the project's code before what supports it
/// (tests, fixtures, docs, vendored code). Every choice is a total order on the
/// paths, so the same tree always yields the same list.
pub fn input_paths(paths: &[String]) -> Vec<String> {
    let mut all = paths.to_vec();
    all.sort();
    all.dedup();
    if all.len() <= MAX_PATHS {
        return all;
    }
    let (code, supporting): (Vec<String>, Vec<String>) =
        all.into_iter().partition(|p| !is_supporting(p));
    let mut picked = Vec::with_capacity(MAX_PATHS);
    for tier in [code, supporting] {
        take_round_robin(tier, MAX_PATHS - picked.len(), &mut picked);
    }
    picked.sort();
    picked
}

/// Whether a path supports the code rather than being it.
fn is_supporting(path: &str) -> bool {
    let (dirs, file) = path.rsplit_once('/').unwrap_or(("", path));
    dirs.split('/')
        .any(|d| SKIPPED_DIRS.contains(&d) || d.starts_with("__"))
        || file.contains("_test.")
        || file.contains(".test.")
        || file.contains(".spec.")
}

/// Directories are told apart by their first two levels: one level lumps
/// `backend/internal/*` into a single queue, and more splits a monorepo into so
/// many that each gets a file or two.
fn directory_of(path: &str) -> &str {
    let dir = path.rsplit_once('/').map_or("", |(d, _)| d);
    match dir.match_indices('/').nth(1) {
        Some((i, _)) => &dir[..i],
        None => dir,
    }
}

fn take_round_robin(tier: Vec<String>, budget: usize, out: &mut Vec<String>) {
    let mut queues: std::collections::BTreeMap<String, std::collections::VecDeque<String>> =
        Default::default();
    for p in tier {
        queues.entry(directory_of(&p).to_string()).or_default().push_back(p);
    }
    for queue in queues.values_mut() {
        queue
            .make_contiguous()
            .sort_by_key(|p| (!KEY_FILE_NAMES.contains(&p.rsplit('/').next().unwrap_or(p)), p.clone()));
    }
    let mut taken = 0;
    while taken < budget && queues.values().any(|q| !q.is_empty()) {
        for queue in queues.values_mut() {
            if taken == budget {
                break;
            }
            if let Some(p) = queue.pop_front() {
                out.push(p);
                taken += 1;
            }
        }
    }
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
    let total = {
        let mut all = paths.to_vec();
        all.sort();
        all.dedup();
        all.len()
    };
    let paths = input_paths(paths);
    if paths.is_empty() {
        return Err("repository tree is empty; nothing to analyze".to_string());
    }
    let mut answer = llm::ask(
        http,
        mode,
        provider,
        key,
        Ask {
            system: SYSTEM,
            language,
            user: prompt(owner, name, branch, &paths, total, excerpts),
            schema: schema(),
            // mock-exception: LLM-01 — 실 LLM 산출물에 대한 결정적 단정을 위해 고정 답을 공급
            stub: stub_answer(&paths),
        },
    )
    .await?;
    let dropped = keep_listed_evidence(&mut answer.content, &paths);
    if dropped.paths > 0 {
        tracing::warn!(
            paths = dropped.paths,
            items = dropped.items,
            "cross-cutting evidence outside the listed tree dropped"
        );
    }
    Ok(answer)
}

/// What [`keep_listed_evidence`] removed.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Dropped {
    pub paths: usize,
    pub items: usize,
}

/// Holds the model to the one rule the prompt can only ask for: evidence comes
/// from the list it was shown. File contents in the prompt make it easy to
/// cite a file a handler name suggests but the tree does not have, so this is
/// code rather than an instruction the model may bend — the same stance
/// `acceptance::merge` takes on contradictions.
///
/// A cited directory is kept when it is the parent of a listed path: models name
/// `app/src` for "the source tree", and the claim is checkable. An item left with
/// no evidence goes, because an item without a path is exactly the guess the
/// stage promises not to make.
pub fn keep_listed_evidence(doc: &mut Value, paths: &[String]) -> Dropped {
    let listed = |cited: &str| {
        let cited = cited.trim_end_matches('/');
        paths.iter().any(|p| {
            p == cited || (p.starts_with(cited) && p.as_bytes().get(cited.len()) == Some(&b'/'))
        })
    };
    let mut dropped = Dropped::default();
    let Some(categories) = doc.get_mut("categories").and_then(Value::as_array_mut) else {
        return dropped;
    };
    for items in categories
        .iter_mut()
        .filter_map(|c| c.get_mut("items").and_then(Value::as_array_mut))
    {
        for item in items.iter_mut() {
            if let Some(evidence) = item.get_mut("evidence").and_then(Value::as_array_mut) {
                let before = evidence.len();
                evidence.retain(|e| e.as_str().is_some_and(|p| listed(p)));
                dropped.paths += before - evidence.len();
            }
        }
        let before = items.len();
        items.retain(|item| {
            item.get("evidence")
                .and_then(Value::as_array)
                .is_some_and(|e| !e.is_empty())
        });
        dropped.items += before - items.len();
    }
    dropped
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

    fn answer_citing(evidence: &[&[&str]]) -> Value {
        let items: Vec<Value> = evidence
            .iter()
            .enumerate()
            .map(|(i, paths)| json!({ "name": format!("item {i}"), "evidence": paths }))
            .collect();
        json!({ "categories": [{ "axis": "architecture", "items": items }] })
    }

    fn cited(doc: &Value) -> Vec<Vec<String>> {
        doc["categories"][0]["items"]
            .as_array()
            .unwrap()
            .iter()
            .map(|i| {
                i["evidence"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|p| p.as_str().unwrap().to_string())
                    .collect()
            })
            .collect()
    }

    #[test]
    fn evidence_outside_the_listed_tree_is_dropped_and_so_is_an_item_left_with_none() {
        let listed = input_paths(&tree());
        let mut doc = answer_citing(&[
            &["src/main.rs", "src/handlers/pod_logs.rs"],
            &["src/invented.rs"],
            &["Cargo.toml"],
        ]);
        let dropped = keep_listed_evidence(&mut doc, &listed);
        assert_eq!(dropped, Dropped { paths: 2, items: 1 });
        assert_eq!(
            cited(&doc),
            vec![vec!["src/main.rs".to_string()], vec!["Cargo.toml".to_string()]]
        );
        assert_eq!(detail(&doc), "5 categories · 2 items");
    }

    #[test]
    fn a_cited_directory_counts_only_when_it_holds_a_listed_path() {
        let listed = input_paths(&tree());
        let mut doc = answer_citing(&[&["src/middleware", "deploy/base/"], &["src/mid"], &["srcx"]]);
        let dropped = keep_listed_evidence(&mut doc, &listed);
        assert_eq!(dropped, Dropped { paths: 2, items: 2 });
        assert_eq!(
            cited(&doc),
            vec![vec!["src/middleware".to_string(), "deploy/base/".to_string()]]
        );
    }

    #[test]
    fn the_stub_answer_survives_the_evidence_check_untouched() {
        let listed = input_paths(&tree());
        let mut doc = stub_answer(&listed);
        let before = doc.clone();
        assert_eq!(keep_listed_evidence(&mut doc, &listed), Dropped::default());
        assert_eq!(doc, before);
    }

    #[test]
    fn the_prompt_describes_every_axis_in_screen_order() {
        let text = prompt("acme", "widgets", "main", &input_paths(&tree()), 4, &[]);
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
    fn key_files_prefers_each_service_s_own_entry_point_over_tools_and_barrels() {
        let tree = owned(&[
            "app/package.json",
            "app/src/components/index.ts",
            "backend/cmd/reset-user/main.go",
            "backend/cmd/seed-diary/main.go",
            "backend/cmd/server/main.go",
            "backend/go.mod",
            "scripts/openrouter-mock/server.js",
            "worker/cmd/worker/main.go",
            "worker/go.mod",
        ]);
        assert_eq!(
            key_files(&tree),
            owned(&[
                "backend/cmd/server/main.go",
                "worker/cmd/worker/main.go",
                "backend/go.mod",
                "worker/go.mod",
                "app/package.json",
                "backend/cmd/reset-user/main.go",
                "backend/cmd/seed-diary/main.go",
            ])
        );
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
        let text = prompt("acme", "widgets", "main", &input_paths(&tree()), 4, &excerpts);
        assert!(text.contains("----- main.go (truncated) -----\n//go:embed frontend/dist"));
        assert!(text.contains("----- go.mod -----\nmodule x"));
        let bare = prompt("acme", "widgets", "main", &input_paths(&tree()), 4, &[]);
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

    /// Shaped like the tree that exposed the old cut: a front end and a back end
    /// that sort first and fill the cap, a service and its manifests that sort
    /// last, and tests spread through all of it.
    fn oversized_tree() -> Vec<String> {
        let mut tree = Vec::new();
        for i in 0..250 {
            tree.push(format!("app/src/screens/screen_{i:03}.tsx"));
            tree.push(format!("app/src/screens/__tests__/screen_{i:03}.test.tsx"));
        }
        for i in 0..150 {
            tree.push(format!("backend/internal/records/file_{i:03}.go"));
            tree.push(format!("backend/internal/records/file_{i:03}_test.go"));
        }
        tree.extend(owned(&[
            "backend/go.mod",
            "worker/go.mod",
            "worker/cmd/worker/main.go",
            "worker/internal/tasks/aipreview/task.go",
            "k8s/base/worker-deployment.yaml",
            "k8s/base/kustomization.yaml",
        ]));
        tree
    }

    #[test]
    fn over_the_cap_every_directory_is_sampled_and_code_comes_before_tests() {
        let tree = oversized_tree();
        let picked = input_paths(&tree);
        assert_eq!(picked.len(), MAX_PATHS);
        for late in [
            "worker/go.mod",
            "worker/cmd/worker/main.go",
            "worker/internal/tasks/aipreview/task.go",
            "k8s/base/worker-deployment.yaml",
            "k8s/base/kustomization.yaml",
        ] {
            assert!(picked.contains(&late.to_string()), "{late} sorts last but must be seen");
        }
        assert!(
            !picked.iter().any(|p| is_supporting(p)),
            "406 non-test paths exist, so no test should take a slot"
        );
        let mut sorted = picked.clone();
        sorted.sort();
        assert_eq!(picked, sorted, "the prompt lists paths in order");
        assert_eq!(input_paths(&tree), picked, "the sample must be deterministic");
        assert!(key_files(&tree).contains(&"worker/cmd/worker/main.go".to_string()));
    }

    #[test]
    fn under_the_cap_nothing_is_dropped() {
        let mut tree = oversized_tree();
        tree.truncate(MAX_PATHS);
        let mut expected = tree.clone();
        expected.sort();
        assert_eq!(input_paths(&tree), expected);
    }

    #[test]
    fn the_prompt_says_when_the_list_is_a_sample() {
        let listed = input_paths(&tree());
        assert!(prompt("acme", "widgets", "main", &listed, 4, &[]).contains("Files (4 of 4):"));
        assert!(prompt("acme", "widgets", "main", &listed, 900, &[])
            .contains("Files (4 of 900, sampled across directories"));
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
