-- M15.2: public API v1 (§19).
CREATE TABLE api_tokens (
  id           uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  org_id       uuid NOT NULL REFERENCES organizations(id),
  user_id      uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  name         text NOT NULL,
  prefix       text NOT NULL,                  -- first characters, shown in lists ("stk_ab12cd")
  token_hash   bytea NOT NULL UNIQUE,          -- SHA-256 of the whole token
  scopes       text[] NOT NULL,                -- permission keys; effective = scopes ∩ owner's permissions
  expires_at   timestamptz,
  last_used_at timestamptz,
  revoked_at   timestamptz,
  created_at   timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX api_tokens_user_idx ON api_tokens(user_id);

-- Replays of a POST with the same Idempotency-Key return the first answer (24 h).
CREATE TABLE idempotency_keys (
  user_id    uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  key        text NOT NULL,
  method     text NOT NULL,
  path       text NOT NULL,
  status     int NOT NULL,
  body       bytea NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY (user_id, key)
);
