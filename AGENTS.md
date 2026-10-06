# AGENTS.md — BookMaker

Este arquivo define as regras obrigatórias para agentes de código, assistentes de IA e contribuidores automatizados trabalhando no BookMaker.

O objetivo deste documento não é descrever toda a arquitetura novamente. A documentação em `docs/` é a fonte de contexto técnico e de produto. Este arquivo define **como trabalhar no projeto**.

---

## 1. Missão

Construir o BookMaker como uma engine editorial desktop, local-first, simples, rápida e intuitiva, capaz de levar um autor do primeiro capítulo até um livro pronto para publicação.

A V1 é focada exclusivamente em livros e `FlowDocument`.

A arquitetura deve continuar preparada para uma futura implementação de HQ/mangá por meio de `CanvasDocument`, mas **nenhuma funcionalidade de edição Canvas deve ser implementada antes de existir uma task explícita para isso**.

A experiência do usuário deve priorizar simplicidade e fluidez, tomando o Reedsy Studio como referência conceitual de fluxo, sem copiar sua interface, identidade, textos ou assets.

---

## 2. Fontes de verdade

Antes de trabalhar em uma task, leia o contexto relevante do projeto.

Ordem de autoridade:

1. task atual;
2. `AGENTS.md`;
3. ADRs em `docs/adr/`;
4. documentação técnica e de produto em `docs/`;
5. `README.md`;
6. código existente e testes.

Se duas fontes entrarem em conflito:

- não invente uma solução silenciosamente;
- preserve a decisão de maior prioridade;
- registre a inconsistência no relatório final da task;
- atualize documentação somente quando isso fizer parte da task ou for necessário para manter o contrato correto.

O roadmap e o backlog fornecem **contexto**, não permissão para implementar trabalho futuro.

---

## 3. Regra mais importante: trabalhar em etapas

O BookMaker deve ser desenvolvido incrementalmente.

### Uma task por vez

Ao receber uma task:

1. entenda a task;
2. identifique suas dependências reais;
3. implemente somente o necessário para concluí-la corretamente;
4. valide o resultado;
5. pare.

**Não continue automaticamente para a próxima task.**

Mesmo que a próxima etapa pareça óbvia, aguarde uma nova instrução.

### Stop Rule

Quando os critérios de aceite da task atual forem atendidos:

> PARE A IMPLEMENTAÇÃO.

Depois disso, apenas:

- execute as validações finais;
- informe arquivos alterados;
- informe testes executados;
- informe riscos ou pendências;
- sugira, no máximo, qual seria a próxima task lógica sem implementá-la.

### Não implementar por antecipação

É proibido implementar uma feature apenas porque ela aparece:

- no roadmap;
- no backlog;
- em um documento de arquitetura;
- como possível necessidade futura.

Exemplo:

Se a task é `BM-001 criar monorepo`, não inicialize Tiptap, SQLite, Typst, EPUB, sistema de copyright ou editor apenas porque serão necessários depois.

---

## 4. Filosofia de engenharia

### Menos é mais

Escolha a solução mais simples que:

- resolve completamente a task atual;
- respeita os contratos arquiteturais existentes;
- permite evolução natural;
- pode ser testada e entendida facilmente.

Não confunda simplicidade com código descartável.

O código deve ser simples **e sólido**.

### KISS

Evite soluções sofisticadas quando uma solução direta resolve o problema com clareza.

### YAGNI

Não implemente requisitos hipotéticos.

Não crie infraestrutura para uma necessidade futura sem uma task concreta que a justifique.

### DRY com bom senso

Evite duplicação de regras importantes, mas não crie abstrações genéricas cedo demais apenas para eliminar duas ou três linhas parecidas.

Uma abstração ruim é mais cara que uma pequena duplicação temporária.

### Progressive Complexity

A complexidade deve surgir conforme o produto realmente precisa dela.

Primeiro:

- caso simples;
- contrato claro;
- teste;
- uso real.

Depois, somente quando necessário:

- generalização;
- otimização;
- extensibilidade adicional.

---

## 5. Clean Code é obrigatório

Todo código do BookMaker deve seguir princípios de Clean Code.

### Nomes

Use nomes explícitos e orientados ao domínio.

Prefira:

```ts
createProject
selectedDocumentId
copyrightHolder
publicationMetadata
```

Evite:

```ts
doStuff
handleData
obj
manager2
utils2
```

### Funções

Funções devem:

- ter uma responsabilidade clara;
- ser pequenas quando isso melhora entendimento;
- evitar muitos níveis de indentação;
- evitar parâmetros booleanos obscuros;
- retornar tipos previsíveis;
- separar regra de negócio de efeitos colaterais quando possível.

### Componentes React

Componentes devem representar UI e comportamento de apresentação.

Não colocar em componentes React:

- regras editoriais;
- regras de persistência;
- queries SQL;
- acesso ao filesystem;
- lógica de exportação;
- regras de copyright;
- transformação central de documentos.

### Classes e módulos

Cada módulo deve possuir responsabilidade identificável.

Evite:

