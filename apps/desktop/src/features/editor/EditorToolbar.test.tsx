import { fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";

import { createEditorExtensions } from "@bookmaker/editor-core";
import { Editor } from "@tiptap/core";

import { EditorToolbar } from "./EditorToolbar";

const editors: Editor[] = [];

function createEditor(): Editor {
  const editor = new Editor({
    content: {
      type: "doc",
      content: [
        {
          type: "paragraph",
          content: [{ type: "text", text: "Texto inicial" }],
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

describe("EditorToolbar", () => {
  it("exposes accessible groups and disables actions without an editor", () => {
    render(<EditorToolbar editor={null} />);

    expect(
      screen.getByRole("toolbar", { name: "Formatação do texto" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("group", { name: "Estilo do bloco" }),
    ).toBeInTheDocument();
    expect(screen.getByRole("group", { name: "Ênfase" })).toBeInTheDocument();
    expect(screen.getByRole("group", { name: "Listas" })).toBeInTheDocument();
    expect(
      screen.getByRole("group", { name: "Histórico" }),
    ).toBeInTheDocument();

    for (const button of screen.getAllByRole("button")) {
      expect(button).toBeDisabled();
    }
  });

  it("applies inline formatting and reflects the active selection", () => {
    const editor = createEditor();
    editor.commands.setTextSelection({ from: 1, to: 6 });
    render(<EditorToolbar editor={editor} />);

    fireEvent.click(screen.getByRole("button", { name: "Negrito" }));

    expect(editor.isActive("bold")).toBe(true);
    expect(screen.getByRole("button", { name: "Negrito" })).toHaveAttribute(
      "aria-pressed",
      "true",
    );
    expect(editor.getJSON().content?.[0]?.content?.[0]?.marks).toEqual([
      { type: "bold" },
    ]);
  });

  it("changes block style, creates lists, and supports undo and redo", () => {
    const editor = createEditor();
    editor.commands.setTextSelection(2);
    render(<EditorToolbar editor={editor} />);

    fireEvent.click(screen.getByRole("button", { name: "Título 2" }));
    expect(editor.getJSON().content?.[0]).toMatchObject({
      type: "heading",
      attrs: { level: 2 },
    });

    fireEvent.click(screen.getByRole("button", { name: "Lista" }));
    expect(editor.isActive("bulletList")).toBe(true);

    fireEvent.click(screen.getByRole("button", { name: "Desfazer" }));
    expect(editor.isActive("bulletList")).toBe(false);

    fireEvent.click(screen.getByRole("button", { name: "Refazer" }));
    expect(editor.isActive("bulletList")).toBe(true);
  });

  it("validates, applies, and removes links", () => {
    const editor = createEditor();
    editor.commands.setTextSelection({ from: 1, to: 6 });
    render(<EditorToolbar editor={editor} />);

    fireEvent.click(screen.getByRole("button", { name: "Link" }));
    const input = screen.getByRole("textbox", { name: "Endereço do link" });
    const form = input.closest("form");
    expect(form).not.toBeNull();

    fireEvent.change(input, { target: { value: "javascript:alert(1)" } });
    fireEvent.submit(form!);
    expect(screen.getByRole("alert")).toHaveTextContent(
      "Informe um endereço seguro e válido.",
    );
    expect(editor.isActive("link")).toBe(false);

    fireEvent.change(input, { target: { value: "https://example.com" } });
    fireEvent.submit(form!);
    expect(editor.isActive("link")).toBe(true);

    fireEvent.click(screen.getByRole("button", { name: "Link" }));
    fireEvent.click(screen.getByRole("button", { name: "Remover" }));
    expect(editor.isActive("link")).toBe(false);
  });
});
