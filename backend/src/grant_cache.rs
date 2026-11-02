//! A short memory of "this running analysis could still read its repository", so the
//! worker's heartbeat does not ask GitHub on every beat.
//!
//! The heartbeat is where an App uninstall or a narrowed repository selection is
//! noticed mid-run (AC4.1). Asking GitHub costs an installation-token mint plus a full
//! repository listing — most of a heartbeat's time. Within [`GRANT_TTL_SECONDS`] of
//! the last successful check the beat skips that, so a change made on GitHub's side
//! is noticed at most that much later. Only successes are remembered, and the
//! installation row is still read on every beat, so disconnecting inside FeatureDoc
//! stops the run at once. The key includes the installation id: a re-installed App is
//! checked afresh. The memory is per process — the API runs as one replica.

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

pub const GRANT_TTL_SECONDS: i64 = 60;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Key {
    api_base: String,
    analysis_id: String,
    installation_id: i64,
}

impl Key {
    pub fn new(api_base: &str, analysis_id: &str, installation_id: i64) -> Key {
        Key {
            api_base: api_base.to_owned(),
            analysis_id: analysis_id.to_owned(),
            installation_id,
        }
    }
}

static GRANTS: LazyLock<Mutex<HashMap<Key, i64>>> = LazyLock::new(Default::default);

/// Whether `key` was granted less than [`GRANT_TTL_SECONDS`] before `now`.
pub fn fresh(key: &Key, now: i64) -> bool {
    let grants = GRANTS.lock().unwrap_or_else(|e| e.into_inner());
    grants
        .get(key)
        .is_some_and(|at| now - at < GRANT_TTL_SECONDS)
}

/// Records a successful check at `now`, dropping entries that have gone stale so
/// finished analyses do not accumulate.
pub fn remember(key: Key, now: i64) {
    let mut grants = GRANTS.lock().unwrap_or_else(|e| e.into_inner());
    grants.retain(|_, at| now - *at < GRANT_TTL_SECONDS);
    grants.insert(key, now);
}

pub fn forget(key: &Key) {
    GRANTS.lock().unwrap_or_else(|e| e.into_inner()).remove(key);
}

#[cfg(test)]
mod tests {
    use super::*;

    const T: i64 = 1_000_000;

    fn key(analysis: &str, installation: i64) -> Key {
        Key::new("http://gh.test", analysis, installation)
    }

    #[test]
    fn a_grant_is_fresh_only_within_the_window() {
        let k = key("fresh-window", 1);
        assert!(!fresh(&k, T));
        remember(k.clone(), T);
        assert!(fresh(&k, T));
        assert!(fresh(&k, T + GRANT_TTL_SECONDS - 1));
        assert!(!fresh(&k, T + GRANT_TTL_SECONDS));
    }

    #[test]
    fn another_installation_or_analysis_is_checked_afresh() {
        remember(key("scoped", 1), T);
        assert!(!fresh(&key("scoped", 2), T));
        assert!(!fresh(&key("scoped-other", 1), T));
    }

    #[test]
    fn forgetting_drops_the_grant() {
        let k = key("forget", 1);
        remember(k.clone(), T);
        forget(&k);
        assert!(!fresh(&k, T));
    }
}
