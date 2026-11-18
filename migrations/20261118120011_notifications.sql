-- M8.6: in-app notifications (§23).
CREATE TABLE notifications (
  id         bigserial PRIMARY KEY,
  org_id     uuid NOT NULL REFERENCES organizations(id),
  user_id    uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  kind       text NOT NULL,                   -- 'new_device' | 'account_locked' | 'announcement' | …
  params     jsonb NOT NULL DEFAULT '{}',     -- values for the translated message
  link       text,                            -- in-app path to open
  read_at    timestamptz,
  created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX notifications_user_idx ON notifications(user_id, created_at DESC);
CREATE INDEX notifications_unread_idx ON notifications(user_id) WHERE read_at IS NULL;

-- Per user, per kind: off = not delivered in-app (email column used once M13 is on).
CREATE TABLE notification_prefs (
  user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  kind    text NOT NULL,
  in_app  boolean NOT NULL DEFAULT true,
  email   boolean NOT NULL DEFAULT true,
  PRIMARY KEY (user_id, kind)
);
