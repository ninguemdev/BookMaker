# 21 — Arquitetura de Informação da UI

## Áreas principais

### Home
- Novo livro
- Abrir projeto
- Recentes
- Templates futuramente
- Preferências

### Workspace
Rotas/estados conceituais:

```text
/project/:id/write/:documentId
/project/:id/metadata
/project/:id/design
/project/:id/preview
/project/:id/export
/project/:id/settings
```

A implementação desktop não precisa necessariamente expor URL real; a estrutura serve para separar responsabilidades.

## Write

A integração visual monta no `App` a abertura de uma pasta `.bookmaker`, a top
bar, a árvore persistida do manuscrito, o editor Tiptap, a toolbar, o status de
autosave, as buscas, o inspector e o modo foco. O conteúdo é carregado sob
demanda e a UI só apresenta o estado `Salvo` depois que a persistência nativa
termina com sucesso.

### Sidebar esquerda
- Front Matter
- Manuscript
- Back Matter
- botão adicionar
- busca/filtro

A árvore usa semântica ARIA e um único item no ciclo de Tab. Com o foco na árvore:

- `↑` e `↓` percorrem os itens visíveis;
- `→` expande um item ou entra no primeiro filho quando ele já está expandido;
- `←` recolhe um item ou volta ao pai;
- `Home` e `End` vão ao primeiro e ao último item visível;
- `Enter` e `Space` selecionam o documento focado;
- itens recolhidos não permanecem no percurso de foco.

### Centro
- título do documento;
- editor;
- estado de save;
- word count.

### Inspector
Contextual e recolhível.

## Metadata

Seções:
- Livro;
- Autores e contribuidores;
- Identificadores;
- Publicação;
- Direitos e Copyright.

## Design

- Theme gallery;
- Page;
- Typography;
- Chapters;
- Paragraphs;
- Scene breaks;
- Header/Footer.

Mudanças atualizam preview sem alterar o conteúdo.

## Preview

- página dupla/simples;
- navegação por capítulo/página;
- zoom;
- alternar perfil;
- warnings clicáveis.

## Export

- formato;
- perfil;
- destino;
- validação;
- summary dos metadados;
- botão Export;
- resultado e “mostrar no explorer/finder”.

## Dialogs críticos

### Excluir documento
Deve preferir “Mover para lixeira” a exclusão definitiva.

### Missing asset
Explicar:
- qual asset;
- onde é usado;
- localizar/substituir/remover referência.

### Recovery
Mostrar:
- projeto;
- data da última sessão normal;
- recovery disponível;
- opção de abrir recovery ou estado salvo.

## Command palette futura

Pode acelerar usuários avançados sem tornar menus complexos.

Comandos possíveis:
- novo capítulo;
- ir para capítulo;
- abrir metadata;
- abrir preview;
- exportar;
- focus mode.
