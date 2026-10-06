import { describe, expect, it } from "vitest";

import { findDocumentMatches } from "./documentSearch";
import { editorSchema } from "./editorSchema";

function createDocument() {
  return editorSchema.nodeFromJSON({
    type: "doc",
    content: [
      {
        type: "heading",
        attrs: { level: 1 },
        content: [{ type: "text", text: "Capítulo único" }],
      },
      {
        type: "paragraph",
        content: [
          { type: "text", text: "Book", marks: [{ type: "bold" }] },
          { type: "text", text: "Maker encontra outro bookmaker." },
        ],
      },
      {
        type: "paragraph",
        content: [{ type: "text", text: "Fim (literal)." }],
      },
      {
        type: "paragraph",
        content: [{ type: "text", text: "Início com 世界 e capítulo." }],
      },
      { type: "sceneBreak" },
    ],
  });
}

describe("document search", () => {
  it("returns no matches for an empty query", () => {
    expect(findDocumentMatches(createDocument(), "")).toEqual([]);
  });

  it("finds literal occurrences without distinguishing letter case", () => {
    const document = createDocument();
    const matches = findDocumentMatches(document, "bookmaker");

    expect(matches).toHaveLength(2);
    expect(
      matches.map((match) => document.textBetween(match.from, match.to)),
    ).toEqual(["BookMaker", "bookmaker"]);
  });

  it("supports Unicode and treats regular-expression characters literally", () => {
    const document = createDocument();

    const unicodeMatch = findDocumentMatches(document, "世界");
    const literalMatch = findDocumentMatches(document, "(literal).");

    expect(
      document.textBetween(unicodeMatch[0]!.from, unicodeMatch[0]!.to),
    ).toBe("世界");
    expect(
      document.textBetween(literalMatch[0]!.from, literalMatch[0]!.to),
    ).toBe("(literal).");
  });

  it("does not join text from separate editorial blocks", () => {
    expect(findDocumentMatches(createDocument(), "Fim Início")).toEqual([]);
  });

  it("keeps accent differences significant", () => {
    expect(findDocumentMatches(createDocument(), "capitulo")).toEqual([]);
    expect(findDocumentMatches(createDocument(), "capítulo")).toHaveLength(2);
  });
});
