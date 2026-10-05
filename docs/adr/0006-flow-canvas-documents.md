# ADR 0006 — FlowDocument e CanvasDocument

Status: Accepted

## Decisão

A abstração `Document` é uma união discriminada. A V1 implementa `FlowDocument`; `CanvasDocument` representa a extensão futura para HQ/mangá.

## Motivo

Não queremos remodelar o projeto inteiro quando páginas espaciais forem adicionadas.

## Restrição

A existência do contrato não autoriza aumentar o escopo da V1 com funcionalidades Canvas.
