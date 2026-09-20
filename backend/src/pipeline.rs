//! The analysis pipeline's stage list.
//!
//! Every analysis is seeded with one `analysis_stages` row per entry at enqueue.
//!
//! [`ACCEPTANCE_DEPENDENCIES`]'s wire key names two halves that the roadmap once put
//! in one slice; only the acceptance half is a pipeline step. The key is a wire
//! contract (`/internal/.../stages/{key}`, `analysis_documents.kind`) and stays —
//! the title, not the key, says what actually runs.

pub struct Stage {
    /// 1-based position; also the `analysis_stages.seq` column.
    pub seq: i64,
    /// Stable identifier used on the wire (`/internal/analyses/{id}/stages/{key}`).
    pub key: &'static str,
    pub title: &'static str,
}

/// The one stage that is executable without an LLM (repository fetch + measure).
pub const FETCH: &str = "fetch";

pub const CROSS_CUTTING: &str = "cross_cutting";

pub const DISCOVERY_STRATEGY: &str = "discovery_strategy";

/// Withheld from the queue until the user approves the strategy stage 3 proposed —
/// the gate is a property of the queue, not a rule each worker remembers.
pub const FEATURE_CANDIDATES: &str = "feature_candidates";

/// Withheld one step further along than stage 4: a scenario is *about a feature*, so
/// the queue holds it back until the reviewer has approved at least one candidate.
pub const ACCEPTANCE_DEPENDENCIES: &str = "acceptance_dependencies";

pub const STAGES: [Stage; 5] = [
    Stage { seq: 1, key: FETCH, title: "Fetch repository" },
    Stage { seq: 2, key: CROSS_CUTTING, title: "Cross-cutting concerns" },
    Stage { seq: 3, key: DISCOVERY_STRATEGY, title: "Discovery strategy" },
    Stage { seq: 4, key: FEATURE_CANDIDATES, title: "Extract feature candidates" },
    Stage { seq: 5, key: ACCEPTANCE_DEPENDENCIES, title: "Acceptance scenarios" },
];

pub fn stage(key: &str) -> Option<&'static Stage> {
    STAGES.iter().find(|s| s.key == key)
}

/// Analysis lifecycle values that live in `analyses.status`.
pub mod status {
    pub const QUEUED: &str = "queued";
    /// Claimed by a worker and within its lease.
    pub const RUNNING: &str = "running";
    /// Every stage this pass was *offered* finished, and the ones that were not
    /// offered are waiting on a person: stage 4 on the strategy approval, stage 5
    /// on at least one approved feature candidate. Deliberately not `succeeded` —
    /// the analysis is not complete, and saying so would overstate what ran.
    pub const AWAITING_PIPELINE: &str = "awaiting_pipeline";
    /// A stage failed; `analyses.error` carries the operator-facing reason.
    pub const FAILED: &str = "failed";
}

/// Values that live in `analysis_stages.status`.
pub mod stage_status {
    pub const PENDING: &str = "pending";
    pub const RUNNING: &str = "running";
    pub const SUCCEEDED: &str = "succeeded";
    pub const FAILED: &str = "failed";

    pub fn is_reportable(s: &str) -> bool {
        matches!(s, RUNNING | SUCCEEDED | FAILED)
    }
}
