CREATE TABLE push_subscriptions (
    endpoint   TEXT    PRIMARY KEY,
    user_id    TEXT    NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    p256dh     TEXT    NOT NULL,
    auth       TEXT    NOT NULL,
    created_at INTEGER NOT NULL
);

CREATE INDEX idx_push_subscriptions_user ON push_subscriptions(user_id);
