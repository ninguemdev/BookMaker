import { act, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";

import { createEditorExtensions } from "@bookmaker/editor-core";
import { Editor } from "@tiptap/core";

import { EditorFind } from "./EditorFind";

const editors: Editor[] = [];

function createEditor(text = "Um termo, outro TERMO e o fim."): Editor {
  const editor = new Editor({
    content: `<p>${text}</p>`,
    extensions: createEditorExtensions(),
  });
  editors.push(editor);
  return editor;
}

function selectedText(editor: Editor): string {
  const { from, to } = editor.state.selection;
  return editor.state.doc.textBetween(from, to);
}

afterEach(() => {
  editors.splice(0).forEach((editor) => editor.destroy());
});

describe("EditorFind", () => {
  it("disables document search without an editor", () => {
    render(<EditorFind editor={null} />);

    expect(screen.getByRole("button", { name: "Buscar" })).toBeDisabled();
  });

  it("opens with Ctrl+F and exposes an accessible search panel", () => {
    render(<EditorFind editor={createEditor()} />);

    fireEvent.keyDown(window, { ctrlKey: true, key: "f" });

    expect(
      screen.getByRole("search", { name: "Buscar no documento" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("searchbox", { name: "Buscar no documento" }),
    ).toHaveFocus();
  });

  it("selects matches and navigates through them with wraparound", () => {
    const editor = createEditor();
    render(<EditorFind editor={editor} />);
    fireEvent.click(screen.getByRole("button", { name: "Buscar" }));

    fireEvent.change(
      screen.getByRole("searchbox", { name: "Buscar no documento" }),
      { target: { value: "termo" } },
    );

    expect(screen.getByRole("status")).toHaveTextContent("1 de 2");
    expect(selectedText(editor)).toBe("termo");

    fireEvent.click(screen.getByRole("button", { name: "Próxima ocorrência" }));
    expect(screen.getByRole("status")).toHaveTextContent("2 de 2");
    expect(selectedText(editor)).toBe("TERMO");

    fireEvent.click(screen.getByRole("button", { name: "Próxima ocorrência" }));
    expect(screen.getByRole("status")).toHaveTextContent("1 de 2");
  });

  it("uses Shift+Enter for the previous occurrence and Escape to close", () => {
    const editor = createEditor();
    render(<EditorFind editor={editor} />);
    fireEvent.click(screen.getByRole("button", { name: "Buscar" }));
    const input = screen.getByRole("searchbox", {
      name: "Buscar no documento",
    });
    fireEvent.change(input, { target: { value: "termo" } });

    fireEvent.keyDown(input, { key: "Enter", shiftKey: true });
    expect(screen.getByRole("status")).toHaveTextContent("2 de 2");

    fireEvent.keyDown(input, { key: "Escape" });
    expect(screen.queryByRole("search")).not.toBeInTheDocument();
  });

  it("refreshes results after document content changes", () => {
    const editor = createEditor("Um termo.");
    render(<EditorFind editor={editor} />);
    fireEvent.click(screen.getByRole("button", { name: "Buscar" }));
    fireEvent.change(
      screen.getByRole("searchbox", { name: "Buscar no documento" }),
      { target: { value: "termo" } },
    );
    expect(screen.getByRole("status")).toHaveTextContent("1 de 1");

    act(() => {
      editor.commands.setContent("<p>Sem a palavra procurada.</p>");
    });

    expect(screen.getByRole("status")).toHaveTextContent("Nenhum resultado");
  });
});