- `Manager` genérico;
- `Helper` que conhece o sistema inteiro;
- `Utils` como depósito de funções sem domínio;
- classes gigantes;
- serviços que concentram regras não relacionadas.

### Comentários

Use comentários para explicar **por quê**, não para narrar código óbvio.

Código claro é preferível a comentários compensando nomes ruins.

### Tipos

TypeScript deve permanecer em `strict`.

Evite `any`.

Use tipos de domínio quando isso reduz ambiguidade.

Use discriminated unions para estados relevantes.

Valide dados nas boundaries.

### Erros

Erros devem ser tratados de forma explícita.

Não:

- engolir exceções;
- retornar `undefined` silenciosamente para falhas críticas;
- exibir stack trace cru como UX;
- mascarar falhas de persistência como sucesso.

---

## 6. Arquitetura obrigatória

Arquitetura: **Modular Monolith**.

Fluxo principal:

```text
UI -> Application -> Domain -> Ports -> Adapters/Persistence
```

Publicação:

```text
Domain -> Publishing IR -> Renderer -> Artifact
```

### Regras

1. Não colocar regras de domínio em componentes React.
2. Não acessar SQLite diretamente da UI.
3. Não acessar filesystem diretamente da UI.
4. Não persistir HTML/CSS arbitrário como modelo principal do manuscrito.
5. `Document` deve continuar compatível conceitualmente com `FlowDocument | CanvasDocument`.
6. A V1 implementa apenas `FlowDocument`.
7. Exportadores recebem Publishing IR, nunca o DOM visual da aplicação.
8. Alterações de schema persistido exigem migration e teste.
9. Não duplicar a mesma regra de negócio em TypeScript e Rust sem motivo concreto.
10. Não criar microserviços.
11. Não criar event bus global sem necessidade comprovada.
12. Não criar arquitetura de plugins antes de existir requisito explícito.
13. Não criar camadas artificiais apenas para “seguir arquitetura”.

Arquitetura existe para reduzir acoplamento, não para aumentar quantidade de arquivos.

---

## 7. Stack aprovada

- Tauri 2.x
- React
- TypeScript strict
- Tiptap / ProseMirror
- Zustand
- Rust para serviços nativos
- SQLite
- filesystem local para assets
- Typst para PDF
- renderer próprio para EPUB 3
- Vitest
- Testing Library
- Playwright quando E2E fizer sentido
- `cargo test`

### Dependências

Antes de adicionar uma dependência:

1. confirme que a task realmente precisa dela;
2. verifique se a plataforma/stack já resolve o problema;
3. prefira bibliotecas maduras e pequenas;
4. avalie custo de manutenção;
5. não instale bibliotecas “para talvez usar depois”.

Toda dependência significativa deve ter justificativa clara.

---

## 8. UX e fluxo são requisitos de engenharia

BookMaker deve ser simples mesmo quando internamente possui recursos complexos.

Toda feature deve buscar:

- poucas etapas;
- ações previsíveis;
- feedback imediato;
- defaults úteis;
- progressive disclosure;
- baixa carga cognitiva;
- consistência entre telas;
- mensagens de erro compreensíveis;
- teclado e mouse funcionando naturalmente.

Evite:

- telas cheias de opções;
- modais desnecessários;
- configurações obrigatórias antes de escrever;
- workflows longos;
- menus redundantes;
- controles avançados expostos sem necessidade.

Se houver duas soluções funcionalmente equivalentes, prefira a que exige menos decisões do usuário.

---

## 9. Persistência e segurança do trabalho

O usuário está produzindo um livro. Perder texto é um erro crítico.

Nunca trate persistência como detalhe secundário.

Ao trabalhar em features que alteram dados persistidos, considere quando aplicável:

- save/reload;
- atomicidade;
- autosave;
- recovery;
- migrations;
- falhas de disco/permissão;
- integridade após crash;
- compatibilidade com versões anteriores.

Nunca informe visualmente “Salvo” antes de a operação relevante realmente ter sido concluída.

Nunca registrar o texto integral do manuscrito em logs por padrão.

---

## 10. Editor

O editor deve permanecer semântico.

Conteúdo e aparência são responsabilidades diferentes.

Exemplo conceitual:

```text
sceneBreak
```

é significado editorial.

A aparência visual desse elemento pertence ao tema/renderização.

Ao criar nodes/extensions Tiptap:

- manter schema explícito;
- preservar undo/redo;
- preservar copy/paste quando aplicável;
- considerar Unicode;
- garantir serialização estável;
- evitar dependência da aparência atual da UI.

---

## 11. Copyright e metadados editoriais

BookMaker deve permitir que produtores configurem corretamente a página e os metadados de copyright de suas obras.

O domínio deve poder representar, quando implementado pelas tasks correspondentes:

- ano;
- titular(es) dos direitos;
- aviso de copyright;
- edição;
- publisher opcional;
- ISBN/identificadores opcionais;
- contribuidores/créditos;
- licença;
- disclaimers;
- texto customizado.

Esses dados devem futuramente alimentar de forma consistente:

- preview;
- página de copyright;
- PDF;
- EPUB metadata.

