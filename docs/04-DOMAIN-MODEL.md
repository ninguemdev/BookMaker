# 04 — Modelo de Domínio

## Entidade raiz

```ts
interface Project {
  id: ProjectId;
  schemaVersion: number;
  metadata: ProjectMetadata;
  book: BookDefinition;
  documentTree: DocumentTree;
  assets: AssetIndex;
  styleProfile: StyleProfileId;
  exportProfiles: ExportProfile[];
}
```

## Document

```ts
type Document = FlowDocument | CanvasDocument;

interface BaseDocument {
  id: DocumentId;
  parentId: DocumentId | null;
  title: string;
  order: number;
  status: DocumentStatus;
  createdAt: string;
  updatedAt: string;
}

interface FlowDocument extends BaseDocument {
  kind: 'flow';
  role: FlowDocumentRole;
  contentRef: ContentRef;
}

interface CanvasDocument extends BaseDocument {
  kind: 'canvas';
  canvasRef: CanvasRef;
}
```

Na V1, `CanvasDocument` pode existir apenas como contrato/reserva de arquitetura e não ser criável pela UI.

## Papéis de FlowDocument

- `part`;
- `chapter`;
- `section`;
- `titlePage`;
- `copyrightPage`;
- `dedication`;
- `epigraph`;
- `toc`;
- `preface`;
- `introduction`;
- `appendix`;
- `acknowledgements`;
- `aboutAuthor`;
- `customFrontMatter`;
- `customBackMatter`.

## Conteúdo semântico

O editor persiste JSON ProseMirror validado por schema versionado.

O conteúdo não deve persistir CSS arbitrário.

Exemplo conceitual:

```json
{
  "type": "doc",
  "content": [
    {"type": "paragraph", "content": [{"type": "text", "text": "Texto"}]},
    {"type": "sceneBreak"}
  ]
}
```

## Metadados

```ts
interface ProjectMetadata {
  title: string;
  subtitle?: string;
  language: string;
  description?: string;
  edition?: string;
  publisher?: string;
  publicationDate?: string;
  identifiers: Identifier[];
  contributors: Contributor[];
  rights: RightsMetadata;
}
```

## Contribuidores

```ts
type ContributorRole =
  | 'author'
  | 'editor'
  | 'proofreader'
  | 'translator'
  | 'illustrator'
  | 'coverDesigner'
  | 'letterer'
  | 'publisher'
  | 'other';
```

## Copyright / rights

```ts
interface RightsMetadata {
  copyrightYear?: number;
  holders: RightsHolder[];
  statementMode: 'generated' | 'custom';
  customStatement?: string;
  license?: LicenseMetadata;
  disclaimer?: string;
  creditsNote?: string;
}
```

O texto gerado é apresentação derivada dos dados. O usuário pode escolher texto customizado.

## Asset

```ts
interface Asset {
  id: AssetId;
  type: 'image' | 'font' | 'ornament' | 'cover' | 'other';
  relativePath: string;
  mimeType: string;
  originalName: string;
  hash?: string;
  metadata: Record<string, unknown>;
}
```

## Style Profile

Não armazenar estilo diretamente em cada parágrafo quando ele for global.

```ts
interface StyleProfile {
  page: PageStyle;
  typography: TypographyStyle;
  chapter: ChapterStyle;
  paragraph: ParagraphStyle;
  sceneBreak: SceneBreakStyle;
  headerFooter: HeaderFooterStyle;
}
```

## Publishing IR

Modelo intermediário para impedir acoplamento entre domínio e mecanismos de exportação.

```ts
type PublicationNode =
  | PublicationSection
  | PublicationHeading
  | PublicationParagraph
  | PublicationImage
  | PublicationSceneBreak
  | PublicationFootnote
  | PublicationPageBreak;
```

A IR deve conter somente o necessário para publicação.
