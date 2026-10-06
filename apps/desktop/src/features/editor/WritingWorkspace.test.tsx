import { fireEvent, render, screen, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { WritingWorkspace } from "./WritingWorkspace";

describe("WritingWorkspace", () => {
  it("mounts the writing interface with a real editable document", () => {
    render(<WritingWorkspace />);

    expect(
      screen.getByRole("heading", { level: 1, name: "BookMaker" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { level: 2, name: "Capítulo 1" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("textbox", { name: "Conteúdo de Capítulo 1" }),
    ).toHaveAttribute("contenteditable", "true");
    expect(screen.getByLabelText("Contagem de palavras")).toHaveTextContent(
      "23 palavras",
    );
    expect(screen.getByText("Sessão não persistida")).toBeInTheDocument();
    expect(
      screen.getByText("Alterações mantidas somente nesta sessão"),
    ).toBeInTheDocument();
  });

  it("switches the editable document from the manuscript tree", () => {
    render(<WritingWorkspace />);

    fireEvent.click(screen.getByRole("treeitem", { name: "Capítulo 2" }));

    expect(
      screen.getByRole("heading", { level: 2, name: "Capítulo 2" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("textbox", { name: "Conteúdo de Capítulo 2" }),
    ).toHaveTextContent(
      "O reencontro aconteceu na plataforma vazia da estação.",
    );
  });

  it("connects toolbar commands to the mounted editor", () => {
    render(<WritingWorkspace />);

    fireEvent.click(screen.getByRole("button", { name: "Quebra de cena" }));

    const editor = screen.getByRole("textbox", {
      name: "Conteúdo de Capítulo 1",
    });
    expect(editor.querySelector("hr[data-scene-break]")).not.toBeNull();
    expect(screen.getByLabelText("Contagem de palavras")).toHaveTextContent(
      "23 palavras",
    );
  });

  it("searches the session and navigates directly to another document", async () => {
    render(<WritingWorkspace />);
    fireEvent.click(screen.getByRole("button", { name: "Buscar no projeto" }));
    const search = screen.getByRole("search", {
      name: "Buscar em todos os documentos",
    });
    fireEvent.change(within(search).getByRole("searchbox"), {
      target: { value: "reencontro" },
    });
    fireEvent.submit(search);
    const result = await screen.findByRole("button", {
      name: "Abrir ocorrência 1 em Capítulo 2",
    });

    fireEvent.click(result);

    expect(
      screen.getByRole("heading", { level: 2, name: "Capítulo 2" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("treeitem", { name: "Capítulo 2" }),
    ).toHaveAttribute("aria-selected", "true");
  });

  it("removes peripheral workspace regions in focus mode", () => {
    render(<WritingWorkspace />);

    fireEvent.click(
      screen.getByRole("button", { name: "Entrar no modo foco" }),
    );

    expect(
      screen.queryByRole("tree", { name: "Estrutura do livro" }),
    ).not.toBeInTheDocument();
    expect(
      screen.queryByRole("complementary", { name: "Ajuda de escrita" }),
    ).not.toBeInTheDocument();
    expect(
      screen.getByRole("textbox", { name: "Conteúdo de Capítulo 1" }),
    ).toBeVisible();
  });
});
