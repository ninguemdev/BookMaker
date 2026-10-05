# 10 — Preparação para HQ e Mangá

## Objetivo

Evitar que decisões da V1 tornem impossível adicionar edição espacial posteriormente.

Não implementar o editor de HQ agora.

## Conceito

Livros usam majoritariamente fluxo:

```text
text -> line -> page
```

HQ/mangá usa composição espacial:

```text
page -> panels -> objects/layers
```

Por isso existem dois tipos de documento:

```ts
type Document = FlowDocument | CanvasDocument;
```

## CanvasDocument futuro

```ts
interface CanvasDocument extends BaseDocument {
  kind: 'canvas';
  size: CanvasSize;
  readingDirection: 'ltr' | 'rtl';
  layers: Layer[];
}
```

Objetos possíveis:
- panel;
- image;
- text;
- speechBalloon;
- captionBox;
- shape;
- effect;
- guide.

## Página e spread

O domínio futuro deve suportar:
- página individual;
- spreads;
- bleed;
- safe area;
- trim;
- presets;
- orientação de leitura.

## Assets compartilhados

A mesma Asset Library usada por livros deve suportar HQ:
- imagens;
- fontes;
- ornaments;
- brushes futuramente, se fizer sentido;
- templates.

## Roteiro -> página

Possível evolução:

```text
FlowDocument(role='script')
        ↓
Story/Scene Model
        ↓
CanvasDocument pages
```

Não assumir que a conversão será automática. O importante é compartilhar projeto, metadata, assets e estrutura.

## Interface futura

O editor Canvas provavelmente exigirá:
- pan/zoom;
- seleção múltipla;
- snapping;
- resize/rotate;
- layers;
- undo/redo espacial;
- renderização acelerada;
- thumbnails;
- text layout em balões.

Tecnologias candidatas devem ser avaliadas quando chegar o milestone, sem amarrar o domínio atual a uma biblioteca específica de canvas.

## Regras atuais para preservar extensibilidade

1. `Project` não significa `Book` exclusivamente.
2. `Document` é discriminated union.
3. assets são independentes do editor rich text.
4. metadata de publicação não vive dentro de capítulos.
5. publishing usa IR, permitindo renderizadores diferentes.
6. IDs não dependem da posição na árvore.
7. UI não deve assumir que todo documento abre em Tiptap.
