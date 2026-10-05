# 17 — Riscos Técnicos e de Produto

## R1 — Editor rich-text virar Word

**Risco:** demanda por customizações inline destrói o modelo semântico.

**Mitigação:** schema restrito, StyleProfile, critérios de produto claros.

## R2 — Preview diferente do PDF

**Risco:** usuário confia no preview e recebe artefato substancialmente diferente.

**Mitigação:** compartilhar Publishing IR; rotular diferenças inevitáveis; testes golden.

## R3 — Paginação é mais difícil que parece

**Risco:** widows/orphans, footnotes, imagens e headers tornam layout complexo.

**Mitigação:** delegar layout de impressão ao Typst; não implementar engine tipográfica própria.

## R4 — Corrupção/perda de manuscrito

**Risco:** destrói confiança no produto.

**Mitigação:** transações, autosave, recovery, snapshots de migration, testes de crash.

## R5 — Tauri + web editor inconsistências de plataforma

**Mitigação:** E2E nos sistemas suportados; abstração de OS; evitar hacks específicos no core.

## R6 — EPUB aparentemente correto mas inválido

**Mitigação:** validator automatizado e fixtures; pipeline baseado em specs, não em “abre no meu leitor”.

## R7 — Font licensing

**Risco:** incorporar fonte no PDF/EPUB sem licença adequada.

**Mitigação:** não distribuir fontes de terceiros sem direito; informar ao usuário que fontes importadas são responsabilidade dele; registrar metadata de embedding se necessário futuramente.

## R8 — Copyright confundido com registro legal

**Mitigação:** copy explícita na UI: BookMaker cria página/metadados de copyright; não registra a obra nem oferece aconselhamento jurídico.

## R9 — HQ/mangá contaminar V1

**Risco:** arquitetura futura vira desculpa para aumentar escopo.

**Mitigação:** apenas contratos e boundaries agora; nenhuma UI Canvas na V1.

## R10 — Arquitetura excessiva para equipe pequena

**Mitigação:** Modular Monolith, poucos packages, ADRs somente para decisões relevantes, sem abstrações sem caso real.

## R11 — Dependência excessiva do formato ProseMirror

**Mitigação:** versionar schema; encapsular transformação; migrations de conteúdo; Publishing IR não depende do JSON cru.
