# BookMaker

BookMaker é uma engine desktop de produção editorial, local-first, criada para tornar simples o processo de estruturar, escrever, revisar, diagramar e exportar livros. A arquitetura nasce preparada para, no futuro, suportar HQs e mangás sem reescrever o núcleo da aplicação.

> Promessa da V1: **do primeiro capítulo ao livro pronto para publicar, sem sair do BookMaker.**

## Princípios do produto

1. **Simplicidade antes de quantidade de features.** A interface deve expor somente a complexidade necessária para a tarefa atual.
2. **Conteúdo separado de aparência.** O manuscrito guarda significado semântico; temas e estilos controlam apresentação.
3. **Local-first.** O usuário deve conseguir criar e exportar livros sem conta e sem conexão obrigatória.
4. **Não perder trabalho.** Autosave, recovery, snapshots e migrações seguras são requisitos de produto.
5. **Projeto editorial, não arquivo de texto.** Um projeto contém documentos, assets, estilos, metadados e perfis de exportação.
6. **Arquitetura preparada para múltiplos tipos de documento.** A V1 implementa `FlowDocument`; o núcleo já prevê `CanvasDocument` para HQ/mangá.
7. **Fluxo inspirado no Reedsy Studio.** Organização simples, escrita sem distrações, preview editorial e publicação integrada.

## Stack aprovada

- Desktop runtime: **Tauri 2.x**
- Frontend: **React + TypeScript**
- Editor rich text: **Tiptap / ProseMirror**
- Estado de UI/aplicação: **Zustand**
- Serviços nativos: **Rust**
- Persistência: **SQLite + filesystem**
- PDF/print: **Typst**
- EPUB: **renderer próprio EPUB 3**
- Testes frontend: **Vitest + Testing Library**
- E2E desktop: **Playwright**, complementado por testes de integração Tauri/Rust
- Testes Rust: `cargo test`
- Arquitetura: **Modular Monolith**

## Documentação

- [Visão e princípios](docs/00-VISION.md)
- [PRD — Product Requirements Document](docs/01-PRD.md)
- [Escopo da V1](docs/02-MVP-SCOPE.md)
- [Arquitetura](docs/03-ARCHITECTURE.md)
- [Modelo de domínio](docs/04-DOMAIN-MODEL.md)
- [Persistência e formato de projeto](docs/05-STORAGE.md)
- [Editor de texto](docs/06-EDITOR.md)
- [Publicação, PDF e EPUB](docs/07-PUBLISHING.md)
- [UX e design do produto](docs/08-UX.md)
- [Copyright, créditos e metadados editoriais](docs/09-COPYRIGHT-AND-METADATA.md)
- [Preparação para HQ e mangá](docs/10-COMICS-MANGA.md)
- [Roadmap](docs/11-ROADMAP.md)
- [Qualidade e testes](docs/12-QUALITY.md)
- [Segurança, privacidade e integridade](docs/13-SECURITY.md)
- [Workflow de desenvolvimento](docs/14-DEVELOPMENT.md)
- [Backlog inicial](docs/15-BACKLOG.md)
- [Requisitos não funcionais](docs/16-NON-FUNCTIONAL.md)
- [Riscos técnicos e de produto](docs/17-RISKS.md)
- [Definição de pronto](docs/18-DEFINITION-OF-DONE.md)
- [Especificação técnica inicial](docs/19-TECHNICAL-SPEC.md)
- [Draft do schema SQLite](docs/20-DATABASE-SCHEMA.md)
- [Arquitetura de informação da UI](docs/21-UI-INFORMATION-ARCHITECTURE.md)
- [Estratégia de release](docs/22-RELEASE-STRATEGY.md)
- [Benchmark de produto e mercado](docs/23-MARKET-BENCHMARK.md)
- [AGENTS.md](AGENTS.md)
- [ADRs](docs/adr/)

## Estrutura proposta do repositório

```text
bookmaker/
├─ apps/
│  └─ desktop/
│     ├─ src/                  # React UI
│     └─ src-tauri/            # Tauri + Rust
├─ packages/
│  ├─ domain/                  # regras de negócio puras
│  ├─ editor-core/             # schema e comandos de edição
│  ├─ publishing/              # IR + renderers EPUB/PDF
│  ├─ project-format/          # serialização e migrations
│  ├─ ui/                      # design system
│  └─ shared/                  # tipos utilitários realmente compartilhados
├─ docs/
├─ fixtures/
│  ├─ projects/
│  └─ exports/
├─ scripts/
├─ AGENTS.md
├─ package.json
├─ pnpm-workspace.yaml
└─ README.md
```

## Regra de ouro arquitetural

A UI nunca deve escrever diretamente no SQLite ou filesystem. Toda mutação significativa passa por uma camada de aplicação/domínio e produz uma operação explícita, validável e testável.

```text
UI -> Application Command -> Domain -> Repository/Service -> Persistence
```

A exportação também não consome diretamente o estado visual do editor:

```text
Project Domain -> Publishing IR -> Renderer -> PDF/EPUB
```

Isso impede que decisões de UI contaminem o formato do livro.
