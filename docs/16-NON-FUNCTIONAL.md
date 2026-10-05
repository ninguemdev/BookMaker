# 16 — Requisitos Não Funcionais

## Confiabilidade

- nenhuma ação normal deve apagar conteúdo sem confirmação/undo/lixeira;
- autosave é padrão;
- falhas de exportação não alteram o manuscrito;
- migrations importantes criam ponto de recuperação.

## Performance

- digitação deve permanecer responsiva independentemente do tamanho total do projeto;
- lazy load de documentos;
- operações longas devem sair da thread principal da UI;
- exportação deve fornecer progresso quando houver etapas perceptíveis.

## Portabilidade

Meta desktop:
- Windows;
- macOS;
- Linux.

A prioridade real de release pode começar por Windows, mas o código não deve depender deliberadamente de APIs Windows quando houver abstração Tauri disponível.

## Compatibilidade de projeto

- versão N deve abrir projetos suportados das versões anteriores via migration;
- versões antigas nunca devem silenciosamente rebaixar formatos novos;
- unknown fields devem ser tratados conforme política definida por schema.

## Internacionalização

Desde o início:
- strings de UI fora dos componentes quando viável;
- UTF-8;
- locale separado do idioma do livro;
- datas formatadas por locale;
- livro pode ter idioma diferente da UI.

## Acessibilidade

- navegação por teclado;
- focus indicators;
- semantic HTML na UI;
- contraste;
- escala/zoom;
- não depender apenas de cor para estado.

## Observabilidade local

- logs rotativos;
- export report;
- crash diagnostics opt-in quando houver backend de telemetria;
- botão futuro para gerar pacote de diagnóstico sem manuscrito.

## Manutenibilidade

- módulos com APIs explícitas;
- cobertura de caminhos críticos;
- ADR para decisões duradouras;
- dependências externas encapsuladas quando críticas.
