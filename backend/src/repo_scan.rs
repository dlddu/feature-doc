//! Stage 1 (`fetch`): resolve a repository tree and measure it.
//!
//! Free functions rather than methods on `AppState`: the worker process has no
//! database and no `AppState`, only an HTTP client and a mode.

use serde::Deserialize;

use crate::config::Mode;

/// What stage 1 measured.
///
/// `paths` is the blob path list the tree request already returns. Stage 1 only
/// needs the count and the byte total, but stage 2 (`cross_cutting`, AC1.2) needs
/// the paths themselves as its evidence source — so they are carried forward
/// rather than re-fetched.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanResult {
    pub files: i64,
    pub bytes: i64,
    pub paths: Vec<String>,
}

impl ScanResult {
    pub fn detail(&self) -> String {
        format!("{} files · {}", self.files, human_size(self.bytes))
    }
}

fn human_size(bytes: i64) -> String {
    if bytes < 1024 {
        return format!("{bytes} B");
    }
    let kb = bytes as f64 / 1024.0;
    if kb < 1024.0 {
        return format!("{} KB", kb.round() as i64);
    }
    format!("{:.1} MB", kb / 1024.0)
}

/// Counts the blobs in `owner/name@branch` and sums their sizes.
pub async fn scan(
    http: &reqwest::Client,
    mode: Mode,
    api_base: &str,
    owner: &str,
    name: &str,
    branch: &str,
    token: Option<&str>,
) -> Result<ScanResult, String> {
    match mode {
        // mock-exception: EXT-03 — 트리 스캔은 실 설치 토큰으로 실 저장소 트리를 읽어야 한다
        Mode::Stub => stub_scan(name, branch),
        Mode::Real => real_scan(http, api_base, owner, name, branch, token).await,
    }
}

/// The branch every `github_app::stub_repositories` entry reports as its default —
/// and, since the stub set has no other refs, the only one that resolves.
const STUB_BRANCH: &str = "main";

/// Deterministic stand-in sized from `github_app`'s stub repositories. The
/// ~3 KiB-per-file ratio is the pre-flight heuristic's, so the measured value and
/// the estimate tell a coherent story in tests.
///
/// A branch the stub repositories do not have fails the same way the real tree
/// request does — GitHub answers `404` for an unknown ref, and [`real_scan`] turns
/// that into `github tree rejected (404)`. Returning a measurement for a ref that
/// does not exist would make the double *more* forgiving than the API it stands in
/// for, and a fetch failure would then be unreachable outside production.
pub fn stub_scan(name: &str, branch: &str) -> Result<ScanResult, String> {
    stub_scan_at(name, branch, revision_from_env())
}

/// Which step of the stub tree's history a scan sees.
///
/// Each revision only **adds** paths on top of the one before it — removing or
/// renaming a path would move the feature keys built on it, and «the same feature's
/// representation was updated» could no longer be observed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Revision {
    First,
    /// One more source file; the acceptance sentences stay as they were.
    Second,
    /// One more file again, and the logic pass reads its first scenario differently
    /// (`acceptance::stub_logic`) — a code change that rewrites a sentence a person
    /// may already have edited.
    Third,
}

impl Revision {
    fn added(self) -> &'static [&'static str] {
        match self {
            Revision::First => &[],
            Revision::Second => &["src/billing/refund.rs"],
            Revision::Third => &["src/billing/refund.rs", REWRITE_MARKER],
        }
    }
}

/// The path whose presence makes the acceptance logic pass rewrite a sentence.
pub const REWRITE_MARKER: &str = "src/billing/refund_policy.rs";

/// [`stub_scan`] with the revision decided by the caller.
///
/// The split exists so the property that matters — *a later revision only adds*
/// — is testable without mutating process-wide environment state, which other
/// tests in the same binary would race against.
pub fn stub_scan_at(name: &str, branch: &str, revision: Revision) -> Result<ScanResult, String> {
    if branch != STUB_BRANCH {
        return Err("github tree rejected (404)".to_string());
    }
    let size_kb: i64 = match name {
        "payments-api" => 2300,
        "checkout-web" => 5100,
        "notif-worker" => 800,
        _ => 1024,
    };
    let files = (size_kb / 3).max(1);
    let mut paths = stub_paths(name, files);
    if revision == Revision::First {
        return Ok(ScanResult {
            files,
            bytes: size_kb * 1024,
            paths,
        });
    }
    let added: Vec<String> = revision
        .added()
        .iter()
        .map(|p| format!("{name}/{p}"))
        .filter(|p| !paths.contains(p))
        .collect();
    let count = added.len() as i64;
    paths.extend(added);
    Ok(ScanResult {
        // 재분석의 1단계가 "그대로다"라고 말하면 거짓이 된다.
        files: files + count,
        bytes: size_kb * 1024 + count * 3 * 1024,
        paths,
    })
}

