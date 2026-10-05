# 07 — Publicação, PDF e EPUB

## Regra central

A exportação nunca deve serializar a UI.

```text
Domain
  -> Publication Builder
  -> Publishing IR
  -> Renderer específico
  -> Artifact
```

## Publication Builder

Responsabilidades:
- resolver ordem do livro;
- inserir front/back matter;
- gerar TOC;
- resolver copyright page;
- aplicar metadata;
- resolver referências de assets;
- produzir warnings;
- construir IR estável.

## Preview

Preview usa a mesma IR dos exportadores sempre que possível.

Não é obrigatório que o preview web seja pixel-perfect ao PDF, mas ele deve representar:
- hierarquia;
- page breaks principais;
- tipografia aproximada;
- imagens;
- headers/footers;
- copyright/front matter;
- ordem final.

## PDF

Pipeline sugerido:

```text
Publishing IR -> Typst source/model -> Typst compiler -> PDF
```

O BookMaker controla templates Typst internos. O usuário não edita Typst na V1.

### Requisitos
- tamanho de página;
- margens internas/externas;
- frente e verso quando aplicável;
- numeração;
- headers/footers;
- capítulos iniciando em nova página;
- imagens;
- hyperlinks quando compatíveis;
- metadata PDF;
- fontes incorporadas quando licença/arquivo permitirem.

### Posterior
- bleed;
- crop marks;
- PDF/X;
- presets de gráficas;
- widows/orphans avançado;
- hyphenation por idioma refinada.

## EPUB 3

Gerar pacote próprio:
- `mimetype`;
- `META-INF/container.xml`;
- package document;
- navigation document;
- XHTML por capítulo/seção;
- CSS;
- assets;
- metadata;
- cover quando fornecida.

### Princípio
EPUB é reflowable por padrão. Não reproduzir à força a paginação do PDF.

### Validação
Integrar etapa automatizada de validação no pipeline de build/teste e, quando possível, na exportação.

Warnings devem diferenciar:
- erro bloqueante;
- risco editorial;
- recomendação.

## Perfis de exportação

```ts
interface ExportProfile {
  id: ExportProfileId;
  name: string;
  format: 'pdf' | 'epub';
  styleProfileId: StyleProfileId;
  options: Record<string, unknown>;
}
```

Exemplos:
- EPUB padrão;
- PDF 6x9 impressão;
- PDF A5 leitura.

## Determinismo

Mesma versão do projeto + versão do renderer + mesmas fontes/assets deve produzir conteúdo equivalente.

Armazenar versão do exportador no relatório de exportação para debugging.

## Relatório de exportação

Após exportar, registrar:
- formato;
- destino;
- timestamp;
- versão do app;
- warnings;
- duração;
- hash opcional do artefato.
