# 20 — Schema SQLite

Este documento apresenta o schema persistido atual. O SQL executável e seus testes são a fonte definitiva:

- `packages/project-format/migrations/0001_initial.sql`;
- `packages/project-format/migrations/0002_internal_trash.sql`;
- `apps/desktop/src-tauri/src/persistence/migrations.rs`;
- `apps/desktop/src-tauri/tests/sqlite_schema_v1.rs`.

## `project_info`

```text
id TEXT PRIMARY KEY
format_version INTEGER NOT NULL
created_at TEXT NOT NULL
updated_at TEXT NOT NULL
```

Uma workspace contém normalmente um único projeto; manter ID explícito simplifica consistência e futuras ferramentas.

## `project_metadata`

```text
project_id TEXT PRIMARY KEY
title TEXT NOT NULL
subtitle TEXT
language TEXT NOT NULL
description TEXT
edition TEXT
publisher TEXT
publication_date TEXT
rights_statement_mode TEXT NOT NULL
custom_rights_statement TEXT
disclaimer TEXT
credits_note TEXT
```

## `rights_holders`

```text
id TEXT PRIMARY KEY
project_id TEXT NOT NULL
name TEXT NOT NULL
position INTEGER NOT NULL
```

## `contributors`

```text
id TEXT PRIMARY KEY
project_id TEXT NOT NULL
name TEXT NOT NULL
sort_name TEXT
position INTEGER NOT NULL
```

## `contributor_roles`

```text
contributor_id TEXT NOT NULL
role TEXT NOT NULL
PRIMARY KEY(contributor_id, role)
```

## `identifiers`

```text
id TEXT PRIMARY KEY
project_id TEXT NOT NULL
type TEXT NOT NULL
value TEXT NOT NULL
label TEXT
```

## `documents`

```text
id TEXT PRIMARY KEY
parent_id TEXT
kind TEXT NOT NULL
role TEXT
title TEXT NOT NULL
position INTEGER NOT NULL
status TEXT NOT NULL
created_at TEXT NOT NULL
updated_at TEXT NOT NULL
trashed_at TEXT
status_before_trash TEXT
```

Índices:
- `(parent_id, position)`;
- `status`.

## `document_content`

```text
document_id TEXT PRIMARY KEY
schema_version INTEGER NOT NULL
json_content TEXT NOT NULL
updated_at TEXT NOT NULL
```

## `assets`

```text
id TEXT PRIMARY KEY
type TEXT NOT NULL
relative_path TEXT NOT NULL UNIQUE
original_name TEXT NOT NULL
mime_type TEXT NOT NULL
byte_size INTEGER
hash TEXT
metadata_json TEXT NOT NULL
created_at TEXT NOT NULL
```

## `style_profiles`

```text
id TEXT PRIMARY KEY
name TEXT NOT NULL
schema_version INTEGER NOT NULL
style_json TEXT NOT NULL
created_at TEXT NOT NULL
updated_at TEXT NOT NULL
```

JSON é aceitável aqui porque style profile é um agregado versionado cuja estrutura muda em conjunto.

## `export_profiles`

```text
id TEXT PRIMARY KEY
name TEXT NOT NULL
format TEXT NOT NULL
style_profile_id TEXT NOT NULL
options_json TEXT NOT NULL
created_at TEXT NOT NULL
updated_at TEXT NOT NULL
```

## `project_settings`

Key/value somente para preferências realmente auxiliares, não para entidades de domínio.

```text
key TEXT PRIMARY KEY
value_json TEXT NOT NULL
```

## `schema_migrations`

```text
version INTEGER PRIMARY KEY
name TEXT NOT NULL
applied_at TEXT NOT NULL
checksum TEXT
```

## Regras

- foreign keys habilitadas;
- operações estruturais dentro de transaction;
- timestamps ISO 8601 UTC;
- IDs gerados pela aplicação;
- não armazenar cache crítico no banco;
- usar WAL se os testes de plataforma confirmarem benefício e segurança para a estratégia de projeto/backup.
