-- Repositories a GitHub App installation can access — the candidate set that
-- `connect` (POST /api/repositories) checks against (AC1.1).
--
-- In real mode this set is fetched live from GitHub and this table is unused. In
-- stub mode it is the seam for *test data*: the `seed` binary populates it so the
-- application code carries no hardcoded fixtures. github_app::list_repositories
-- reads it in stub mode.
CREATE TABLE installation_repositories (
    installation_id INTEGER NOT NULL,
    owner           TEXT    NOT NULL,
    name            TEXT    NOT NULL,
    default_branch  TEXT    NOT NULL,
    PRIMARY KEY (installation_id, owner, name)
);
