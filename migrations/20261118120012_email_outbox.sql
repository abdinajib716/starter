-- M13.1: transactional email outbox (§20). Written in the business transaction, sent by a worker.
CREATE TABLE email_outbox (
  id              bigserial PRIMARY KEY,
  org_id          uuid REFERENCES organizations(id),
  to_email        citext NOT NULL,
  template        text NOT NULL,                 -- 'invite' | 'password_reset' | …
  locale          text NOT NULL DEFAULT 'en',
  params          jsonb NOT NULL DEFAULT '{}',   -- secrets (links with tokens) are cleared after sending
  status          text NOT NULL DEFAULT 'pending', -- pending | sent | failed | delivered | bounced | complained | suppressed
  attempts        int NOT NULL DEFAULT 0,
  next_attempt_at timestamptz NOT NULL DEFAULT now(),
  last_error      text,
  provider_id     text,                          -- Resend email id (webhooks match on it)
  created_at      timestamptz NOT NULL DEFAULT now(),
  sent_at         timestamptz
);
CREATE INDEX email_outbox_due_idx ON email_outbox(next_attempt_at) WHERE status = 'pending';
CREATE INDEX email_outbox_provider_idx ON email_outbox(provider_id);

-- One-time links: password reset, invite, email verification. Only the SHA-256 is stored.
CREATE TABLE auth_tokens (
  id          uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  token_hash  bytea NOT NULL UNIQUE,
  user_id     uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  purpose     text NOT NULL,                     -- 'password_reset' | 'invite' | 'verify_email'
  expires_at  timestamptz NOT NULL,
  used_at     timestamptz,
  created_at  timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX auth_tokens_user_idx ON auth_tokens(user_id, purpose);

-- Hard bounces / complaints: never email these addresses again.
CREATE TABLE email_suppressions (
  email      citext PRIMARY KEY,
  reason     text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now()
);
