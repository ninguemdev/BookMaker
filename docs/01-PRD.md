# 01 — Product Requirements Document

## Objetivo

Construir uma aplicação desktop local-first para criação de livros com fluxo inspirado em ferramentas como Reedsy Studio: estrutura clara, editor focado no conteúdo, formatação editorial guiada e exportação integrada.

## Objetivos da V1

- criar, abrir, salvar e recuperar projetos;
- organizar manuscrito em estrutura hierárquica;
- oferecer editor rich-text semântico;
- permitir front matter e back matter;
- permitir configuração de copyright/créditos;
- separar conteúdo de estilo visual;
- oferecer temas editoriais;
- preview paginado suficientemente fiel;
- exportar EPUB 3;
- exportar PDF para leitura/impressão;
- funcionar offline;
- proteger o trabalho do usuário contra crash/corrupção comum.

## Fora do escopo da V1

- colaboração em tempo real;
- sincronização em nuvem;
- marketplace de temas;
- edição gráfica de capas;
- ferramentas de desenho;
- editor completo de HQ/mangá;
- track changes avançado;
- IA integrada;
- publicação direta em Amazon/Kobo;
- DRM;
- registro governamental de copyright;
- gerenciamento financeiro/royalties.

## Personas

### Autor independente
Quer escrever e publicar sem conhecer ferramentas de diagramação.

### Autor técnico
Quer organização, notas, imagens, capítulos e exportação previsível.

### Pequena editora/revisor
Quer abrir um projeto, revisar estrutura, metadados e gerar artefatos consistentes.

## Fluxos principais

### Novo livro
`Home -> Novo projeto -> Metadados básicos -> Editor -> Estrutura -> Tema -> Preview -> Exportar`

### Retomar livro
`Home -> Recentes -> Projeto -> Última posição -> Continuar edição`

### Preparar publicação
`Projeto -> Dados editoriais -> Copyright -> Tema -> Preview -> Validação -> Exportar`

## Requisitos funcionais

### Projeto
- FR-PROJ-001 — criar projeto;
- FR-PROJ-002 — abrir projeto existente;
- FR-PROJ-003 — autosave;
- FR-PROJ-004 — salvar cópia/exportar projeto;
- FR-PROJ-005 — recuperar sessão após crash;
- FR-PROJ-006 — migrar projeto de versões anteriores;
- FR-PROJ-007 — manter lista de projetos recentes sem tornar essa lista uma dependência do arquivo real.

### Manuscrito
- FR-DOC-001 — criar parte/capítulo/seção;
- FR-DOC-002 — renomear;
- FR-DOC-003 — reordenar via drag-and-drop;
- FR-DOC-004 — arquivar/excluir com lixeira interna;
- FR-DOC-005 — pesquisar no projeto;
- FR-DOC-006 — contar palavras/caracteres;
- FR-DOC-007 — navegar sem carregar todo o manuscrito visualmente de uma vez.

### Editor
- FR-ED-001 — parágrafo;
- FR-ED-002 — títulos suportados pelo schema;
- FR-ED-003 — negrito, itálico, sublinhado quando editorialmente permitido;
- FR-ED-004 — blockquote;
- FR-ED-005 — listas;
- FR-ED-006 — hyperlinks;
- FR-ED-007 — scene break;
- FR-ED-008 — imagem inline/bloco;
- FR-ED-009 — notas de rodapé ou notas finais, se viável no milestone editorial;
- FR-ED-010 — undo/redo robusto;
- FR-ED-011 — copy/paste com sanitização;
- FR-ED-012 — atalhos de teclado;
- FR-ED-013 — focus mode.

### Estrutura editorial
- FR-BOOK-001 — capa como asset/metadata, sem editor gráfico na V1;
- FR-BOOK-002 — folha de rosto;
- FR-BOOK-003 — página de copyright;
- FR-BOOK-004 — dedicatória;
- FR-BOOK-005 — epígrafe;
- FR-BOOK-006 — sumário automático;
- FR-BOOK-007 — prefácio/introdução;
- FR-BOOK-008 — apêndice;
- FR-BOOK-009 — agradecimentos;
- FR-BOOK-010 — sobre o autor.

### Copyright e créditos
- FR-RIGHTS-001 — informar titular dos direitos;
- FR-RIGHTS-002 — ano de copyright;
- FR-RIGHTS-003 — texto de direitos customizável;
- FR-RIGHTS-004 — edição/versão;
- FR-RIGHTS-005 — ISBN opcional;
- FR-RIGHTS-006 — créditos de edição, revisão, capa, ilustração, tradução etc.;
- FR-RIGHTS-007 — aviso legal customizável;
- FR-RIGHTS-008 — licença opcional, incluindo texto livre;
- FR-RIGHTS-009 — gerar página de copyright a partir dos dados;
- FR-RIGHTS-010 — permitir override manual sem destruir os metadados estruturados.

### Estilo e layout
- FR-STY-001 — escolher tema;
- FR-STY-002 — tamanho de página;
- FR-STY-003 — margens;
- FR-STY-004 — fonte do corpo e títulos dentro das fontes suportadas;
- FR-STY-005 — primeira página de capítulo;
- FR-STY-006 — headers/footers;
- FR-STY-007 — numeração;
- FR-STY-008 — ornaments/scene breaks;
- FR-STY-009 — preview sem alterar o conteúdo original.

### Exportação
- FR-EXP-001 — EPUB 3;
- FR-EXP-002 — PDF;
- FR-EXP-003 — validar dados obrigatórios antes da exportação;
- FR-EXP-004 — relatório de warnings não bloqueantes;
- FR-EXP-005 — perfil de exportação persistente;
- FR-EXP-006 — exportação determinística dada a mesma versão do projeto/renderizador.

## Métricas iniciais

- tempo para criar projeto e escrever primeiro texto;
- crashes por 100 sessões;
- projetos recuperados com sucesso após encerramento inesperado;
- taxa de exportações concluídas;
- erros de EPUB detectados pelo validador;
- tempo de abertura para projetos de 100k/300k palavras;
- latência percebida de digitação;
- percentual de usuários que conclui o fluxo de publicação sem documentação externa.
