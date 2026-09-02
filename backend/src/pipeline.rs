//! The analysis pipeline's stage list — one place, mirrored by the S04 mockup.
//!
//! `docs/mockups/JRN-discover-features.html#STP-leave-and-return` renders exactly these five steps
//! ("Pipeline · 3 of 5"), so the screen and the persisted rows cannot drift apart.
//! Every analysis is seeded with one `analysis_stages` row per entry at enqueue.
//!
//! All five stages run today. The last one to land is [`ACCEPTANCE_DEPENDENCIES`],
//! whose wire key still names the two halves of PRD-2 that the roadmap once put in
//! one slice. Only the acceptance half is implemented (AC2.1~AC2.3); the dependency
//! half (AC2.4~AC2.6) is a *per-feature* action in `docs/test/02` 시나리오 5, not a
//! pipeline step, which is also why the S04 mockup draws this step as
//! 「인수 시나리오 생성」. The key is a wire contract (`/internal/.../stages/{key}`,
//! `analysis_documents.kind`) and stays; the title says what actually runs.

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

/// Stage 3 (AC1.3): the discovery strategy the user reviews and approves.
pub const DISCOVERY_STRATEGY: &str = "discovery_strategy";

/// Stage 4 (AC1.4): feature candidate extraction. Withheld from the queue until the
/// user approves the discovery strategy stage 3 proposed — AC1.3's gate is
/// expressed as a property of the queue, not a rule each worker remembers.
pub const FEATURE_CANDIDATES: &str = "feature_candidates";

/// Stage 5 (AC2.1~AC2.3): the acceptance scenarios a reviewer reads on S08.
///
/// Gated one step further along than stage 4: a scenario is *about a feature*, so
/// this stage is withheld until the reviewer has approved at least one candidate.
/// Same mechanism as AC1.3's gate — a property of the queue, not a rule each worker
/// remembers.
pub const ACCEPTANCE_DEPENDENCIES: &str = "acceptance_dependencies";

pub const STAGES: [Stage; 5] = [
    Stage { seq: 1, key: FETCH, title: "Fetch repository" },
    Stage { seq: 2, key: CROSS_CUTTING, title: "Cross-cutting concerns" },
    Stage { seq: 3, key: DISCOVERY_STRATEGY, title: "Discovery strategy" },
    Stage { seq: 4, key: FEATURE_CANDIDATES, title: "Extract feature candidates" },
    Stage { seq: 5, key: ACCEPTANCE_DEPENDENCIES, title: "Acceptance scenarios" },
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

    /// Whether a worker-reported stage status is one we accept.
    pub fn is_reportable(s: &str) -> bool {
        matches!(s, RUNNING | SUCCEEDED | FAILED)
    }
}