A UI nunca deve afirmar que gerar uma página de copyright equivale a registrar juridicamente direitos autorais perante um órgão governamental.

Não implementar esse módulo antes das tasks correspondentes.

---

## 12. Processo obrigatório para cada task

### Etapa A — Contexto

Antes de alterar código:

1. leia `AGENTS.md`;
2. leia a task atual;
3. leia os documentos diretamente relacionados;
4. verifique ADRs relacionados;
5. inspecione o código existente;
6. identifique testes existentes relevantes.

Não releia mecanicamente todos os documentos a cada pequena task se o contexto já estiver claro, mas na primeira interação com o repositório leia todo o conjunto de documentação para formar o mapa do projeto.

### Etapa B — Definir escopo

Antes da implementação, declare internamente:

- objetivo;
- critérios de aceite;
- não objetivos;
- arquivos/módulos provavelmente afetados;
- testes necessários.

Se uma task for grande demais, divida-a em subtasks pequenas antes de codificar.

Não transforme essa decomposição em autorização para executar todas as subtasks de uma vez.

### Etapa C — Implementar o mínimo correto

Implemente somente o necessário para os critérios de aceite.

Evite:

- refactors laterais sem necessidade;
- novas abstrações não exigidas;
- redesign visual amplo;
- features próximas;
- otimizações sem medida;
- preparação excessiva para roadmap futuro.

### Etapa D — Validar

Execute somente as verificações aplicáveis, incluindo quando apropriado:

```text
format
lint
typecheck
unit tests
integration tests
cargo fmt
cargo clippy
cargo test
build
```

Não declare sucesso quando validações relevantes falharem.

### Etapa E — Revisão de simplicidade

Antes de concluir, pergunte:

- existe código que foi criado sem necessidade para esta task?
- existe abstração prematura?
- há uma forma mais simples mantendo os mesmos contratos?
- algum nome pode ser mais claro?
- alguma responsabilidade está no módulo errado?

Remova complexidade acidental encontrada.

### Etapa F — Encerrar

Antes do relatório final, toda task concluída deve possuir seu próprio commit.

Regras obrigatórias de commit:

- criar o commit somente depois que as validações aplicáveis passarem;
- incluir apenas as alterações pertencentes à task atual;
- usar uma mensagem curta e orientada ao resultado da task;
- não adicionar `Co-authored-by` nem qualquer outro trailer de coautoria;
- não reescrever commits anteriores ou incluir alterações alheias para obter uma árvore limpa;
- informar o hash e a mensagem do commit no relatório final.

Se a task não puder ser concluída ou se uma validação relevante falhar, não criar um commit declarando sucesso. Relate o bloqueio e preserve as alterações para revisão.

Relatório final da task deve ser curto e objetivo:

```text
Task concluída: <id/nome>

Implementado:
- ...

Validações:
- ...

Commit:
- <hash e mensagem; sem coautoria>

Arquivos principais:
- ...

Pendências/riscos:
- ...

Próxima task sugerida:
- <somente sugestão; não implementar>
```

---

## 13. Definition of Done

Uma task só pode ser marcada como concluída quando os critérios relevantes de `docs/18-DEFINITION-OF-DONE.md` forem satisfeitos.

No mínimo:

- comportamento esperado funcionando;
- código legível;
- sem complexidade desnecessária;
- tipos corretos;
- testes proporcionais ao risco;
- lint/typecheck aplicáveis sem erro;
- estados de erro relevantes tratados;
- documentação atualizada quando o contrato mudou;
- nenhum TODO crítico escondendo parte necessária da task.

Não escreva testes vazios apenas para aumentar quantidade.

Testes devem proteger comportamento importante.

---

## 14. O que NÃO fazer

Não:

- implementar várias epics em uma única etapa;
- criar toda a V1 em um único prompt;
- “adiantar” tasks futuras;
- criar infraestrutura especulativa;
- criar abstrações só porque parecem elegantes;
- criar repository/service/factory para cada arquivo automaticamente;
- criar microserviços;
- duplicar domínio completo em Rust e TypeScript;
- implementar CanvasDocument editável durante V1;
- criar cloud/sync/contas sem task explícita;
- adicionar IA ao produto sem task explícita;
- redesenhar partes não relacionadas durante uma task;
- instalar dezenas de dependências no bootstrap;
- esconder falhas para deixar CI verde;
- alterar requisitos silenciosamente.

---

## 15. Prioridades em caso de dúvida

Quando duas opções forem válidas, priorize nesta ordem:

1. integridade dos dados do usuário;
2. comportamento correto;
3. simplicidade para o usuário;
4. simplicidade do código;
5. clareza arquitetural;
6. testabilidade;
7. performance medida;
8. extensibilidade futura comprovadamente necessária.

Nunca troque simplicidade atual por extensibilidade hipotética.

---

## 16. Regra final

> Faça a menor mudança capaz de concluir bem a task atual, mantendo o projeto limpo, testável, consistente e pronto para receber a próxima mudança.

BookMaker deve crescer por **pequenos passos corretos**, não por grandes saltos especulativos.
