-- §6 0001_init_auth: organizations, roles, users, sessions
CREATE EXTENSION IF NOT EXISTS citext;
CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- Always created, even in single-tenant mode (one default row). See §18.
CREATE TABLE organizations (
  id         uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  name       text NOT NULL,
  slug       citext NOT NULL UNIQUE,            -- subdomain / URL key
  is_default boolean NOT NULL DEFAULT false,    -- the single org in single mode
  is_active  boolean NOT NULL DEFAULT true,
  created_at timestamptz NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX one_default_org ON organizations(is_default) WHERE is_default;

CREATE TABLE roles (
  id          uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  org_id      uuid REFERENCES organizations(id),  -- NULL = platform role (Super Admin)
  name        text NOT NULL,                      -- unique per org, see index below
  description text NOT NULL DEFAULT '',
  require_2fa boolean NOT NULL DEFAULT false,     -- used when security.2fa_mode = required_roles (§21)
  is_system   boolean NOT NULL DEFAULT false,     -- cannot be deleted/renamed
  created_at  timestamptz NOT NULL DEFAULT now(),
  updated_at  timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE users (
  id                uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  org_id            uuid NOT NULL REFERENCES organizations(id),
  email             citext NOT NULL,              -- unique among non-deleted rows, see index below
  full_name         text NOT NULL,
  password_hash     text NOT NULL,
  role_id           uuid NOT NULL REFERENCES roles(id),
  avatar_key        text,                         -- storage key, see §24
  locale            text,                         -- NULL = org/app default, see §26
  timezone          text,                         -- NULL = app.timezone setting
  email_verified_at timestamptz,
  deleted_by        uuid,
  is_active         boolean NOT NULL DEFAULT true,
  must_change_pw    boolean NOT NULL DEFAULT false,
  failed_attempts   int NOT NULL DEFAULT 0,
  locked_until      timestamptz,
  last_login_at     timestamptz,
  created_at        timestamptz NOT NULL DEFAULT now(),
  updated_at        timestamptz NOT NULL DEFAULT now(),
  deleted_at        timestamptz                   -- soft delete
);
CREATE INDEX users_role_idx ON users(role_id);
CREATE INDEX users_org_idx  ON users(org_id);
-- soft delete frees the email for re-use; restore must re-check this (§25)
CREATE UNIQUE INDEX users_email_active ON users(org_id, email) WHERE deleted_at IS NULL;
-- role names unique per org; NULL org (platform roles) share one namespace
CREATE UNIQUE INDEX roles_name_per_org ON roles(coalesce(org_id, '00000000-0000-0000-0000-000000000000'::uuid), name);

CREATE TABLE sessions (
  id              uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  token_hash      bytea NOT NULL UNIQUE,          -- SHA-256 of the cookie token
  user_id         uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  ip              inet,
  user_agent      text,
  created_at      timestamptz NOT NULL DEFAULT now(),
  last_seen_at    timestamptz NOT NULL DEFAULT now(),
  expires_at      timestamptz NOT NULL,
  device_label    text,                           -- "Chrome on Windows", parsed from user_agent (§22)
  mfa_verified_at timestamptz,                    -- NULL = password only (§21)
  kind            text NOT NULL DEFAULT 'web'     -- 'web' | 'pending_2fa' | 'api_refresh'
);
CREATE INDEX sessions_user_idx ON sessions(user_id);
CREATE INDEX sessions_exp_idx  ON sessions(expires_at);
