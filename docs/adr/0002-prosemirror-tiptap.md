# ADR 0002 — Tiptap / ProseMirror para manuscrito

Status: Accepted

## Decisão

Usar Tiptap sobre ProseMirror e um schema editorial restrito/versionado.

## Motivo

O domínio exige seleção, input, IME, undo/redo, copy/paste e extensibilidade sem construir um rich-text editor do zero.

## Consequência importante

O JSON ProseMirror é persistência de conteúdo editável, mas não é o formato final de publicação. Exportação passa por transformação para Publishing IR.
