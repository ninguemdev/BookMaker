ALTER TABLE documents ADD COLUMN trashed_at TEXT;
ALTER TABLE documents ADD COLUMN status_before_trash TEXT;

CREATE INDEX documents_status_idx ON documents (status);
