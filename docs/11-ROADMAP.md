# 11 — Roadmap

Roadmap baseado em dependências, não em datas arbitrárias.

## M0 — Bootstrap técnico

Entregáveis:
- monorepo;
- Tauri + React funcionando;
- lint/format/test;
- CI;
- design tokens mínimos;
- ADRs iniciais;
- logging;
- error boundary.

Critério: app abre e pipeline verde em Windows/macOS/Linux onde suportado pela CI.

## M1 — Project Core

- criar/abrir projeto;
- manifest;
- SQLite;
- migrations;
- document tree;
- autosave base;
- crash/recovery foundation;
- recent projects.

Critério: projeto sobrevive a múltiplas sessões e reordenação.

## M2 — Writing Core

- Tiptap schema;
- capítulos;
- editor;
- undo/redo;
- paste sanitizer;
- scene break;
- word count;
- busca;
- focus mode.

Critério: manuscrito longo pode ser escrito com latência aceitável.

## M3 — Editorial Structure

- front/back matter;
- title page;
- copyright page;
- contributors;
- metadata;
- image assets;
- TOC model.

Critério: projeto contém todos os dados necessários para publicação básica.

## M4 — Styling & Preview

- StyleProfile;
- temas internos;
- preview;
- page size/margins;
- chapter style;
- headers/footers;
- page numbering.

Critério: usuário entende visualmente o resultado final antes de exportar.

## M5 — Publishing

- Publishing IR;
- EPUB 3;
- PDF/Typst;
- validações;
- export profiles;
- export report.

Critério: livro real de teste gera EPUB e PDF válidos.

## M6 — Hardening / Beta

- testes de stress;
- recovery;
- migration fixtures;
- acessibilidade;
- performance;
- telemetry opcional e privacy-safe somente se decidida;
- packaging/updater;
- documentação do usuário.

## Pós-V1

### V1.1
- import/export DOCX;
- snapshots;
- comentários básicos;
- mais temas;
- notas de rodapé refinadas.

### V1.5
- custom themes;
- revisão avançada;
- plugins/extensões avaliadas;
- sync opcional projetado separadamente.

### V2 — Comic Foundation
- CanvasDocument;
- page manager;
- layers;
- panels;
- images;
- text objects;
- balloons;
- reading direction;
- fixed-layout export experimental.

### V2.x — Manga/HQ Production
- templates;
- spreads;
- bleed/safe zones;
- lettering tools;
- asset reuse;
- storyboard/script workflow;
- export de impressão/fixed layout.
