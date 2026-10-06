import { describe, expect, it } from "vitest";

import { editorSchema } from "./editorSchema";
import { countDocumentWords, countWords } from "./wordCount";

describe("word count", () => {
  it.each([
    ["", 0],
    ["   … — 😀", 0],
    ["Olá, mundo! 42 vezes.", 4],
    ["d'água guarda-chuva l’amour co‑autor", 4],
    ["Cafe\u0301 世界 Привет", 3],
  ])("counts %j consistently", (text, expectedCount) => {
    expect(countWords(text)).toBe(expectedCount);
  });

  it("counts semantic document text without splitting adjacent marks", () => {
    const document = editorSchema.nodeFromJSON({
      type: "doc",
      content: [
        {
          type: "paragraph",
          content: [
            { type: "text", text: "Book", marks: [{ type: "bold" }] },
            { type: "text", text: "Maker" },
            { type: "hardBreak" },
            { type: "text", text: "escreve livros" },
          ],
        },
        { type: "sceneBreak" },
        {
          type: "image",
          attrs: {
            alignment: "center",
            alt: "Capa ilustrada",
            assetId: "0199bc4e-8b72-7000-8000-000000000001",
            caption: "Texto do atributo",
            decorative: false,
            width: "medium",
          },
        },
        {
          type: "bulletList",
          content: [
            {
              type: "listItem",
              content: [
                {
                  type: "paragraph",
                  content: [{ type: "text", text: "Último item" }],
                },
              ],
            },
          ],
        },
      ],
    });

    expect(countDocumentWords(document)).toBe(5);
  });
});
