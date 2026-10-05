# 05 — Persistência e Formato de Projeto

## Filosofia

- local-first;
- transações para mudanças estruturais;
- dados recuperáveis;
- formato versionado;
- assets não armazenados como BLOB por padrão;
- nenhuma edição depende de conexão remota.

## Estrutura física

Durante desenvolvimento, usar projeto como diretório:

```text
MyBook.bookmaker/
├─ manifest.json
├─ project.db
├─ assets/
│  ├─ images/
│  ├─ covers/
│  ├─ ornaments/
│  └─ fonts/
├─ recovery/
├─ snapshots/
└─ cache/
```

No futuro, `.bookmaker` pode ser um pacote/arquivo compactado para distribuição. O editor deve trabalhar sobre uma workspace descompactada/segura, nunca alterar diretamente um ZIP a cada tecla.

## Criação da workspace

- o destino deve ser um novo diretório com extensão `.bookmaker`;
- a raiz existente nunca é sobrescrita;
- diretórios, banco e dados iniciais são preparados antes do manifest;
- `project_info` e os metadados mínimos são persistidos na mesma transação;
- `manifest.json` é gravado por arquivo temporário e rename;
- falhas removem somente a raiz recém-criada, evitando projetos parciais.

## Abertura da workspace

- a raiz deve ser um diretório `.bookmaker` real, não um symlink;
- o manifest é validado antes de abrir o banco;
- formatos futuros e versões mínimas incompatíveis falham de modo explícito;
- migrations são executadas antes da leitura dos dados;
- `projectId` e `formatVersion` devem coincidir entre manifest e SQLite.

## Projetos recentes

A lista de projetos recentes pertence aos dados locais da aplicação, não à workspace. Ela:

- é atualizada depois de criar ou abrir um projeto com sucesso;
- não bloqueia a criação/abertura caso sua própria gravação falhe;
- deduplica entradas por `projectId` e caminho;
- mantém projetos movidos ou ausentes marcados como indisponíveis;
- usa arquivo versionado com gravação temporária e backup de substituição.

## `manifest.json`

Informações mínimas para identificar e migrar o projeto antes de abrir o banco.

```json
{
  "format": "bookmaker-project",
  "formatVersion": 1,
  "projectId": "...",
  "createdBy": "BookMaker",
  "minimumAppVersion": "..."
}
```

## SQLite

Tabelas conceituais:

```text
projects
metadata
contributors
documents
document_content
assets
style_profiles
export_profiles
snapshots
schema_migrations
```

Evitar EAV genérico para o domínio principal.

## Conteúdo do editor

Guardar conteúdo textual separadamente da linha da árvore para:
- lazy loading;
- reduzir writes;
- permitir migrations específicas;
- facilitar snapshots.

Exemplo:

```text
documents
- id
- parent_id
- kind
- role
- title
- position
- status

document_content
- document_id
- schema_version
- json_content
- updated_at
```

## Autosave

Objetivo: usuário nunca precisar pensar no botão Save.

Estratégia:

1. mudanças do editor ficam em memória imediatamente;
2. debounce curto grava o documento alterado;
3. operações estruturais são transacionais e imediatas;
4. flush ocorre ao trocar de documento e antes de exportar;
5. janela fecha somente após flush ou apresenta erro explícito.

Não regravar o projeto completo a cada tecla.

O coordenador base:

- mantém somente o conteúdo mais recente de cada documento sujo;
- serializa saves do mesmo documento;
- não publica `saved` antes da conclusão da persistência;
- permite `flush(documentId)` e `flushAll()`;
- preserva conteúdo pendente após erro para retry explícito.

## Recovery

Separar **autosave** de **recovery**.

- autosave = estado normal persistido;
- recovery = proteção contra write interrompido/crash.

Manter journal/checkpoint de documentos sujos. Na inicialização, detectar sessão não encerrada e oferecer recuperação quando houver divergência.

## Snapshots

Snapshots são versões de segurança semânticas.

- manual na V1.1 ou posterior;
- automáticos antes de migration destrutiva;
- automáticos opcionalmente antes de export/import grande.

## Migrations

Cada mudança persistida requer migration incremental.

Regras:
- migrations nunca dependem de UI;
- migrations são aplicadas em ordem e cada etapa usa uma transação própria;
- migrations já aplicadas têm nome e checksum validados antes da abertura;
- backup/snapshot antes de migration não reversível;
- abrir projeto mais novo com app antigo deve falhar de modo legível;
- testes com fixtures reais de versões anteriores.

## Integridade de assets

- caminho sempre relativo ao projeto;
- nome físico pode usar AssetId para evitar colisões;
- manter nome original como metadata;
- deletar asset só quando não referenciado ou após confirmação;
- cache/thumbnails são descartáveis;
- assets nunca dependem de caminhos absolutos da máquina original.
