# 12 — Qualidade e Testes

## Pirâmide

### Unit
Maior volume.

Testar:
- domínio;
- validators;
- schema conversions;
- publishing builder;
- migrations puras;
- helpers de ISBN quando existirem.

### Integration
- SQLite repositories;
- filesystem;
- project open/save;
- export pipeline;
- Typst invocation;
- EPUB packaging.

### Component
React/Tiptap:
- toolbars;
- inspector;
- tree;
- dialogs;
- editor commands.

### E2E
Fluxos críticos:
1. criar projeto;
2. escrever capítulos;
3. reordenar;
4. configurar copyright;
5. fechar/reabrir;
6. preview;
7. exportar.

## Golden files

Manter fixtures de exportação.

Para EPUB:
- estrutura esperada;
- metadata;
- XHTML normalizado.

Para PDF:
- evitar depender apenas de comparação binária;
- testar número de páginas/metadados/texto extraível quando possível;
- snapshots visuais seletivos para layouts críticos.

## Property/fuzz tests

Úteis para:
- migrations;
- parser/import;
- sanitizer;
- tree operations;
- serialização.

## Crash tests

Simular encerramento:
- durante autosave;
- durante import de asset;
- durante migration;
- durante export;
- com disco cheio/erro de permissão quando viável.

## Performance budgets

Definir benchmarks para:
- projeto 100k palavras;
- 300k palavras;
- 500 documentos;
- centenas de assets.

Targets iniciais devem ser medidos no hardware de desenvolvimento e refinados. Evitar números fictícios antes dos benchmarks.

## Bugs bloqueantes de release

- perda de texto;
- corrupção de projeto;
- exportação silenciosamente incompleta;
- crash recorrente em fluxo principal;
- migration sem caminho de recuperação;
- EPUB inválido no fixture principal;
- copyright/metadados divergindo do preview sem warning.
