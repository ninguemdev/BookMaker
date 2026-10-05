# ADR 0003 — Local-first com SQLite + filesystem

Status: Accepted

## Decisão

Projetos funcionam offline e usam SQLite para dados estruturados e filesystem para assets.

## Motivo

Manuscritos são dados críticos e devem permanecer acessíveis sem serviço remoto. SQLite fornece transações e migrations; assets permanecem portáveis sem grandes BLOBs no banco.
