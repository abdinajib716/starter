-- M16: multi-tenant mode (§18).
-- Row-level security on every tenant table. The app sets `app.org_id` on each connection it
-- uses for a request (multi mode); background jobs and single mode leave it empty and see
-- every row. RLS applies to the app's non-owner role (see docs/DEPLOY.md); the owner role used
-- for migrations bypasses it.
CREATE FUNCTION app_org() RETURNS uuid LANGUAGE sql STABLE AS $$
  SELECT NULLIF(current_setting('app.org_id', true), '')::uuid
$$;

DO $$
DECLARE t text;
BEGIN
  FOREACH t IN ARRAY ARRAY['users', 'roles', 'audit_logs', 'notifications', 'files', 'api_tokens', 'email_outbox']
  LOOP
    EXECUTE format('ALTER TABLE %I ENABLE ROW LEVEL SECURITY', t);
    EXECUTE format(
      'CREATE POLICY tenant_isolation ON %I USING (app_org() IS NULL OR org_id IS NULL OR org_id = app_org())
                                         WITH CHECK (app_org() IS NULL OR org_id IS NULL OR org_id = app_org())', t);
  END LOOP;
END $$;

-- M16.6 optional plan limits (NULL = unlimited).
ALTER TABLE organizations
  ADD COLUMN max_users  int,
  ADD COLUMN storage_mb int;
