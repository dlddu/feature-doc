//! Stage 1 (`fetch`): resolve a repository tree and measure it.
//!
//! This is the only pipeline stage that needs no LLM, so it is the one the worker
//! actually executes in this slice. It replaces the size-derived guess used for the
//! S03 pre-flight with a *measured* file count and byte total — the same pair the
//! S04 mockup shows under "Fetch repository" ("847 files · 2.3 MB").
//!
//! Free functions rather than methods on `AppState`: the worker process has no
//! database and no `AppState`, only an HTTP client and a mode.

use serde::Deserialize;

use crate::config::Mode;

/// What stage 1 measured.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScanResult {
    pub files: i64,
    pub bytes: i64,
}

impl ScanResult {
    /// The one-liner S04 renders under the step ("847 files · 2.3 MB").
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
///
/// `Mode::Stub` answers deterministically from the repository name so the kind e2e
/// and unit tests never reach the network — the same doubling used by
/// `github_app::stub_repositories`. `Mode::Real` reads the recursive git tree with
/// the caller's short-lived installation token.
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
        Mode::Stub => Ok(stub_scan(name)),
        Mode::Real => real_scan(http, api_base, owner, name, branch, token).await,
    }
}

/// Deterministic stand-in derived from the stub repository sizes in `github_app`
/// (payments-api 2300 KiB, checkout-web 5100 KiB, notif-worker 800 KiB). Keeping
/// the ~3 KiB-per-file ratio of the pre-flight heuristic makes the measured value
/// and the estimate tell a coherent story in tests.
pub fn stub_scan(name: &str) -> ScanResult {
    let size_kb: i64 = match name {
        "payments-api" => 2300,
        "checkout-web" => 5100,
        "notif-worker" => 800,
        _ => 1024,
    };
    ScanResult {
        files: (size_kb / 3).max(1),
        bytes: size_kb * 1024,
    }
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
        // Never interpolate the response or the token into the message (AC4.3).
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
    for b in blobs {
        files += 1;
        bytes += b.size;
    }
    Ok(ScanResult { files, bytes })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_scan_is_deterministic_and_positive() {
        let a = stub_scan("payments-api");
        let b = stub_scan("payments-api");
        assert_eq!(a, b);
        assert!(a.files > 0 && a.bytes > 0);
        assert_ne!(stub_scan("checkout-web"), a);
    }

    #[test]
    fn detail_reads_like_the_s04_step() {
        let r = ScanResult {
            files: 847,
            bytes: 2_411_724,
        };
        assert_eq!(r.detail(), "847 files · 2.3 MB");
    }

    #[test]
    fn human_size_covers_each_unit() {
        assert_eq!(human_size(512), "512 B");
        assert_eq!(human_size(2048), "2 KB");
        assert_eq!(human_size(5 * 1024 * 1024), "5.0 MB");
    }
}
