import { fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import { createEditorExtensions } from "@bookmaker/editor-core";
import { Editor } from "@tiptap/core";

import { EditorFocusMode, FocusModeDistraction } from "./EditorFocusMode";

const editors: Editor[] = [];

function createEditor(): Editor {
  const editor = new Editor({
    content: "<p>Texto em edição.</p>",
    extensions: createEditorExtensions(),
  });
  editors.push(editor);
  return editor;
}

function renderFocusMode(editor: Editor | null, onActiveChange = vi.fn()) {
  return render(
    <EditorFocusMode editor={editor} onActiveChange={onActiveChange}>
      <FocusModeDistraction data-testid="sidebar">
        <button type="button">Ação da lateral</button>
      </FocusModeDistraction>
      <div data-testid="editor-surface">Editor</div>
      <FocusModeDistraction data-testid="inspector">
        Inspetor
      </FocusModeDistraction>
    </EditorFocusMode>,
  );
}

afterEach(() => {
  editors.splice(0).forEach((editor) => editor.destroy());
});

describe("EditorFocusMode", () => {
  it("disables focus mode without an editor", () => {
    renderFocusMode(null);

    expect(
      screen.getByRole("button", { name: "Entrar no modo foco" }),
    ).toBeDisabled();
  });

  it("hides distractions while preserving the editor surface", () => {
    const onActiveChange = vi.fn();
    renderFocusMode(createEditor(), onActiveChange);

    fireEvent.click(
      screen.getByRole("button", { name: "Entrar no modo foco" }),
    );

    expect(screen.getByLabelText("Área de escrita")).toHaveAttribute(
      "data-focus-mode",
      "active",
    );
    expect(screen.getByTestId("sidebar")).toHaveAttribute("hidden");
    expect(screen.getByTestId("inspector")).toHaveAttribute("hidden");
    expect(screen.getByTestId("editor-surface")).toBeVisible();
    expect(screen.getByRole("status")).toHaveTextContent("Modo foco ativado");
    expect(onActiveChange).toHaveBeenCalledWith(true);
  });

  it("toggles with Ctrl+Shift+Enter and exits with Escape", () => {
    renderFocusMode(createEditor());

    fireEvent.keyDown(window, {
      ctrlKey: true,
      shiftKey: true,
      key: "Enter",
    });
    expect(
      screen.getByRole("button", { name: "Sair do modo foco" }),
    ).toHaveAttribute("aria-pressed", "true");

    fireEvent.keyDown(window, { key: "Escape" });
    expect(
      screen.getByRole("button", { name: "Entrar no modo foco" }),
    ).toHaveAttribute("aria-pressed", "false");
    expect(screen.getByRole("status")).toHaveTextContent(
      "Modo foco desativado",
    );
  });

  it("does not consume Escape already handled by a nested control", () => {
    renderFocusMode(createEditor());
    fireEvent.click(
      screen.getByRole("button", { name: "Entrar no modo foco" }),
    );
    const handledEscape = new KeyboardEvent("keydown", {
      bubbles: true,
      cancelable: true,
      key: "Escape",
    });
    handledEscape.preventDefault();

    window.dispatchEvent(handledEscape);

    expect(
      screen.getByRole("button", { name: "Sair do modo foco" }),
    ).toHaveAttribute("aria-pressed", "true");
  });

  it("keeps focus mode active when the current document editor changes", () => {
    const firstEditor = createEditor();
    const secondEditor = createEditor();
    const view = renderFocusMode(firstEditor);
    fireEvent.click(
      screen.getByRole("button", { name: "Entrar no modo foco" }),
    );

    view.rerender(
      <EditorFocusMode editor={secondEditor}>
        <FocusModeDistraction data-testid="sidebar">
          Lateral
        </FocusModeDistraction>
        <div data-testid="editor-surface">Editor</div>
      </EditorFocusMode>,
    );

    expect(screen.getByLabelText("Área de escrita")).toHaveAttribute(
      "data-focus-mode",
      "active",
    );
    expect(screen.getByTestId("sidebar")).toHaveAttribute("hidden");
  });
});
