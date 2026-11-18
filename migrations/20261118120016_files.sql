-- M11.2: uploaded files (§24.1). The bytes live in storage (local disk or R2/S3) under
-- `storage_key`; this row is the index and the access check.
CREATE TABLE files (
  id          uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  org_id      uuid NOT NULL REFERENCES organizations(id),
  owner_id    uuid REFERENCES users(id) ON DELETE SET NULL,
  kind        text NOT NULL,                 -- 'avatar' | future kinds
  driver      text NOT NULL,                 -- 'local' | 'r2' | 'gdrive'
  storage_key text NOT NULL,
  mime        text NOT NULL,
  size_bytes  bigint NOT NULL,
  sha256      text NOT NULL,
  created_at  timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX files_owner_idx ON files(owner_id, kind);
