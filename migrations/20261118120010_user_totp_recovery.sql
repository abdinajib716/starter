-- M9.6: TOTP 2FA (§21). The secret is AES-256-GCM encrypted with a key derived from APP_SECRET.
CREATE TABLE user_totp (
  user_id      uuid PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
  secret_enc   bytea NOT NULL,                 -- nonce (12) || ciphertext
  confirmed_at timestamptz,                    -- NULL = enrolment started, not confirmed
  last_step    bigint NOT NULL DEFAULT 0,      -- replay block: a code's time step is used once
  created_at   timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE recovery_codes (
  id        uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id   uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  code_hash bytea NOT NULL,                    -- SHA-256 of the normalised code
  used_at   timestamptz
);
CREATE INDEX recovery_codes_user_idx ON recovery_codes(user_id);

-- Failed second-step attempts on a pending_2fa session (5 → session dropped, account locked).
ALTER TABLE sessions ADD COLUMN mfa_attempts int NOT NULL DEFAULT 0;
