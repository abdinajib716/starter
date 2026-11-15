-- §6 0002_rbac: code-defined permission catalog + role grants
CREATE TABLE permissions (
  key         text PRIMARY KEY,                  -- e.g. 'users.create'
  module      text NOT NULL,                     -- 'users'
  action      text NOT NULL,                     -- 'create'
  description text NOT NULL
);

CREATE TABLE role_permissions (
  role_id        uuid NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
  permission_key text NOT NULL REFERENCES permissions(key) ON DELETE CASCADE,
  PRIMARY KEY (role_id, permission_key)
);
