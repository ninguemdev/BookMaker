import type { Node as ProseMirrorNode } from "@tiptap/pm/model";

export interface DocumentTextMatch {
  from: number;
  to: number;
}

function escapeRegularExpression(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

export function findDocumentMatches(
  document: ProseMirrorNode,
  query: string,
): DocumentTextMatch[] {
  if (query.length === 0) {
    return [];
  }

  const matches: DocumentTextMatch[] = [];
  const queryPattern = new RegExp(escapeRegularExpression(query), "giu");

  document.descendants((node, position) => {
    if (!node.isTextblock) {
      return;
    }

    const text = node.textBetween(0, node.content.size, "\n", "\n");
    for (const match of text.matchAll(queryPattern)) {
      if (match.index === undefined) {
        continue;
      }

      const from = position + 1 + match.index;
      matches.push({ from, to: from + match[0].length });
    }

    return false;
  });

  return matches;
}
