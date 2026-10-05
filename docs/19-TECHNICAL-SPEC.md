# 19 — Especificação Técnica Inicial

## Estado e ownership

O projeto terá três classes principais de estado.

### Estado persistente de domínio
Fonte de verdade: repositórios/SQLite.

Exemplos:
- árvore de documentos;
- metadata;
- rights;
- style profiles;
- export profiles;
- conteúdo salvo.

### Estado de sessão
Fonte de verdade: application/store.

Exemplos:
- projeto aberto;
- documento selecionado;
- painéis abertos;
- filtros;
- status de autosave;
- tarefas em execução.

### Estado efêmero de componente
Fonte de verdade: React/Tiptap local.

Exemplos:
- menu aberto;
- hover;
- campo de diálogo ainda não submetido;
- seleção do editor.

Não sincronizar tudo automaticamente com Zustand.

## Services / ports iniciais

```ts
interface ProjectRepository {
  create(input: CreateProjectInput): Promise<Project>;
  open(path: string): Promise<Project>;
  saveMetadata(projectId: ProjectId, metadata: ProjectMetadata): Promise<void>;
  close(projectId: ProjectId): Promise<void>;
}

interface DocumentRepository {
  getTree(projectId: ProjectId): Promise<DocumentNode[]>;
  getContent(id: DocumentId): Promise<PersistedEditorContent>;
  saveContent(id: DocumentId, content: PersistedEditorContent): Promise<void>;
  create(input: CreateDocumentInput): Promise<Document>;
  move(command: MoveDocumentCommand): Promise<void>;
}

interface AssetService {
  importFile(projectId: ProjectId, sourcePath: string): Promise<Asset>;
  remove(assetId: AssetId): Promise<void>;
  resolve(assetId: AssetId): Promise<ResolvedAsset>;
}

interface PublishingService {
  validate(projectId: ProjectId, profileId: ExportProfileId): Promise<PublishDiagnostics>;
  export(request: ExportRequest): Promise<ExportResult>;
}
```

## Application use cases

- CreateProject
- OpenProject
- CloseProject
- CreateDocument
- RenameDocument
- MoveDocument
- TrashDocument
- RestoreDocument
- SaveDocumentContent
- ImportAsset
- UpdateProjectMetadata
- UpdateRightsMetadata
- UpdateStyleProfile
- BuildPublication
- ValidatePublication
- ExportPublication
- CreateSnapshot
- RestoreSnapshot

## Validation boundary

Dados vindos de:
- SQLite;
- manifest;
- clipboard;
- arquivos importados;
- IPC;

são validados antes de entrar no domínio tipado.

Escolher uma estratégia consistente de runtime validation no TypeScript (ex.: schema library) durante bootstrap, registrada em ADR.

## Concurrency

V1 assume **um processo BookMaker editando um projeto por vez**.

Ao abrir projeto:
- criar lock de sessão;
- detectar lock antigo;
- distinguir crash de projeto aberto em outra instância quando possível;
- não permitir duas instâncias escreverem simultaneamente sem mecanismo de coordenação.

## Background work

Operações pesadas:
- exportação;
- geração de thumbnails;
- indexação ampla;
- importação pesada;

devem rodar fora do caminho síncrono de renderização da UI.

## Cancelamento

Long-running jobs devem possuir CancellationToken/AbortSignal conceitual quando for seguro cancelar.

Exportação cancelada não pode deixar o arquivo final aparentando válido.

Estratégia:
1. escrever em arquivo temporário;
2. concluir/validar;
3. rename atômico para destino final quando possível.

## File paths

Frontend não manipula paths internos livremente. O native boundary normaliza/canonicaliza caminhos e aplica regras da raiz do projeto.

## Feature architecture

```text
features/
  project/
  manuscript/
  editor/
  metadata/
  rights/
  styles/
  preview/
  publishing/
  settings/
```

Cada feature pode conter UI e adapters, mas regras compartilhadas devem permanecer nos packages apropriados.