/// 재분석이 한 걸음 나아간 트리를 보게 하는 결정적 트리거.
///
/// 실 모드에서는 시간이 지나면 같은 브랜치의 트리가 달라지지만, 이름에서만 파생되는
/// stub 트리는 시간을 모른다 — `FEATUREDOC_STUB_LLM_FAIL` 이 LLM 더블 안에서 실패를
/// 재현하는 것과 같은 충실도 확장이다. 비어 있지 않은 값은 두 번째 리비전이고, `3` 만
/// 세 번째다 — 값을 세지 않던 시절의 spec 이 넣던 `2` 가 그대로 두 번째로 읽힌다.
fn revision_from_env() -> Revision {
    match std::env::var("FEATUREDOC_STUB_REPO_REVISION") {
        Ok(value) if value.trim() == "3" => Revision::Third,
        Ok(value) if !value.trim().is_empty() => Revision::Second,
        _ => Revision::First,
    }
}

/// A deterministic stand-in tree for the stub repositories.
///
/// Shaped like a real project rather than `file_0..file_N` so the stage-2 document
/// reads plausibly. The list is capped well below `files` — the measured count is
/// what stage 1 reports, and enumerating thousands of synthetic paths would only
/// bloat the prompt.
fn stub_paths(name: &str, files: i64) -> Vec<String> {
    const SHAPE: [&str; 12] = [
        "README.md",
        "Cargo.toml",
        "deploy/base/deployment.yaml",
        "deploy/base/kustomization.yaml",
        "src/main.rs",
        "src/lib.rs",
        "src/api/routes.rs",
        "src/domain/model.rs",
        "src/middleware/auth.rs",
        "src/middleware/logging.rs",
        "tests/integration.rs",
        ".github/workflows/ci.yml",
    ];
    let take = SHAPE.len().min(files.max(1) as usize);
    SHAPE
        .iter()
        .take(take)
        .map(|p| format!("{name}/{p}"))
        .collect()
}

/// The head of one file, as read for stage 2's context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileExcerpt {
    pub path: String,
    pub body: String,
    /// The file continued past `max_bytes`; the model is told so it does not
    /// mistake a cut-off head for the whole file.
    pub truncated: bool,
}

/// Reads the first `max_bytes` of each path in `owner/name@branch`.
///
/// A path that cannot be read (removed since the tree was listed, not text, a
/// transient 5xx) is left out rather than failing the caller: the excerpts are
/// context on top of the path list, and the stage still has its evidence without
/// them. A missing token is still an error, as it is for [`scan`].
pub async fn read_files(
    http: &reqwest::Client,
    mode: Mode,
    api_base: &str,
    owner: &str,
    name: &str,
    branch: &str,
    token: Option<&str>,
    paths: &[String],
    max_bytes: usize,
) -> Result<Vec<FileExcerpt>, String> {
    match mode {
        // mock-exception: EXT-03 — 파일 본문 읽기도 실 설치 토큰으로 실 저장소의 contents API를 불러야 한다
        Mode::Stub => stub_read(name, branch, paths, max_bytes),
        Mode::Real => {
            let token =
                token.ok_or_else(|| "no installation token for repository fetch".to_string())?;
            let mut out = Vec::new();
            for path in paths {
                match real_read(http, api_base, owner, name, branch, token, path).await {
                    Ok(bytes) => out.push(excerpt(path, &bytes, max_bytes)),
                    Err(reason) => {
                        tracing::warn!(%path, %reason, "file excerpt skipped");
                    }
                }
            }
            Ok(out)
        }
    }
}

