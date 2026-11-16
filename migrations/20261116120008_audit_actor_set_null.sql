-- M5.11: purging a user must be possible. `audit_logs.actor_id` is `ON DELETE SET NULL`,
-- which Postgres performs as an UPDATE — the append-only trigger rejected it. Allow exactly
-- that one change (actor_id → NULL, every other column unchanged); everything else is still
-- refused. `actor_email` keeps who did it.
CREATE OR REPLACE FUNCTION audit_immutable() RETURNS trigger AS $$
BEGIN
  IF TG_OP = 'UPDATE'
     AND OLD.actor_id IS NOT NULL AND NEW.actor_id IS NULL
     AND (to_jsonb(NEW) - 'actor_id') = (to_jsonb(OLD) - 'actor_id') THEN
    RETURN NEW;
  END IF;
  RAISE EXCEPTION 'audit_logs is append-only';
END $$ LANGUAGE plpgsql;
