-- Realtime signals for role changes (roles page live, M8.5).
CREATE TRIGGER roles_notify AFTER INSERT OR UPDATE OR DELETE ON roles FOR EACH STATEMENT EXECUTE FUNCTION notify_event();
