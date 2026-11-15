-- §6 0006_notify_triggers (realtime, §11): signals, not payloads
CREATE FUNCTION notify_event() RETURNS trigger AS $$
BEGIN
  PERFORM pg_notify('app_events', json_build_object(
      'table', TG_TABLE_NAME, 'op', TG_OP)::text);
  RETURN NULL;
END $$ LANGUAGE plpgsql;
CREATE TRIGGER audit_notify    AFTER INSERT ON audit_logs FOR EACH STATEMENT EXECUTE FUNCTION notify_event();
CREATE TRIGGER users_notify    AFTER INSERT OR UPDATE OR DELETE ON users    FOR EACH STATEMENT EXECUTE FUNCTION notify_event();
CREATE TRIGGER sessions_notify AFTER INSERT OR DELETE ON sessions           FOR EACH STATEMENT EXECUTE FUNCTION notify_event();
CREATE TRIGGER backups_notify  AFTER INSERT OR UPDATE ON backups            FOR EACH STATEMENT EXECUTE FUNCTION notify_event();
