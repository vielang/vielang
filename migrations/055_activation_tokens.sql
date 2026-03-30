CREATE TABLE user_activation_token (
    token       TEXT PRIMARY KEY,
    user_id     UUID NOT NULL REFERENCES tb_user(id) ON DELETE CASCADE,
    expires_at  BIGINT NOT NULL,
    used        BOOLEAN NOT NULL DEFAULT FALSE,
    created_at  BIGINT NOT NULL
);
CREATE INDEX idx_activation_token_user ON user_activation_token(user_id);
CREATE INDEX idx_activation_token_expiry ON user_activation_token(expires_at) WHERE used = FALSE;
