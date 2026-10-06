# 06 — Editor de Texto

## Tecnologia

Tiptap sobre ProseMirror.

## Objetivo

Construir um editor de manuscrito, não um clone de Word.

Quanto menor e mais controlado o schema, mais previsíveis ficam exportação, temas e migrações.

## Schema inicial

O schema persistido atual é a versão `1`, implementada em
`packages/editor-core`. Nesta versão, headings aceitam somente os níveis 1, 2 e
3. O documento raiz aceita zero ou mais blocos para permanecer compatível com o
conteúdo vazio criado junto de um novo `FlowDocument`.

Conteúdo vindo da persistência deve passar por `parseEditorContent`. A boundary
rejeita nodes, marks, campos e atributos desconhecidos, estruturas inválidas e
URLs inseguras em links, sem descartar dados silenciosamente.

Nodes:
- `doc`;
- `paragraph`;
- `heading` limitado aos níveis necessários;
- `blockquote`;
- `bulletList`;
- `orderedList`;
- `listItem`;
- `hardBreak`;
- `horizontalRule` somente se semanticamente diferente de `sceneBreak` (não
  integra a versão 1);
- `sceneBreak` custom (task própria);
- `image` custom (task própria);
- `footnote` futuramente;
- `text`.

Marks:
- bold;
- italic;
- underline;
- strike opcional (não integra a versão 1);
- link;
- emphasis custom somente quando necessário.

## O que não permitir por padrão

- `<span style="font-size: 17.3px">` arbitrário;
- cores livres em cada palavra;
- fonte arbitrária em cada seleção;
- margin/padding inline;
- HTML não sanitizado.

Esses recursos transformariam o documento semântico em DTP genérico e dificultariam EPUB/temas.

## Scene break

`sceneBreak` deve ser node próprio.

Motivo: o tema decide se será espaço, asteriscos, ornamento ou outro símbolo.

## Imagens

A imagem no documento referencia `AssetId`, não caminho absoluto.

Atributos possíveis:
- alt text;
- caption;
- alignment semântico;
- width preset/percentual limitado;
- decorative flag.

## Paste

Prioridades:
1. preservar texto;
2. preservar semântica simples;
3. remover estilos externos incompatíveis;
4. converter headings/listas quando confiável;
5. nunca importar scripts/HTML inseguro.

Oferecer futuramente:
- colar mantendo formatação compatível;
- colar como texto puro.

## Undo/redo

Existem dois níveis:

### Conteúdo
Histórico ProseMirror por documento.

### Estrutural
Mover/renomear/criar/excluir documento deve usar command stack própria quando houver UX para undo estrutural.

Não tentar unificar os dois historicamente na primeira implementação.

## Performance

Projetos longos não devem renderizar todos os capítulos num único ProseMirror EditorView.

Estratégia V1:
- um documento editável por vez;
- árvore e busca indexam projeto inteiro;
- troca de capítulo faz flush e carrega o próximo;
- opcionalmente modo leitura contínua no futuro, virtualizado.

## Busca

Duas modalidades:
- documento atual: rápida/local;
- projeto: índice/consulta de todos os documentos.

Resultado inclui:
- título do documento;
- trecho;
- posição;
- navegação direta.

## Contagem de palavras

Definir regras por idioma depois; na V1 usar algoritmo consistente e documentado, não confiar em `split(' ')`.

## Acessibilidade

Editor deve manter:
- navegação por teclado;
- foco previsível;
- labels acessíveis;
- contraste mínimo;
- zoom da interface;
- compatibilidade com IME e caracteres Unicode.
