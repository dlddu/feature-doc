//! The analysis pipeline's stage list — one place, mirrored by the S04 mockup.
//!
//! `docs/mockups/JRN-discover-features.html#STP-leave-and-return` renders exactly these five steps
//! ("Pipeline · 3 of 5"), so the screen and the persisted rows cannot drift apart.
//! Every analysis is seeded with one `analysis_stages` row per entry at enqueue.
//!
//! [`FETCH`] and [`CROSS_CUTTING`] run today. Stages 3-5 stay `pending` until the
//! rest of the pipeline lands (AC1.3~AC1.4) — the worker never fabricates a result
//! for work that is not implemented.

/// One step of the pipeline as the user sees it on S04.
pub struct Stage {
    /// 1-based position; also the `analysis_stages.seq` column.
    pub seq: i64,
    /// Stable identifier used on the wire (`/internal/analyses/{id}/stages/{key}`).
    pub key: &'static str,
    /// Label rendered by S04.
    pub title: &'static str,
}

/// The one stage that is executable without an LLM (repository fetch + measure).
pub const FETCH: &str = "fetch";

/// Stage 2 (AC1.2): cross-cutting concerns, the first LLM-backed stage.
pub const CROSS_CUTTING: &str = "cross_cutting";

pub const STAGES: [Stage; 5] = [
    Stage { seq: 1, key: FETCH, title: "Fetch repository" },
    Stage { seq: 2, key: CROSS_CUTTING, title: "Cross-cutting concerns" },
    Stage { seq: 3, key: "discovery_strategy", title: "Discovery strategy" },
    Stage { seq: 4, key: "feature_candidates", title: "Extract feature candidates" },
    Stage { seq: 5, key: "acceptance_dependencies", title: "Acceptance & dependencies" },
];

/// Looks a stage up by its wire key.
pub fn stage(key: &str) -> Option<&'static Stage> {
    STAGES.iter().find(|s| s.key == key)
}

/// Analysis lifecycle values that live in `analyses.status`.
pub mod status {
    /// Enqueued, waiting for a worker to claim it.
    pub const QUEUED: &str = "queued";
    /// Claimed by a worker and within its lease.
    pub const RUNNING: &str = "running";
    /// Every *implemented* stage finished; the remaining stages are still
    /// `pending` because their implementation has not landed yet (AC1.2~AC1.4).
    /// Deliberately not `succeeded` — the analysis is not complete, and saying so
    /// would overstate what ran.
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

    /// Whether a worker-reported stage status is one we accept.
    pub fn is_reportable(s: &str) -> bool {
        matches!(s, RUNNING | SUCCEEDED | FAILED)
    }
}
