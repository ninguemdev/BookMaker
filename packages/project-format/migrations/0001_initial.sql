CREATE TABLE project_info (
  id TEXT PRIMARY KEY NOT NULL,
  format_version INTEGER NOT NULL CHECK (format_version > 0),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
) STRICT;

CREATE TABLE project_metadata (
  project_id TEXT PRIMARY KEY NOT NULL,
  title TEXT NOT NULL,
  subtitle TEXT,
  language TEXT NOT NULL,
  description TEXT,
  edition TEXT,
  publisher TEXT,
  publication_date TEXT,
  rights_statement_mode TEXT NOT NULL CHECK (
    rights_statement_mode IN ('generated', 'custom')
  ),
  custom_rights_statement TEXT,
  disclaimer TEXT,
  credits_note TEXT,
  FOREIGN KEY (project_id) REFERENCES project_info (id) ON DELETE CASCADE
) STRICT;

CREATE TABLE rights_holders (
  id TEXT PRIMARY KEY NOT NULL,
  project_id TEXT NOT NULL,
  name TEXT NOT NULL,
  position INTEGER NOT NULL CHECK (position >= 0),
  FOREIGN KEY (project_id) REFERENCES project_info (id) ON DELETE CASCADE
) STRICT;

CREATE TABLE contributors (
  id TEXT PRIMARY KEY NOT NULL,
  project_id TEXT NOT NULL,
  name TEXT NOT NULL,
  sort_name TEXT,
  position INTEGER NOT NULL CHECK (position >= 0),
  FOREIGN KEY (project_id) REFERENCES project_info (id) ON DELETE CASCADE
) STRICT;

CREATE TABLE contributor_roles (
  contributor_id TEXT NOT NULL,
  role TEXT NOT NULL,
  PRIMARY KEY (contributor_id, role),
  FOREIGN KEY (contributor_id) REFERENCES contributors (id) ON DELETE CASCADE
) STRICT;

CREATE TABLE identifiers (
  id TEXT PRIMARY KEY NOT NULL,
  project_id TEXT NOT NULL,
  type TEXT NOT NULL,
  value TEXT NOT NULL,
  label TEXT,
  FOREIGN KEY (project_id) REFERENCES project_info (id) ON DELETE CASCADE
) STRICT;

CREATE TABLE documents (
  id TEXT PRIMARY KEY NOT NULL,
  parent_id TEXT,
  kind TEXT NOT NULL,
  role TEXT,
  title TEXT NOT NULL,
  position INTEGER NOT NULL CHECK (position >= 0),
  status TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY (parent_id) REFERENCES documents (id) ON DELETE RESTRICT
) STRICT;

CREATE INDEX documents_parent_position_idx
  ON documents (parent_id, position);

CREATE TABLE document_content (
  document_id TEXT PRIMARY KEY NOT NULL,
  schema_version INTEGER NOT NULL CHECK (schema_version > 0),
  json_content TEXT NOT NULL CHECK (json_valid(json_content)),
  updated_at TEXT NOT NULL,
  FOREIGN KEY (document_id) REFERENCES documents (id) ON DELETE CASCADE
) STRICT;

CREATE TABLE assets (
  id TEXT PRIMARY KEY NOT NULL,
  type TEXT NOT NULL,
  relative_path TEXT NOT NULL UNIQUE,
  original_name TEXT NOT NULL,
  mime_type TEXT NOT NULL,
  byte_size INTEGER CHECK (byte_size IS NULL OR byte_size >= 0),
  hash TEXT,
  metadata_json TEXT NOT NULL CHECK (json_valid(metadata_json)),
  created_at TEXT NOT NULL
) STRICT;

CREATE TABLE style_profiles (
  id TEXT PRIMARY KEY NOT NULL,
  name TEXT NOT NULL,
  schema_version INTEGER NOT NULL CHECK (schema_version > 0),
  style_json TEXT NOT NULL CHECK (json_valid(style_json)),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
) STRICT;

CREATE TABLE export_profiles (
  id TEXT PRIMARY KEY NOT NULL,
  name TEXT NOT NULL,
  format TEXT NOT NULL CHECK (format IN ('pdf', 'epub')),
  style_profile_id TEXT NOT NULL,
  options_json TEXT NOT NULL CHECK (json_valid(options_json)),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY (style_profile_id) REFERENCES style_profiles (id) ON DELETE RESTRICT
) STRICT;

CREATE TABLE project_settings (
  key TEXT PRIMARY KEY NOT NULL,
  value_json TEXT NOT NULL CHECK (json_valid(value_json))
) STRICT;

CREATE TABLE schema_migrations (
  version INTEGER PRIMARY KEY NOT NULL CHECK (version > 0),
  name TEXT NOT NULL,
  applied_at TEXT NOT NULL,
  checksum TEXT
) STRICT;
