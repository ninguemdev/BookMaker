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

### Sidebar esquerda
- Front Matter
- Manuscript
- Back Matter
- botão adicionar
- busca/filtro

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
