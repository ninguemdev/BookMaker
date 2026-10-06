// @vitest-environment jsdom

import { afterEach, describe, expect, it } from "vitest";

import { Editor } from "@tiptap/core";

import {
  createEditorExtensions,
  isEditorContent,
  parseEditorContent,
} from "./editorSchema";

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

describe("SceneBreak", () => {
  it("round-trips the semantic node through JSON", () => {
    const content = {
      type: "doc",
      content: [
        { type: "paragraph", content: [{ type: "text", text: "Antes" }] },
        { type: "sceneBreak" },
        { type: "paragraph", content: [{ type: "text", text: "Depois" }] },
      ],
    };

    expect(parseEditorContent(content)).toEqual(content);
  });

  it.each([
    ["attributes", { type: "sceneBreak", attrs: { ornament: "asterisks" } }],
    [
      "children",
      {
        type: "sceneBreak",
        content: [{ type: "text", text: "***" }],
      },
    ],
    ["marks", { type: "sceneBreak", marks: [{ type: "bold" }] }],
  ])("rejects scene breaks with %s", (_case, sceneBreak) => {
    expect(
      isEditorContent({
        type: "doc",
        content: [sceneBreak],
      }),
    ).toBe(false);
  });

  it("uses an explicit HTML marker and does not treat a generic rule as a scene break", () => {
    const editor = createEditor(
      '<p>Antes</p><hr data-scene-break=""><p>Depois</p>',
    );
    expect(editor.getJSON().content?.map((node) => node.type)).toEqual([
      "paragraph",
      "sceneBreak",
      "paragraph",
    ]);
    expect(editor.getHTML()).toContain('<hr data-scene-break="">');

    editor.commands.setContent("<p>Antes</p><hr><p>Depois</p>");
    expect(editor.getJSON().content?.map((node) => node.type)).toEqual([
      "paragraph",
      "paragraph",
    ]);
  });

  it("inserts a paragraph after a scene break at the end of a document", () => {
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

    expect(editor.commands.insertSceneBreak()).toBe(true);
    expect(editor.getJSON().content?.map((node) => node.type)).toEqual([
      "paragraph",
      "sceneBreak",
      "paragraph",
    ]);
    expect(editor.state.selection.$from.parent.type.name).toBe("paragraph");
  });

  it("splits a paragraph and participates in undo and redo", () => {
    const editor = createEditor({
      type: "doc",
      content: [
        {
          type: "paragraph",
          content: [{ type: "text", text: "Antes" }],
        },
      ],
    });
    editor.commands.setTextSelection(3);

    editor.commands.insertSceneBreak();
    expect(editor.getJSON()).toEqual({
      type: "doc",
      content: [
        { type: "paragraph", content: [{ type: "text", text: "An" }] },
        { type: "sceneBreak" },
        { type: "paragraph", content: [{ type: "text", text: "tes" }] },
      ],
    });

    editor.commands.undo();
    expect(editor.getText()).toBe("Antes");
    expect(
      editor.getJSON().content?.some((node) => node.type === "sceneBreak"),
    ).toBe(false);

    editor.commands.redo();
    expect(
      editor.getJSON().content?.some((node) => node.type === "sceneBreak"),
    ).toBe(true);
  });
});
