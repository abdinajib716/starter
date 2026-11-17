-- §27 (blueprint 0011_feature_flags): flags declared in code, synced at startup; an org row
-- overrides the global (org_id NULL) row. Changes are announced on `settings_changed`.
CREATE TABLE feature_flags (
  key              text NOT NULL,
  org_id           uuid REFERENCES organizations(id),
  description      text NOT NULL DEFAULT '',
  enabled          boolean NOT NULL DEFAULT false,
  rollout_percent  int NOT NULL DEFAULT 100 CHECK (rollout_percent BETWEEN 0 AND 100),
  allowed_role_ids uuid[] NOT NULL DEFAULT '{}',
  allowed_user_ids uuid[] NOT NULL DEFAULT '{}',
  updated_by       uuid REFERENCES users(id) ON DELETE SET NULL,
  updated_at       timestamptz NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX feature_flags_key_org
  ON feature_flags (key, coalesce(org_id, '00000000-0000-0000-0000-000000000000'::uuid));

CREATE FUNCTION notify_flags_changed() RETURNS trigger AS $$
BEGIN
  PERFORM pg_notify('settings_changed', 'flag:' || COALESCE(NEW.key, OLD.key));
  RETURN NULL;
END $$ LANGUAGE plpgsql;
CREATE TRIGGER feature_flags_changed AFTER INSERT OR UPDATE OR DELETE ON feature_flags
  FOR EACH ROW EXECUTE FUNCTION notify_flags_changed();

-- Purge needs these references cleared too (M5.11 pattern): settings.updated_by → SET NULL.
ALTER TABLE settings DROP CONSTRAINT settings_updated_by_fkey,
  ADD CONSTRAINT settings_updated_by_fkey FOREIGN KEY (updated_by) REFERENCES users(id) ON DELETE SET NULL;