/// Answers only for paths the stub tree of the same repository and branch holds —
/// a path outside it is skipped exactly as the real `404` is, so the double never
/// hands stage 2 a file the real API could not.
fn stub_read(
    name: &str,
    branch: &str,
    paths: &[String],
    max_bytes: usize,
) -> Result<Vec<FileExcerpt>, String> {
    let tree = stub_scan(name, branch)?.paths;
    Ok(paths
        .iter()
        .filter(|p| tree.contains(p))
        .map(|p| excerpt(p, format!("// stub body of {p}\n").as_bytes(), max_bytes))
        .collect())
}

/// Cuts on a UTF-8 boundary so a multi-byte character is never split into
/// replacement noise at the end of an excerpt.
fn excerpt(path: &str, bytes: &[u8], max_bytes: usize) -> FileExcerpt {
    let text = String::from_utf8_lossy(bytes);
    let mut end = text.len().min(max_bytes);
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    FileExcerpt {
        path: path.to_string(),
        body: text[..end].to_string(),
        truncated: end < text.len(),
    }
}

async fn real_read(
    http: &reqwest::Client,
    api_base: &str,
    owner: &str,
    name: &str,
    branch: &str,
    token: &str,
    path: &str,
) -> Result<Vec<u8>, String> {
    let mut url =
        url::Url::parse(api_base).map_err(|_| "github api base is not a URL".to_string())?;
    url.path_segments_mut()
        .map_err(|_| "github api base cannot take a path".to_string())?
        .pop_if_empty()
        .extend(["repos", owner, name, "contents"])
        .extend(path.split('/'));
    url.query_pairs_mut().append_pair("ref", branch);
    let resp = http
        .get(url)
        .header("Authorization", format!("Bearer {token}"))
        .header("Accept", "application/vnd.github.raw+json")
        .header("User-Agent", "featuredoc-worker/0.1")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .await
        .map_err(|_| "github contents request failed".to_string())?;
    if !resp.status().is_success() {
        return Err(format!("github contents rejected ({})", resp.status().as_u16()));
    }
    resp.bytes()
        .await
        .map(|b| b.to_vec())
        .map_err(|_| "github contents: body unreadable".to_string())
}

