# ADR 0001 — Tauri + React + TypeScript

Status: Accepted

## Contexto

BookMaker é uma aplicação desktop centrada em texto, UI complexa, painéis, drag-and-drop e futuro canvas. Precisamos de acesso nativo sem adotar uma engine de jogos.

## Decisão

Usar Tauri 2.x como runtime desktop e React + TypeScript no frontend.

## Consequências

Positivas:
- ecossistema web maduro;
- integração natural com ProseMirror/Tiptap;
- footprint menor que stacks desktop baseadas em Chromium completo em muitos cenários;
- Rust disponível para serviços nativos.

Negativas:
- duas linguagens no produto;
- cuidados com IPC/capabilities;
- diferenças de webview entre plataformas precisam de teste.
