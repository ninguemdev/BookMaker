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
- `sceneBreak` custom;
- `image` custom;
- `footnote` futuramente;
- `text`.

Marks:
- bold;
- italic;
- underline;
- strike opcional (não integra a versão 1);
- link;
- emphasis custom somente quando necessário.

## Toolbar inicial

A toolbar expõe somente os comandos presentes no schema atual:

- parágrafo, headings de nível 1 a 3, blockquote e scene break;
- bold, italic, underline e link;
- listas com e sem numeração;
- undo e redo.

Os controles informam o estado ativo com `aria-pressed`, ficam indisponíveis sem
um editor e exibem no tooltip os atalhos implementados. A edição de link valida
o endereço antes de aplicar a mark e permite removê-la sem alterar o texto.

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

Na versão 1, o node é atômico, não possui atributos ou conteúdo e usa
`<hr data-scene-break>` apenas como representação no editor/clipboard. A
aparência de publicação não é derivada desse HTML. O comando
`insertSceneBreak` mantém o cursor em um parágrafo editável após a inserção.

## Imagens

A imagem no documento referencia `AssetId`, não caminho absoluto.

Atributos da versão 1:
- `assetId` obrigatório;
- `alt` textual;
- `caption` opcional;
- `alignment`: `left`, `center` ou `right`;
- `width`: `small`, `medium`, `large` ou `full`;
- `decorative` booleano; quando verdadeiro, `alt` deve ficar vazio.

O HTML do editor/clipboard usa um marcador `figure[data-bookmaker-image]` com
metadata semântica e nunca persiste `src` ou caminho de filesystem. A resolução
visual do `AssetId` pertence à integração com a Asset Library. O comando
`insertImage` recebe um `AssetId`, insere o node como uma operação de histórico
e mantém um parágrafo editável depois dele.

## Paste

O paste sanitizer base é aplicado antes do parser do ProseMirror. Ele usa uma
allowlist compatível com o schema atual e:

- preserva texto Unicode, parágrafos, headings 1–3, blockquote, listas e marks
  simples;
- remove estilos, classes, event handlers e atributos externos;
- remove elementos executáveis ou incorporados, como script, iframe e SVG;
- mantém links somente quando a URL é segura;
- converte imagens externas em seu alt/caption textual, sem importar `src`;
- preserva `sceneBreak` e `image` apenas quando possuem os marcadores internos
  e atributos válidos do BookMaker.

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
