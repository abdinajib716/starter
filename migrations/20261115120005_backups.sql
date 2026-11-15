-- §6 0005_backups
CREATE TABLE backups (
  id          uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  filename    text NOT NULL UNIQUE,
  size_bytes  bigint NOT NULL DEFAULT 0,
  sha256      text,
  status      text NOT NULL,                     -- 'running'|'completed'|'failed'
  kind        text NOT NULL,                     -- 'manual'|'scheduled'|'pre_restore'
  error       text,
  created_by  uuid REFERENCES users(id),
  created_at  timestamptz NOT NULL DEFAULT now(),
  finished_at timestamptz
);
