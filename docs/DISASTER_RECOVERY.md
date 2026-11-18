# Disaster recovery

How to get the database back when the app itself is gone or won't start. For a normal
restore, use **Backups → ⋯ → Restore** in the app; it does all of this for you.

## What a backup is

- A `pg_dump --format=custom` file of the whole database.
- With `backup.encrypt_mode = age`, the file is encrypted to `BACKUP_PUBLIC_KEY` and ends in
  `.dump.age`. Only the **private key** opens it. It is never stored on the server.
- With `backup.encrypt_mode = symmetric`, it is encrypted with the passphrase in
  `BACKUP_ENCRYPTION_KEY` (also `.dump.age`).
- The SHA-256 of the stored file is in the `backups` table and in the backup list (Checksum).
- Copies: `BACKUP_DIR` on the server, and with `backup.offsite_enabled` one copy per backup
  in R2/S3 (`backups/<file>`) or Google Drive (the folder `GDRIVE_FOLDER_ID`).

## Keys

Create the key pair once and keep the private key **off the server** (password manager, safe):

```bash
startup backup-keygen        # prints BACKUP_PUBLIC_KEY=age1… and AGE-SECRET-KEY-1…
```

Put only `BACKUP_PUBLIC_KEY` in the server's `.env`. Losing the private key means losing
every encrypted backup.

## Restore with no running app

Needs `age` (https://age-encryption.org) and PostgreSQL client tools of the same major
version as the server (or newer).

```bash
# 1. Get the file: from the server's BACKUP_DIR, or the off-site copy, e.g.
aws s3 cp s3://$R2_BUCKET/backups/backup_20261118_020000_ab12cd.dump.age . \
    --endpoint-url https://$R2_ACCOUNT_ID.r2.cloudflarestorage.com

# 2. Check it is the file that was written (compare with the Checksum column / your notes)
sha256sum backup_20261118_020000_ab12cd.dump.age

# 3. Decrypt (skip for plain .dump files)
age -d -i private-key.txt -o backup.dump backup_20261118_020000_ab12cd.dump.age
#   passphrase backups: age -d -o backup.dump file.dump.age   (asks for BACKUP_ENCRYPTION_KEY)

# 4. Restore into an empty or existing database
pg_restore --clean --if-exists --no-owner --single-transaction \
    --dbname "postgres://USER@HOST:5432/DBNAME" backup.dump

# 5. Start the app: pending migrations run on boot (or run `startup migrate` first).
#    Everyone signs in again; delete old sessions to be sure:
psql "postgres://USER@HOST:5432/DBNAME" -c "DELETE FROM sessions"

# 6. Remove the decrypted file
shred -u backup.dump
```

## Checks

- `startup backup-verify` re-reads every stored backup (size + SHA-256) and every off-site
  copy (exists, same size). The app also runs this weekly; problems show as **Check failed**
  in the backup list.
- **System Setting → Storage → Test off-site connection** writes, reads and deletes a small
  file with the current off-site settings.
- Practise a restore on a spare database every few months.
