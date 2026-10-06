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

## Criação de documentos

Na V1, a criação produz um `FlowDocument` raiz, com status `active` e conteúdo semântico vazio. O título é normalizado e precisa conter texto. O papel opcional é validado contra `FlowDocumentRole`; quando omitido, usa `chapter`.

- a posição é anexada ao fim da raiz;
- a linha estrutural e o conteúdo inicial são gravados na mesma transação;
- falha em qualquer gravação não deixa documento parcial;
- a seção editorial (`frontMatter`, `manuscript` ou `backMatter`) é derivada do papel, sem coluna persistida adicional;
- criação direta de filhos e edição pertencem às tasks específicas seguintes.

## Renomeação de documentos

A renomeação normaliza espaços externos e rejeita títulos sem texto. O título e o timestamp estrutural do documento são atualizados junto ao timestamp do projeto em uma única transação; o conteúdo editorial e seu timestamp não são alterados.

## Movimentação de documentos

O comando de movimentação recebe o documento, o novo pai opcional e sua posição final entre os filhos do destino. A operação rejeita posições fora dos limites e ciclos na hierarquia, compacta a ordem da origem e abre espaço no destino em uma única transação.

- posições de irmãos permanecem contíguas e começam em zero;
- mover para a raiz usa pai `null`;
- documentos descendentes acompanham o documento movido;
- timestamps estruturais dos documentos afetados e do projeto são atualizados;
- conteúdo editorial e seu timestamp não são alterados.

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

A integração do editor carrega a árvore do projeto sem trazer todos os textos
para a memória, abre um `FlowDocument` por vez e valida seu schema na boundary
TypeScript. Alterações usam o coordenador com debounce e são persistidas em uma
transação que atualiza o conteúdo e o timestamp do projeto. A troca de documento
e o fechamento explícito do projeto aguardam o flush; se ele falhar, o conteúdo
continua no editor e a navegação é interrompida até o usuário tentar novamente.

## Recovery

Separar **autosave** de **recovery**.

- autosave = estado normal persistido;
- recovery = proteção contra write interrompido/crash.

Manter journal/checkpoint de documentos sujos. Na inicialização, detectar sessão não encerrada e oferecer recuperação quando houver divergência.

Fundação implementada:

- `recovery/session.json` identifica a sessão e impede ownership silencioso por outra sessão;
- checkpoints ficam separados por documento em `recovery/documents/`;
- cada checkpoint inclui versão, schema do conteúdo, timestamp e revisão persistida de referência;
- substituição usa arquivo temporário e backup;
- somente a sessão proprietária grava, limpa checkpoints ou encerra o marcador;
- IDs usados como nomes físicos são validados contra path traversal.

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

## Lixeira interna

Documentos movidos para a lixeira permanecem no SQLite com conteúdo e hierarquia preservados.

- `status = 'trashed'` identifica a entrada;
- `trashed_at` registra quando ocorreu a operação;
- `status_before_trash` permite restaurar o estado anterior;
- mover novamente para a lixeira é idempotente;
- conteúdo, posição e hierarquia são preservados ao mover e restaurar;
- o timestamp do projeto é atualizado na mesma transação da mudança;
- a árvore de documentos expõe operações para mover, listar e restaurar itens da lixeira;
- exclusão definitiva não integra o fluxo atual e exige uma task explícita.
