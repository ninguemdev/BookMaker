# ADR 0007 — Zod para validação em runtime no TypeScript

Status: Accepted

## Contexto

Manifestos, dados persistidos, IPC e conteúdo importado entram na aplicação como dados não confiáveis. Os tipos TypeScript não validam esses valores em runtime.

## Decisão

Usar Zod nas boundaries TypeScript para validar dados externos antes de convertê-los em tipos de domínio. O schema do manifest do projeto é a primeira aplicação dessa estratégia.

## Consequências

Positivas:
- schemas executáveis e tipos TypeScript derivados da mesma definição;
- erros estruturados para tratamento nas camadas de aplicação e UI;
- estratégia única para as próximas boundaries TypeScript.

Negativas:
- dependência de runtime adicional;
- schemas persistidos exigem manutenção coordenada com suas migrations.
