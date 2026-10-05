# 22 — Estratégia de Build, Release e Distribuição

## Plataformas

Arquitetura alvo:
- Windows;
- macOS;
- Linux.

Estratégia pragmática sugerida para equipe pequena:
1. Windows como primeira plataforma de desenvolvimento/distribuição;
2. validar macOS antes de beta pública ampla;
3. Linux suportado conforme testes de WebView/packaging.

Isso é prioridade de execução, não licença para introduzir dependências Windows no domínio.

## Canais

- `dev` — builds locais/CI;
- `beta` — testers;
- `stable` — público.

## Versionamento

SemVer para aplicação, com formato de projeto versionado separadamente.

```text
App: 1.3.0
Project format: 4
Editor schema: 6
Publishing IR: internal version if serialized/cached
```

Não assumir que versionamento do app substitui migrations.

## Build reproducível

- lockfiles;
- versões toolchain fixadas em CI;
- hashes dos artefatos;
- changelog;
- build metadata.

## Assinatura

Antes de distribuição pública:
- code signing Windows;
- signing/notarization macOS;
- assinatura do updater.

## Auto-update

Não implementar antes da fundação de build/signing.

Requisitos:
- assinatura criptográfica;
- stable/beta separados;
- rollback manual claro;
- update nunca deve migrar projeto até o usuário abrir esse projeto conscientemente.

## Crash reports

Se adicionados:
- consentimento/política transparente;
- remover texto do manuscrito;
- IDs de projeto não precisam ser enviados;
- incluir versão app/OS e stack técnica suficiente.

## Checklist de release

- CI verde;
- migration suite verde;
- fixtures antigos abrem;
- E2E de livro principal;
- EPUB validation verde;
- PDF smoke test;
- recovery test;
- assinatura;
- changelog;
- teste de update quando aplicável;
- backup dos artefatos de release.
