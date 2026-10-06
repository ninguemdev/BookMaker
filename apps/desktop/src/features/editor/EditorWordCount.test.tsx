import { act, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";

import { createEditorExtensions } from "@bookmaker/editor-core";
import { Editor } from "@tiptap/core";

import { EditorWordCount } from "./EditorWordCount";

const editors: Editor[] = [];

function createEditor(text: string): Editor {
  const editor = new Editor({
    content: {
      type: "doc",
      content: [
        {
          type: "paragraph",
          content: text.length === 0 ? undefined : [{ type: "text", text }],
        },
      ],
    },
    extensions: createEditorExtensions(),
  });
  editors.push(editor);
  return editor;
}

afterEach(() => {
  editors.splice(0).forEach((editor) => editor.destroy());
});

describe("EditorWordCount", () => {
  it("shows an accessible empty count without an editor", () => {
    render(<EditorWordCount editor={null} />);

    expect(
      screen.getByRole("status", { name: "Contagem de palavras" }),
    ).toHaveTextContent("0 palavras");
  });

  it("uses singular for one word", () => {
    render(<EditorWordCount editor={createEditor("Olá")} />);

    expect(
      screen.getByRole("status", { name: "Contagem de palavras" }),
    ).toHaveTextContent("1 palavra");
  });

  it("updates after the editor content changes", () => {
    const editor = createEditor("Duas palavras");
    render(<EditorWordCount editor={editor} />);

    expect(
      screen.getByRole("status", { name: "Contagem de palavras" }),
    ).toHaveTextContent("2 palavras");

    act(() => {
      editor.commands.setContent("<p>Agora são quatro palavras.</p>");
    });

    expect(
      screen.getByRole("status", { name: "Contagem de palavras" }),
    ).toHaveTextContent("4 palavras");
  });
});
