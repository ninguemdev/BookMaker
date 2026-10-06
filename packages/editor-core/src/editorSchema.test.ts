import { describe, expect, it } from "vitest";

import {
  EDITOR_SCHEMA_VERSION,
  InvalidEditorContentError,
  createEmptyEditorContent,
  editorSchema,
  isEditorContent,
  isSafeEditorLink,
  parseEditorContent,
} from "./editorSchema";

const supportedContent = {
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
        { type: "text", text: "Olá, 世界 — " },
        {
          type: "text",
          marks: [{ type: "bold" }, { type: "italic" }, { type: "underline" }],
          text: "BookMaker",
        },
        { type: "hardBreak" },
        {
          type: "text",
          marks: [{ type: "link", attrs: { href: "https://example.com" } }],
          text: "Referência",
        },
      ],
    },
    {
      type: "blockquote",
      content: [
        {
          type: "paragraph",
          content: [{ type: "text", text: "Uma citação." }],
        },
      ],
    },
    {
      type: "bulletList",
      content: [
        {
          type: "listItem",
          content: [
            {
              type: "paragraph",
              content: [{ type: "text", text: "Item" }],
            },
          ],
        },
      ],
    },
    {
      type: "orderedList",
      attrs: { start: 3 },
      content: [
        {
          type: "listItem",
          content: [{ type: "paragraph" }],
        },
      ],
    },
  ],
};

describe("editor schema v1", () => {
  it("uses version 1 and accepts the persisted empty flow document", () => {
    expect(EDITOR_SCHEMA_VERSION).toBe(1);
    expect(parseEditorContent(createEmptyEditorContent())).toEqual({
      type: "doc",
    });
  });

  it("contains only the supported v1 node and mark types", () => {
    expect(Object.keys(editorSchema.nodes).sort()).toEqual([
      "blockquote",
      "bulletList",
      "doc",
      "hardBreak",
      "heading",
      "listItem",
      "orderedList",
      "paragraph",
      "text",
    ]);
    expect(Object.keys(editorSchema.marks).sort()).toEqual([
      "bold",
      "italic",
      "link",
      "underline",
    ]);
  });

  it("round-trips supported semantic content and Unicode text", () => {
    const parsed = parseEditorContent(supportedContent);

    expect(parsed.type).toBe("doc");
    expect(JSON.stringify(parsed)).toContain("Olá, 世界");
    expect(isEditorContent(parsed)).toBe(true);
    expect(parseEditorContent(parsed)).toEqual(parsed);
  });

  it.each([1, 2, 3])("accepts heading level %i", (level) => {
    expect(
      isEditorContent({
        type: "doc",
        content: [{ type: "heading", attrs: { level } }],
      }),
    ).toBe(true);
  });

  it("accepts safe editor links and rejects executable URLs", () => {
    expect(isSafeEditorLink("https://example.com/book")).toBe(true);
    expect(isSafeEditorLink("chapter-2.xhtml#section")).toBe(true);
    expect(isSafeEditorLink("javascript:alert(1)")).toBe(false);
    expect(isSafeEditorLink("   ")).toBe(false);
  });

  it.each([
    [
      "an unsupported heading level",
      {
        type: "doc",
        content: [{ type: "heading", attrs: { level: 4 } }],
      },
    ],
    ["an unknown node", { type: "doc", content: [{ type: "image" }] }],
    [
      "an unsupported mark",
      {
        type: "doc",
        content: [
          {
            type: "paragraph",
            content: [
              { type: "text", text: "Code", marks: [{ type: "code" }] },
            ],
          },
        ],
      },
    ],
    [
      "invalid nesting",
      { type: "doc", content: [{ type: "text", text: "Loose text" }] },
    ],
    [
      "an arbitrary style attribute",
      {
        type: "doc",
        content: [{ type: "paragraph", attrs: { style: "font-size: 17.3px" } }],
      },
    ],
    [
      "an unsafe link URL",
      {
        type: "doc",
        content: [
          {
            type: "paragraph",
            content: [
              {
                type: "text",
                text: "Unsafe",
                marks: [
                  { type: "link", attrs: { href: "javascript:alert(1)" } },
                ],
              },
            ],
          },
        ],
      },
    ],
    ["a non-document root", { type: "paragraph" }],
    ["a malformed value", null],
  ])("rejects %s", (_case, content) => {
    expect(() => parseEditorContent(content)).toThrow(
      InvalidEditorContentError,
    );
    expect(isEditorContent(content)).toBe(false);
  });
});
