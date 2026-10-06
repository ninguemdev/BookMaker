import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import type { ProjectId } from "@bookmaker/domain";

import type { WritingProjectGateway } from "./application/writingProject";
import App from "./App";

describe("App", () => {
  it("opens a real project from its local path", async () => {
    const project = {
      projectId: "00000000-0000-7000-8000-000000000000" as ProjectId,
      path: "C:/Livros/Teste.bookmaker",
      title: "Livro vazio",
      language: "pt-BR",
      documents: [],
    };
    const gateway: WritingProjectGateway = {
      loadProject: vi.fn(async () => project),
      loadDocument: vi.fn(),
      saveDocument: vi.fn(),
    };
    render(<App gateway={gateway} />);
    fireEvent.change(screen.getByLabelText("Caminho da pasta .bookmaker"), {
      target: { value: project.path },
    });

    fireEvent.click(screen.getByRole("button", { name: "Abrir projeto" }));

    expect(
      await screen.findByRole("heading", { name: "Livro vazio" }),
    ).toBeInTheDocument();
    expect(
      screen.getByText("Nenhum documento disponível neste projeto."),
    ).toBeInTheDocument();
    expect(gateway.loadProject).toHaveBeenCalledWith(project.path);
  });

  it("keeps the launcher open and explains project opening failures", async () => {
    const gateway: WritingProjectGateway = {
      loadProject: vi.fn(async () =>
        Promise.reject({ message: "Escolha uma pasta de projeto válida." }),
      ),
      loadDocument: vi.fn(),
      saveDocument: vi.fn(),
    };
    render(<App gateway={gateway} />);
    fireEvent.change(screen.getByLabelText("Caminho da pasta .bookmaker"), {
      target: { value: "C:/Livros/Inexistente.bookmaker" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Abrir projeto" }));

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Escolha uma pasta de projeto válida.",
    );
    expect(screen.getByRole("button", { name: "Abrir projeto" })).toBeEnabled();
  });
});
