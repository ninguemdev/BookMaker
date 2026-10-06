import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { DocumentTree } from "./DocumentTree";
import type { DocumentTreeNode } from "./DocumentTree";

const nodes: readonly DocumentTreeNode[] = [
  {
    id: "part-1",
    title: "Parte I",
    children: [
      { id: "chapter-1", title: "Capítulo 1" },
      { id: "chapter-2", title: "Capítulo 2" },
    ],
  },
  { id: "appendix", title: "Apêndice" },
];

describe("DocumentTree", () => {
  it("moves focus through visible items with arrows, Home and End", () => {
    render(<DocumentTree nodes={nodes} onSelect={vi.fn()} />);
    const part = screen.getByRole("treeitem", { name: "Parte I" });
    const appendix = screen.getByRole("treeitem", { name: "Apêndice" });

    expect(part).toHaveAttribute("tabindex", "0");
    expect(appendix).toHaveAttribute("tabindex", "-1");
    part.focus();

    fireEvent.keyDown(part, { key: "ArrowDown" });
    expect(appendix).toHaveFocus();
    expect(appendix).toHaveAttribute("tabindex", "0");

    fireEvent.keyDown(appendix, { key: "ArrowUp" });
    expect(part).toHaveFocus();

    fireEvent.keyDown(part, { key: "End" });
    expect(appendix).toHaveFocus();

    fireEvent.keyDown(appendix, { key: "Home" });
    expect(part).toHaveFocus();
  });

  it("expands, enters, leaves and collapses branches with horizontal arrows", () => {
    render(<DocumentTree nodes={nodes} onSelect={vi.fn()} />);
    const part = screen.getByRole("treeitem", { name: "Parte I" });
    part.focus();

    expect(part).toHaveAttribute("aria-expanded", "false");
    fireEvent.keyDown(part, { key: "ArrowRight" });
    expect(part).toHaveAttribute("aria-expanded", "true");

    fireEvent.keyDown(part, { key: "ArrowRight" });
    const firstChapter = screen.getByRole("treeitem", {
      name: "Capítulo 1",
    });
    expect(firstChapter).toHaveFocus();

    fireEvent.keyDown(firstChapter, { key: "ArrowLeft" });
    expect(part).toHaveFocus();

    fireEvent.keyDown(part, { key: "ArrowLeft" });
    expect(part).toHaveAttribute("aria-expanded", "false");
    expect(
      screen.queryByRole("treeitem", { name: "Capítulo 1" }),
    ).not.toBeInTheDocument();
  });

  it("activates the focused item with Enter, Space or click", () => {
    const onSelect = vi.fn();
    const { rerender } = render(
      <DocumentTree nodes={nodes} selectedId="part-1" onSelect={onSelect} />,
    );
    const part = screen.getByRole("treeitem", { name: "Parte I" });
    const appendix = screen.getByRole("treeitem", { name: "Apêndice" });

    expect(part).toHaveAttribute("aria-selected", "true");
    fireEvent.keyDown(part, { key: "Enter" });
    fireEvent.keyDown(part, { key: " " });
    fireEvent.click(appendix);

    expect(onSelect).toHaveBeenNthCalledWith(1, "part-1");
    expect(onSelect).toHaveBeenNthCalledWith(2, "part-1");
    expect(onSelect).toHaveBeenNthCalledWith(3, "appendix");
    expect(appendix).toHaveFocus();

    rerender(
      <DocumentTree nodes={nodes} selectedId="appendix" onSelect={onSelect} />,
    );
    expect(appendix).toHaveAttribute("aria-selected", "true");
  });

  it("keeps an accessible empty tree without a keyboard stop", () => {
    render(<DocumentTree nodes={[]} onSelect={vi.fn()} />);

    expect(
      screen.getByRole("tree", { name: "Estrutura do livro" }),
    ).toBeInTheDocument();
    expect(screen.queryByRole("treeitem")).not.toBeInTheDocument();
    expect(
      screen.getByText("Nenhum documento neste projeto."),
    ).toBeInTheDocument();
  });
});
