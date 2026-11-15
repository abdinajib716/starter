-- §6 0004_settings: typed key/value settings (seeded in M2.8)
CREATE TABLE settings (
  key         text PRIMARY KEY,                  -- 'app.name', 'security.session_hours'
  value       jsonb NOT NULL,
  value_type  text NOT NULL,                     -- 'string'|'int'|'bool'|'enum'
  category    text NOT NULL,                     -- 'general'|'security'|'backup'
  label       text NOT NULL,
  description text NOT NULL DEFAULT '',
  updated_by  uuid REFERENCES users(id),
  updated_at  timestamptz NOT NULL DEFAULT now()
);
