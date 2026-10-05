# ADR 0004 — Typst como motor de PDF

Status: Accepted, sujeito a spike técnico antes do milestone Publishing.

## Decisão

Usar Typst como principal candidato para typesetting/exportação PDF.

## Motivo

Paginação e tipografia de livros são problemas complexos. O projeto não deve implementar um motor tipográfico próprio.

## Validação necessária

Antes de congelar a integração, executar spike com:
- capítulos;
- imagens;
- headers/footers;
- numeração;
- front matter;
- fontes;
- links;
- 100k+ palavras;
- plataformas alvo.

Se houver bloqueador técnico ou de distribuição, registrar novo ADR antes de substituir.
