-- M12.2: encrypted backups and off-site copies (§24.2).
ALTER TABLE backups
  ADD COLUMN encrypted     boolean NOT NULL DEFAULT false,
  ADD COLUMN enc_scheme    text,                 -- 'age' (public key) | 'passphrase'
  ADD COLUMN verified_at   timestamptz,          -- last successful verification
  ADD COLUMN verify_error  text;

CREATE TABLE backup_copies (
  id              uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  backup_id       uuid NOT NULL REFERENCES backups(id) ON DELETE CASCADE,
  target          text NOT NULL,                 -- 'r2' | 'gdrive'
  remote_key      text NOT NULL,
  status          text NOT NULL DEFAULT 'pending', -- pending | done | failed
  attempts        int NOT NULL DEFAULT 0,
  next_attempt_at timestamptz NOT NULL DEFAULT now(),
  error           text,
  size_bytes      bigint,
  verified_at     timestamptz,
  created_at      timestamptz NOT NULL DEFAULT now(),
  UNIQUE (backup_id, target)
);
CREATE TRIGGER backup_copies_notify AFTER INSERT OR UPDATE ON backup_copies
  FOR EACH STATEMENT EXECUTE FUNCTION notify_event();
