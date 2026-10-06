// @vitest-environment jsdom

import type { AssetId } from "@bookmaker/domain";
import { Editor } from "@tiptap/core";
import { afterEach, describe, expect, it } from "vitest";

import {
  createEditorExtensions,
  isEditorContent,
  parseEditorContent,
} from "./editorSchema";
import { createImageNode } from "./image";

const assetId = "0199bc4e-8b72-7000-8000-000000000001" as AssetId;
const editors: Editor[] = [];

function createEditor(content: string | Record<string, unknown>): Editor {
  const editor = new Editor({
    content,
    extensions: createEditorExtensions(),
  });
  editors.push(editor);
  return editor;
}

afterEach(() => {
  editors.splice(0).forEach((editor) => editor.destroy());
});

describe("BookImage", () => {
  it("creates and validates semantic image JSON with stable defaults", () => {
    const image = createImageNode({
      assetId,
      alt: "Mapa da jornada",
    });
    const content = {
      type: "doc",
      content: [image],
    };

    expect(image).toEqual({
      type: "image",
      attrs: {
        alignment: "center",
        alt: "Mapa da jornada",
        assetId,
        caption: null,
        decorative: false,
        width: "full",
      },
    });
    expect(parseEditorContent(content)).toEqual(content);
  });

  it("refuses invalid attributes when creating an image", () => {
    expect(() => createImageNode({ assetId: "" as AssetId })).toThrow();
    expect(() =>
      createImageNode({
        alt: "Ornamento",
        assetId,
        decorative: true,
      }),
    ).toThrow();
  });

  it("rejects unsupported persisted image attributes", () => {
    const image = createImageNode({ assetId, alt: "Imagem" });
    const attributes = image.attrs;

    expect(attributes).toBeDefined();
    for (const invalidAttributes of [
      { ...attributes, unexpected: true },
      { ...attributes, alignment: "justify" },
      { ...attributes, width: "42%" },
      { ...attributes, decorative: true },
    ]) {
      expect(
        isEditorContent({
          type: "doc",
          content: [{ ...image, attrs: invalidAttributes }],
        }),
      ).toBe(false);
    }
  });

  it.each([
    ["children", { content: [{ type: "text", text: "hidden" }] }],
    ["marks", { marks: [{ type: "bold" }] }],
  ])("rejects persisted images with %s", (_case, invalidNodeData) => {
    const image = createImageNode({ assetId });
    expect(
      isEditorContent({
        type: "doc",
        content: [{ ...image, ...invalidNodeData }],
      }),
    ).toBe(false);
  });

  it("round-trips approved metadata through HTML without persisting a path", () => {
    const editor = createEditor({
      type: "doc",
      content: [
        createImageNode({
          alignment: "right",
          alt: "Café sobre a mesa",
          assetId,
          caption: "Figura 1 — Café",
          width: "medium",
        }),
      ],
    });

    const html = editor.getHTML();
    expect(html).toContain(`data-asset-id="${assetId}"`);
    expect(html).toContain("Figura 1 — Café");
    expect(html).not.toContain("src=");

    const restored = createEditor(html);
    expect(restored.getJSON()).toEqual(editor.getJSON());
  });

  it("drops untrusted source paths while parsing marked image HTML", () => {
    const editor = createEditor(
      `<figure data-bookmaker-image data-asset-id="${assetId}" data-alignment="center" data-width="full" data-decorative="false"><img src="file:///private/book.png" alt="Capa"></figure>`,
    );

    expect(editor.getJSON().content?.[0]).toMatchObject({
      type: "image",
      attrs: { assetId },
    });
    expect(editor.getHTML()).not.toContain("src=");
  });

  it("inserts an image as one undoable operation and leaves an editable paragraph", () => {
    const editor = createEditor({
      type: "doc",
      content: [
        {
          type: "paragraph",
          content: [{ type: "text", text: "Antes" }],
        },
      ],
    });
    editor.commands.setTextSelection(6);

    expect(
      editor.commands.insertImage({
        alignment: "left",
        alt: "Ilustração",
        assetId,
        width: "large",
      }),
    ).toBe(true);
    expect(editor.getJSON().content?.map((node) => node.type)).toEqual([
      "paragraph",
      "image",
      "paragraph",
    ]);
    expect(editor.state.selection.$from.parent.type.name).toBe("paragraph");

    editor.commands.undo();
    expect(
      editor.getJSON().content?.some((node) => node.type === "image"),
    ).toBe(false);
    editor.commands.redo();
    expect(
      editor.getJSON().content?.some((node) => node.type === "image"),
    ).toBe(true);
  });

  it("does not mutate the document when command attributes are invalid", () => {
    const editor = createEditor({
      type: "doc",
      content: [{ type: "paragraph" }],
    });
    const contentBeforeCommand = editor.getJSON();

    expect(editor.commands.insertImage({ assetId: "" as AssetId })).toBe(false);
    expect(editor.getJSON()).toEqual(contentBeforeCommand);
  });
});