async fn real_scan(
    http: &reqwest::Client,
    api_base: &str,
    owner: &str,
    name: &str,
    branch: &str,
    token: Option<&str>,
) -> Result<ScanResult, String> {
    let token = token.ok_or_else(|| "no installation token for repository fetch".to_string())?;
    let url = format!("{api_base}/repos/{owner}/{name}/git/trees/{branch}?recursive=1");

    let resp = http
        .get(&url)
        .header("Authorization", format!("Bearer {token}"))
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "featuredoc-worker/0.1")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .await
        // Never interpolate the response or the token into the message.
        .map_err(|_| "github tree request failed".to_string())?;

    if !resp.status().is_success() {
        return Err(format!("github tree rejected ({})", resp.status().as_u16()));
    }

    #[derive(Deserialize)]
    struct Entry {
        #[serde(rename = "type")]
        kind: Option<String>,
        #[serde(default)]
        size: i64,
        #[serde(default)]
        path: Option<String>,
    }
    #[derive(Deserialize)]
    struct Tree {
        #[serde(default)]
        tree: Vec<Entry>,
        #[serde(default)]
        truncated: bool,
    }

    let tree: Tree = resp
        .json()
        .await
        .map_err(|_| "github tree: malformed response".to_string())?;

    // A truncated tree would undercount silently. Say so rather than report a
    // number the user cannot trust; paging arrives with the real pipeline.
    if tree.truncated {
        return Err("repository tree too large to measure in one request".to_string());
    }

    let blobs = tree
        .tree
        .iter()
        .filter(|e| e.kind.as_deref() == Some("blob"));
    let mut files = 0i64;
    let mut bytes = 0i64;
    let mut paths = Vec::new();
    for b in blobs {
        files += 1;
        bytes += b.size;
        if let Some(p) = &b.path {
            paths.push(p.clone());
        }
    }
    Ok(ScanResult {
        files,
        bytes,
        paths,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_read_answers_only_for_paths_in_the_stub_tree() {
        let paths = vec![
            "payments-api/src/main.rs".to_string(),
            "payments-api/not/in/tree.rs".to_string(),
        ];
        let read = stub_read("payments-api", "main", &paths, 4096).unwrap();
        assert_eq!(read.len(), 1, "a path the real API would 404 must be skipped");
        assert_eq!(read[0].path, "payments-api/src/main.rs");
        assert!(!read[0].truncated);
        assert_eq!(
            stub_read("payments-api", "no-such-branch", &paths, 4096),
            Err("github tree rejected (404)".to_string())
        );
    }

    #[test]
    fn excerpt_cuts_on_a_char_boundary_and_says_it_cut() {
        let e = excerpt("a.md", "가나다".as_bytes(), 4);
        assert_eq!(e.body, "가");
        assert!(e.truncated);
        let whole = excerpt("a.md", b"abc", 10);
        assert_eq!(whole.body, "abc");
        assert!(!whole.truncated);
    }

    #[test]
    fn stub_scan_is_deterministic_and_positive() {
        let a = stub_scan("payments-api", "main").unwrap();
        let b = stub_scan("payments-api", "main").unwrap();
        assert_eq!(a, b);
        assert!(a.files > 0 && a.bytes > 0);
        assert_ne!(stub_scan("checkout-web", "main").unwrap(), a);
    }

    #[test]
    fn stub_scan_rejects_an_unknown_branch_like_the_real_tree_request() {
        assert_eq!(
            stub_scan("payments-api", "no-such-branch"),
            Err("github tree rejected (404)".to_string())
        );
    }

    #[test]
    fn detail_reads_like_the_s04_step() {
        let r = ScanResult {
            files: 847,
            bytes: 2_411_724,
            paths: Vec::new(),
        };
        assert_eq!(r.detail(), "847 files · 2.3 MB");
    }

    #[test]
    fn stub_scan_returns_a_stable_non_empty_path_list() {
        let a = stub_scan("payments-api", "main").unwrap();
        let b = stub_scan("payments-api", "main").unwrap();
        assert_eq!(a.paths, b.paths);
        assert!(!a.paths.is_empty());
        assert!(a.paths.iter().all(|p| p.starts_with("payments-api/")));
        assert_ne!(stub_scan("checkout-web", "main").unwrap().paths, a.paths);
    }

    #[test]
    fn the_second_revision_only_adds_paths() {
        let first = stub_scan_at("payments-api", "main", Revision::First).unwrap();
        let second = stub_scan_at("payments-api", "main", Revision::Second).unwrap();

        assert_eq!(
            second.paths[..first.paths.len()],
            first.paths[..],
            "앞선 리비전의 경로가 그 순서 그대로 남아 있어야 한다"
        );
        assert_eq!(second.paths.len(), first.paths.len() + 1);
        assert!(second
            .paths
            .contains(&"payments-api/src/billing/refund.rs".to_string()));
        assert!(second.files > first.files && second.bytes > first.bytes);
    }

    #[test]
    fn the_third_revision_adds_on_top_of_the_second() {
        let second = stub_scan_at("payments-api", "main", Revision::Second).unwrap();
        let third = stub_scan_at("payments-api", "main", Revision::Third).unwrap();

        assert_eq!(third.paths[..second.paths.len()], second.paths[..]);
        assert_eq!(third.paths.len(), second.paths.len() + 1);
        assert!(third
            .paths
            .contains(&format!("payments-api/{REWRITE_MARKER}")));
        assert!(!second.paths.iter().any(|p| p.ends_with(REWRITE_MARKER)));
    }

    /// 리비전을 켜지 않은 호출은 바이트 단위로 예전과 같다 — 다른 spec 들의
    /// 단정(파일 수·문서 내용)이 이 확장으로 흔들리지 않는다는 뜻이다.
    #[test]
    fn an_unrevised_scan_is_unchanged() {
        assert_eq!(
            stub_scan_at("payments-api", "main", Revision::First).unwrap(),
            stub_scan("payments-api", "main").unwrap()
        );
    }

    #[test]
    fn a_revised_scan_still_rejects_an_unknown_branch() {
        assert_eq!(
            stub_scan_at("payments-api", "no-such-branch", Revision::Third),
            Err("github tree rejected (404)".to_string())
        );
    }

    #[test]
    fn human_size_covers_each_unit() {
        assert_eq!(human_size(512), "512 B");
        assert_eq!(human_size(2048), "2 KB");
        assert_eq!(human_size(5 * 1024 * 1024), "5.0 MB");
    }
}
