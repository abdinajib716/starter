-- M14.1: shared state across app instances (§28).
CREATE TABLE system_state (
  key        text PRIMARY KEY,                 -- 'maintenance'
  value      jsonb NOT NULL,                   -- {"on": true, "reason": "restore", "since": …}
  updated_at timestamptz NOT NULL DEFAULT now()
);
INSERT INTO system_state (key, value) VALUES ('maintenance', '{"on": false}');

CREATE FUNCTION system_state_notify() RETURNS trigger AS $$
BEGIN
  PERFORM pg_notify('system_state', NEW.key);
  RETURN NULL;
END $$ LANGUAGE plpgsql;
CREATE TRIGGER system_state_changed AFTER INSERT OR UPDATE ON system_state
  FOR EACH ROW EXECUTE FUNCTION system_state_notify();

-- Fixed-window counters shared by every instance (login, password reset, API).
CREATE UNLOGGED TABLE rate_limits (
  key          text PRIMARY KEY,
  window_start timestamptz NOT NULL DEFAULT now(),
  count        int NOT NULL DEFAULT 0
);
