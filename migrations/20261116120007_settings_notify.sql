-- §28: every instance keeps an in-memory settings cache; a change on any instance is announced
-- on `settings_changed` (payload = key) so the others reload. Same channel for feature flags
-- and permission-registry changes later.
CREATE FUNCTION notify_settings_changed() RETURNS trigger AS $$
BEGIN
  PERFORM pg_notify('settings_changed', COALESCE(NEW.key, OLD.key));
  RETURN NULL;
END $$ LANGUAGE plpgsql;
CREATE TRIGGER settings_changed AFTER INSERT OR UPDATE OR DELETE ON settings
  FOR EACH ROW EXECUTE FUNCTION notify_settings_changed();
