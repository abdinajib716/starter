# Deploying to a VPS

One Linux server with Docker runs the whole stack: PostgreSQL 17, the app, headless Chrome
(PDF) and Caddy (HTTPS). Images come from GitHub Container Registry; a GitHub Action deploys
a tag and rolls back on its own if the new version doesn't become healthy.

## 1. Server (once)

```bash
# Ubuntu 24.04, 2 vCPU / 4 GB is plenty to start
curl -fsSL https://get.docker.com | sh
adduser --disabled-password deploy && usermod -aG docker deploy
ufw allow OpenSSH && ufw allow 80 && ufw allow 443 && ufw enable
```

DNS: an `A` record for your domain (e.g. `app.example.com`) → the server. Multi-tenant mode
also needs `*.app.example.com` (see §6).

## 2. Files on the server

As `deploy`, in `/srv/starter`:

- `docker-compose.prod.yml` and `deploy/Caddyfile` from this repository
- `.env` from `.env.example`, with at least:

```bash
APP_ENV=production
APP_URL=https://app.example.com
APP_SECRET=<openssl rand -base64 48>          # never change it later: 2FA secrets depend on it
DOMAIN=app.example.com
POSTGRES_PASSWORD=<openssl rand -base64 24>
CLIENT_IP_HEADER=x-forwarded-for              # Caddy sets it
SEED_ADMIN_EMAIL=you@example.com
SEED_ADMIN_PASSWORD=<temporary; changed at first sign-in>
BACKUP_PUBLIC_KEY=age1…                       # `docker run --rm IMAGE backup-keygen`, keep the private key offline
# optional: RESEND_API_KEY, RESEND_WEBHOOK_SECRET, R2_*, GDRIVE_*
```

`chmod 600 .env`.

## 3. First start

```bash
export APP_IMAGE=ghcr.io/<owner>/<repo>:1.0.0
docker compose -f docker-compose.prod.yml pull
docker compose -f docker-compose.prod.yml run --rm app migrate
docker compose -f docker-compose.prod.yml up -d
echo "$APP_IMAGE" > .current_image
docker compose -f docker-compose.prod.yml exec app /app/server health && echo healthy
```

Sign in at `https://app.example.com` with `SEED_ADMIN_*`, change the password, then in
**System Setting**: email (Resend + test email), backups (schedule, encryption `age`, off-site),
2FA mode.

## 4. Releases

1. Merge to `main` (CI: fmt, clippy server + wasm, `sqlx prepare --check`, tests with
   Postgres, `cargo deny`, tenant-scope check, release bundle).
2. Tag: `git tag v1.2.0 && git push --tags` → the `docker` workflow pushes
   `ghcr.io/<owner>/<repo>:1.2.0`.
3. Actions → **deploy** → tag `1.2.0`, environment `staging` or `production`.
   It pulls, runs `migrate`, starts the new app, waits for `/readyz`, and on failure starts
   the previous image again (`.current_image`).

Secrets per GitHub environment: `DEPLOY_HOST`, `DEPLOY_USER`, `DEPLOY_SSH_KEY`, `DEPLOY_PATH`
(`/srv/starter`). Protect `production` with required reviewers.

Migrations must stay backward compatible for one release (add columns/tables; drop in a
later release), because the rollback runs the previous image on the migrated database.

## 5. Operations

| Task | Command |
|---|---|
| Logs | `docker compose -f docker-compose.prod.yml logs -f app` |
| Readiness | `curl -s https://app.example.com/readyz` |
| New Super Admin | `docker compose … run --rm -e ADMIN_PASSWORD=… app create-admin a@b.c "Name"` |
| Remove someone's 2FA | `docker compose … run --rm app reset-2fa user@example.com` |
| Check backups | `docker compose … run --rm app backup-verify` |
| Restore without the app | [DISASTER_RECOVERY.md](DISASTER_RECOVERY.md) |

Backups are written to the `appdata` volume (`/app/data/backups`); turn on off-site copies so a
lost server doesn't take them along.

## 6. Multi-tenant mode

```bash
TENANCY_MODE=multi
APP_BASE_DOMAIN=app.example.com     # tenants at <slug>.app.example.com
```

- DNS `*.app.example.com` → the server, and a wildcard certificate: build Caddy with your DNS
  provider's module and add `*.{$DOMAIN}` with `tls { dns <provider> … }` to the Caddyfile.
- Row-level security: connect the app as a non-owner role so the policies apply
  (migrations keep using the owner):

```sql
CREATE ROLE starter_app LOGIN PASSWORD '…';
GRANT CONNECT ON DATABASE starter TO starter_app;
GRANT USAGE ON SCHEMA public TO starter_app;
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO starter_app;
GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO starter_app;
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO starter_app;
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT USAGE, SELECT ON SEQUENCES TO starter_app;
```

  Run `migrate` with the owner's `DATABASE_URL`, the app with `starter_app`'s.
- More than one app container (`APP_INSTANCES>1`) needs R2/S3 as primary storage.

## 7. Security headers

The app sends them itself (`src/server/security.rs`): `Content-Security-Policy`,
`Strict-Transport-Security` (production), `X-Content-Type-Options`, `X-Frame-Options: DENY`,
`Referrer-Policy`, `Permissions-Policy`. Check after a deploy:

```bash
curl -sI https://app.example.com/sign-in | grep -iE 'content-security|strict-transport|x-frame|x-content|referrer|permissions'
```
