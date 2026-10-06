import type { Node as ProseMirrorNode } from "@tiptap/pm/model";

const WORD_PATTERN =
  /[\p{L}\p{N}][\p{L}\p{N}\p{M}]*(?:['’\p{Pd}][\p{L}\p{N}][\p{L}\p{N}\p{M}]*)*/gu;

export function countWords(text: string): number {
  return text.match(WORD_PATTERN)?.length ?? 0;
}

export function countDocumentWords(document: ProseMirrorNode): number {
  const text = document.textBetween(0, document.content.size, " ", " ");
  return countWords(text);
}
