# 14 — Workflow de Desenvolvimento

## Monorepo

Sugestão: pnpm workspace.

```text
apps/desktop
packages/domain
packages/editor-core
packages/publishing
packages/project-format
packages/ui
packages/shared
```

Rust permanece em `apps/desktop/src-tauri` até surgir motivo real para crates separados.

## Branching

Preferir trunk-based leve:
- `main` sempre saudável;
- branches curtas por feature/fix;
- PR mesmo em equipe pequena quando útil;
- feature flags para trabalho incompleto.

## Commits

Conventional Commits é aceitável:
- `feat:`;
- `fix:`;
- `refactor:`;
- `docs:`;
- `test:`;
- `build:`.

## Tooling sugerido

Frontend:
- TypeScript strict;
- ESLint;
- Prettier;
- Vitest;
- Testing Library.

Rust:
- rustfmt;
- clippy;
- cargo test.

Git hooks são opcionais; CI é autoridade final.

## TypeScript

Regras:
- `strict: true`;
- evitar `any`;
- validar dados em boundaries;
- branded types para IDs quando benefício superar complexidade;
- funções de domínio preferencialmente puras.

## React

- componentes pequenos por responsabilidade;
- lógica de negócio fora dos componentes;
- Zustand para estado de aplicação/UI compartilhado;
- estado local continua local;
- não criar mega-store global contendo editor inteiro.

## Rust

Usar quando houver benefício nativo:
- filesystem;
- dialogs/OS;
- processos externos controlados;
- packaging;
- performance específica;
- integração com SQLite se a arquitetura decidir manter persistência no lado nativo.

Não duplicar domínio inteiro em Rust e TypeScript sem necessidade.

## Errors

Erros devem conter:
- código estável;
- mensagem técnica/log;
- mensagem amigável para UI;
- causa quando disponível.

Nunca mostrar stack trace cru como única mensagem ao usuário.

## Logging

Categorias:
- project;
- persistence;
- editor;
- publishing;
- migration;
- native.

Logs nunca devem conter texto integral do manuscrito por padrão.

## ADR

Toda decisão difícil e duradoura deve ter Architecture Decision Record.

Exemplos:
- escolha Tauri;
- formato do projeto;
- local da persistência SQLite;
- Typst;
- schema Tiptap;
- estratégia de package `.bookmaker`.
