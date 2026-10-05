# 09 — Copyright, Créditos e Metadados Editoriais

## Objetivo

Permitir que autores produzam uma página de copyright profissional e incluam metadados de direitos nos formatos exportados.

BookMaker não realiza registro legal de obra e não deve sugerir que a presença de uma página de copyright substitui procedimentos jurídicos aplicáveis.

## Dados estruturados

### Obra
- título;
- subtítulo;
- idioma;
- edição;
- publisher/imprint;
- data de publicação;
- identificadores.

### Direitos
- ano de copyright;
- titular(es);
- texto de direitos;
- licença;
- território, opcional/futuro;
- disclaimer.

### Identificadores
Tipo extensível:

```ts
interface Identifier {
  type: 'isbn10' | 'isbn13' | 'doi' | 'uuid' | 'custom';
  value: string;
  label?: string;
}
```

Não validar ISBN apenas por regex; quando implementado, usar checksum.

### Créditos
Cada contribuição é dado estruturado:

```ts
interface Contributor {
  id: ContributorId;
  name: string;
  roles: ContributorRole[];
  sortName?: string;
}
```

## Modos da página de copyright

### Generated
BookMaker compõe a página usando template e dados.

Exemplo conceitual, não texto legal obrigatório:

```text
Copyright © {year} {holders}

{rightsStatement}

{edition}
{publisher}
{isbn}

Créditos...
```

### Custom
Usuário escreve livremente o corpo da página.

Mesmo no modo custom, manter metadados estruturados para EPUB/PDF e futuras integrações.

## Presets

Podem existir presets de conveniência:
- Todos os direitos reservados;
- Creative Commons selecionável futuramente;
- domínio público/declarativo, com cautela;
- custom.

Nunca gerar aconselhamento jurídico personalizado automaticamente.

## EPUB metadata

Mapear dados do projeto para metadata do pacote EPUB, incluindo:
- title;
- creator/contributors;
- language;
- identifier;
- rights;
- publisher quando informado.

## PDF metadata

Inserir quando suportado:
- Title;
- Author;
- Subject/Description;
- Keywords futuramente;
- copyright/rights quando o pipeline permitir.

## Capa e créditos de imagem

Assets podem armazenar:
- autor/criador;
- fonte;
- licença;
- texto de atribuição;
- observações.

Na V1 isso pode ser metadata não automaticamente impressa. A arquitetura deve permitir criar uma página futura de créditos de imagens.

## Privacidade

Metadados exportados podem conter nomes reais. Antes da exportação, a UI deve permitir revisar quais campos serão incorporados ao arquivo final.
