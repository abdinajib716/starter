-- §6 0003_audit: append-only audit log
CREATE TABLE audit_logs (
  id           bigserial PRIMARY KEY,
  at           timestamptz NOT NULL DEFAULT now(),
  org_id       uuid REFERENCES organizations(id),
  actor_id     uuid REFERENCES users(id) ON DELETE SET NULL,
  channel      text NOT NULL DEFAULT 'web',      -- 'web' | 'api' | 'system'
  actor_email  text,                             -- denormalized: survives user deletion
  action       text NOT NULL,                    -- 'user.create', 'auth.login_failed'
  entity_type  text,
  entity_id    text,
  before       jsonb,
  after        jsonb,
  ip           inet,
  user_agent   text,
  success      boolean NOT NULL DEFAULT true
);
CREATE INDEX audit_at_idx     ON audit_logs(at DESC);
CREATE INDEX audit_action_idx ON audit_logs(action);
CREATE INDEX audit_actor_idx  ON audit_logs(actor_id);

-- append-only: block UPDATE/DELETE
CREATE FUNCTION audit_immutable() RETURNS trigger AS $$
BEGIN RAISE EXCEPTION 'audit_logs is append-only'; END $$ LANGUAGE plpgsql;
CREATE TRIGGER audit_no_update BEFORE UPDATE OR DELETE ON audit_logs
  FOR EACH ROW EXECUTE FUNCTION audit_immutable();
-- (retention purge, if ever needed, runs as a separate DB role / TRUNCATE partition — see M4.9)
