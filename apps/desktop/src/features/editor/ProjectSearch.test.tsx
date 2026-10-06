import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import type { DocumentId } from "@bookmaker/domain";

import type { ProjectSearchResults } from "../../application/projectSearch";
import { ProjectSearch } from "./ProjectSearch";

const results: ProjectSearchResults = {
  matches: [
    {
      documentId: "document-1" as DocumentId,
      documentTitle: "Capítulo um",
      snippet: "Um termo encontrado no texto.",
      from: 4,
      to: 9,
    },
    {
      documentId: "document-2" as DocumentId,
      documentTitle: "Capítulo dois",
      snippet: "Outro termo aparece aqui.",
      from: 12,
      to: 17,
    },
  ],
  truncated: false,
};

describe("ProjectSearch", () => {
  it("disables search without an open project", () => {
    render(<ProjectSearch onNavigate={vi.fn()} projectPath={null} />);

    expect(
      screen.getByRole("button", { name: "Buscar no projeto" }),
    ).toBeDisabled();
  });

  it("opens with Ctrl+Shift+F and focuses the query", () => {
    render(
      <ProjectSearch
        onNavigate={vi.fn()}
        projectPath="C:/Livros/Teste.bookmaker"
      />,
    );

    fireEvent.keyDown(window, { ctrlKey: true, shiftKey: true, key: "f" });

    expect(
      screen.getByRole("search", { name: "Buscar em todos os documentos" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("searchbox", { name: "Buscar no projeto" }),
    ).toHaveFocus();
  });

  it("submits a normalized query and renders project results", async () => {
    const search = vi.fn().mockResolvedValue(results);
    render(
      <ProjectSearch
        onNavigate={vi.fn()}
        projectPath="C:/Livros/Teste.bookmaker"
        search={search}
      />,
    );
    fireEvent.click(screen.getByRole("button", { name: "Buscar no projeto" }));
    const panel = screen.getByRole("search", {
      name: "Buscar em todos os documentos",
    });
    fireEvent.change(within(panel).getByRole("searchbox"), {
      target: { value: "  termo  " },
    });
    fireEvent.submit(panel);

    expect(await screen.findByText("2 resultados")).toBeInTheDocument();
    expect(search).toHaveBeenCalledWith({
      projectPath: "C:/Livros/Teste.bookmaker",
      query: "termo",
    });
    expect(screen.getByText("Capítulo um")).toBeInTheDocument();
    expect(screen.getByText("Outro termo aparece aqui.")).toBeInTheDocument();
  });

  it("navigates directly to the selected document occurrence", async () => {
    const onNavigate = vi.fn();
    render(
      <ProjectSearch
        onNavigate={onNavigate}
        projectPath="C:/Livros/Teste.bookmaker"
        search={vi.fn().mockResolvedValue(results)}
      />,
    );
    fireEvent.click(screen.getByRole("button", { name: "Buscar no projeto" }));
    const panel = screen.getByRole("search");
    fireEvent.change(within(panel).getByRole("searchbox"), {
      target: { value: "termo" },
    });
    fireEvent.submit(panel);
    const occurrence = await screen.findByRole("button", {
      name: "Abrir ocorrência 2 em Capítulo dois",
    });

    fireEvent.click(occurrence);

    expect(onNavigate).toHaveBeenCalledWith(results.matches[1]);
    expect(screen.queryByRole("search")).not.toBeInTheDocument();
  });

  it("shows native failures without discarding the query", async () => {
    render(
      <ProjectSearch
        onNavigate={vi.fn()}
        projectPath="C:/Livros/Teste.bookmaker"
        search={vi.fn().mockRejectedValue({ message: "Projeto indisponível." })}
      />,
    );
    fireEvent.click(screen.getByRole("button", { name: "Buscar no projeto" }));
    const panel = screen.getByRole("search");
    const input = within(panel).getByRole("searchbox");
    fireEvent.change(input, { target: { value: "termo" } });
    fireEvent.submit(panel);

    await waitFor(() => {
      expect(screen.getByRole("alert")).toHaveTextContent(
        "Projeto indisponível.",
      );
    });
    expect(input).toHaveValue("termo");
  });
});
