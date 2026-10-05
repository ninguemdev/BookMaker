# 13 — Segurança, Privacidade e Integridade

## Modelo de ameaça básico

BookMaker abre conteúdo criado pelo usuário, importa arquivos e executa processos de exportação. Isso exige tratar projetos como dados não confiáveis, especialmente projetos recebidos de terceiros.

## Tauri

- permitir somente comandos necessários;
- configurar capabilities/permissions mínimas;
- não expor shell arbitrário à UI;
- validar todos os paths no boundary Rust;
- não aceitar comandos de filesystem arbitrários vindos do conteúdo do projeto.

## HTML / editor

- sanitizar paste/import;
- não executar scripts contidos em HTML;
- links não devem executar `javascript:`;
- preview deve escapar conteúdo corretamente;
- EPUB importado futuramente é não confiável.

## Filesystem

- impedir path traversal (`../`);
- resolver assets dentro da raiz do projeto;
- exportação escreve somente em destino explicitamente escolhido;
- symlinks exigem política definida antes de serem aceitos.

## Assets

- validar tipo real quando possível, não apenas extensão;
- impor limites razoáveis de tamanho para evitar consumo acidental de memória;
- thumbnails/cache gerados em processo controlado.

## Privacidade

Padrão: nenhuma conta e nenhum upload obrigatório.

Se telemetry for adicionada:
- opt-in ou política explícita;
- nunca enviar manuscrito;
- nunca enviar nomes de arquivos/projetos sem necessidade;
- documentar eventos coletados.

## Atualizações

- builds assinados quando o produto for distribuído publicamente;
- mecanismo de update deve validar assinatura;
- canal stable/beta separado.

## Dependências

- lockfiles commitados;
- updates regulares;
- audit automatizado;
- evitar bibliotecas pequenas/desnecessárias no core.
