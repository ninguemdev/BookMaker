# 18 — Definition of Done

Uma feature não está concluída apenas porque “funciona na máquina do desenvolvedor”.

## Feature comum

- comportamento especificado;
- estados vazios/erro tratados;
- persistência quando aplicável;
- testes unitários/integration adequados;
- acessibilidade básica;
- sem warning de lint/clippy relevante;
- documentação técnica atualizada quando altera contrato;
- migration criada se alterar persistência;
- nenhum conteúdo do usuário incluído indevidamente em logs.

## Feature de editor

Além do anterior:
- teclado;
- undo/redo;
- paste;
- reload;
- autosave;
- conteúdo Unicode;
- exportação correspondente testada quando o node é publicável.

## Feature de metadata/copyright

- dado salvo/reaberto;
- preview atualizado;
- export EPUB mapeado;
- export PDF mapeado;
- custom mode preservado;
- campo opcional realmente opcional;
- nenhuma afirmação jurídica enganosa na UI.

## Exportador

- saída criada;
- erros claros;
- fixture/golden test;
- metadata testada;
- assets ausentes diagnosticados;
- arquivo parcial removido ou claramente marcado em caso de falha.

## Migration

- forward migration testada;
- fixture anterior preservada;
- failure path testado;
- snapshot/backup quando necessário;
- documentação de compatibilidade atualizada.
