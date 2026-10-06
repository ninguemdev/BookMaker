import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import type { DocumentId, ProjectId } from "@bookmaker/domain";
import type { EditorContent } from "@bookmaker/editor-core";

import type { WritingProjectGateway } from "../../application/writingProject";
import { WritingWorkspace } from "./WritingWorkspace";

const chapterOneId = "00000000-0000-7000-8000-000000000001" as DocumentId;
const chapterTwoId = "00000000-0000-7000-8000-000000000002" as DocumentId;

const project = {
  projectId: "00000000-0000-7000-8000-000000000000" as ProjectId,
  path: "C:/Livros/Teste.bookmaker",
  title: "Livro persistido",
  language: "pt-BR",
  documents: [
    {
      documentId: chapterOneId,
      parentId: null,
      title: "Capítulo 1",
      position: 0,
    },
    {
      documentId: chapterTwoId,
      parentId: null,
      title: "Capítulo 2",
      position: 1,
    },
  ],
};

const contentByDocument = new Map<DocumentId, EditorContent>([
  [
    chapterOneId,
    {
      type: "doc",
      content: [
        {
          type: "paragraph",
          content: [{ type: "text", text: "Texto persistido do capítulo um." }],
        },
      ],
    },
  ],
  [
    chapterTwoId,
    {
      type: "doc",
      content: [
        {
          type: "paragraph",
          content: [{ type: "text", text: "O reencontro aconteceu aqui." }],
        },
      ],
    },
  ],
]);

function createGateway(): WritingProjectGateway {
  return {
    loadProject: vi.fn(async () => project),
    loadDocument: vi.fn(async (_projectPath, documentId) => ({
      documentId,
      schemaVersion: 1,
      content: contentByDocument.get(documentId)!,
      updatedAt: "2026-10-06T12:00:00.000Z",
    })),
    saveDocument: vi.fn(async (_projectPath, documentId) => ({
      documentId,
      updatedAt: "2026-10-06T12:01:00.000Z",
    })),
  };
}

describe("WritingWorkspace", () => {
  it("loads persisted content and exposes an honest save state", async () => {
    const gateway = createGateway();
    render(<WritingWorkspace gateway={gateway} project={project} />);

    expect(screen.getByText("Carregando documento…")).toBeInTheDocument();
    const editor = await screen.findByRole("textbox", {
      name: "Conteúdo de Capítulo 1",
    });
    expect(editor).toHaveTextContent("Texto persistido do capítulo um.");
    expect(screen.getByText("Livro persistido")).toBeInTheDocument();
    expect(screen.getByText("Sem alterações")).toBeInTheDocument();
    expect(screen.queryByText("Sessão não persistida")).not.toBeInTheDocument();
    expect(gateway.loadDocument).toHaveBeenCalledWith(
      project.path,
      chapterOneId,
    );
  });

  it("explains a document load failure and retries without losing the project", async () => {
    const gateway = createGateway();
    vi.mocked(gateway.loadDocument)
      .mockRejectedValueOnce({ message: "Conteúdo indisponível" })
      .mockResolvedValueOnce({
        documentId: chapterOneId,
        schemaVersion: 1,
        content: contentByDocument.get(chapterOneId)!,
        updatedAt: "2026-10-06T12:00:00.000Z",
      });
    render(<WritingWorkspace gateway={gateway} project={project} />);

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Conteúdo indisponível",
    );
    fireEvent.click(screen.getByRole("button", { name: "Tentar novamente" }));

    expect(
      await screen.findByRole("textbox", { name: "Conteúdo de Capítulo 1" }),
    ).toHaveTextContent("Texto persistido do capítulo um.");
    expect(gateway.loadDocument).toHaveBeenCalledTimes(2);
  });

  it("flushes the current document before loading another one", async () => {
    const gateway = createGateway();
    render(
      <WritingWorkspace
        autosaveDebounceMs={60_000}
        gateway={gateway}
        project={project}
      />,
    );
    await screen.findByRole("textbox", { name: "Conteúdo de Capítulo 1" });
    fireEvent.click(screen.getByRole("button", { name: "Quebra de cena" }));

    fireEvent.click(screen.getByRole("treeitem", { name: "Capítulo 2" }));

    const nextEditor = await screen.findByRole("textbox", {
      name: "Conteúdo de Capítulo 2",
    });
    expect(nextEditor).toHaveTextContent("O reencontro aconteceu aqui.");
    expect(gateway.saveDocument).toHaveBeenCalledOnce();
    expect(gateway.saveDocument).toHaveBeenCalledWith(
      project.path,
      chapterOneId,
      expect.objectContaining({ type: "doc" }),
    );
    const saveOrder = vi.mocked(gateway.saveDocument).mock
      .invocationCallOrder[0]!;
    const secondLoadOrder = vi.mocked(gateway.loadDocument).mock
      .invocationCallOrder[1]!;
    expect(saveOrder).toBeLessThan(secondLoadOrder);
  });

  it("autosaves editor changes after the configured debounce", async () => {
    const gateway = createGateway();
    render(
      <WritingWorkspace
        autosaveDebounceMs={0}
        gateway={gateway}
        project={project}
      />,
    );
    await screen.findByRole("textbox", { name: "Conteúdo de Capítulo 1" });

    fireEvent.click(screen.getByRole("button", { name: "Quebra de cena" }));

    await waitFor(() => expect(gateway.saveDocument).toHaveBeenCalledOnce());
    expect(await screen.findByText(/Salvo às/)).toBeInTheDocument();
  });

  it("does not leave the document when its flush fails and supports retry", async () => {
    const gateway = createGateway();
    vi.mocked(gateway.saveDocument)
      .mockRejectedValueOnce({ message: "Disco indisponível" })
      .mockResolvedValueOnce({
        documentId: chapterOneId,
        updatedAt: "2026-10-06T12:02:00.000Z",
      });
    render(
      <WritingWorkspace
        autosaveDebounceMs={60_000}
        gateway={gateway}
        project={project}
      />,
    );
    await screen.findByRole("textbox", { name: "Conteúdo de Capítulo 1" });
    fireEvent.click(screen.getByRole("button", { name: "Quebra de cena" }));

    fireEvent.click(screen.getByRole("treeitem", { name: "Capítulo 2" }));

    expect(await screen.findByText("Erro ao salvar")).toBeInTheDocument();
    expect(
      screen.getByRole("textbox", { name: "Conteúdo de Capítulo 1" }),
    ).toBeInTheDocument();
    expect(gateway.loadDocument).toHaveBeenCalledTimes(1);

    fireEvent.click(screen.getByRole("button", { name: "Tentar novamente" }));
    await waitFor(() => expect(gateway.saveDocument).toHaveBeenCalledTimes(2));
    expect(await screen.findByText(/Salvo às/)).toBeInTheDocument();
  });

  it("uses persisted project search and navigates to its document", async () => {
    const gateway = createGateway();
    const search = vi.fn(async () => ({
      matches: [
        {
          documentId: chapterTwoId,
          documentTitle: "Capítulo 2",
          snippet: "O reencontro aconteceu aqui.",
          from: 3,
          to: 13,
        },
      ],
      truncated: false,
    }));
    render(
      <WritingWorkspace gateway={gateway} project={project} search={search} />,
    );
    await screen.findByRole("textbox", { name: "Conteúdo de Capítulo 1" });
    fireEvent.click(screen.getByRole("button", { name: "Buscar no projeto" }));
    const searchForm = screen.getByRole("search", {
      name: "Buscar em todos os documentos",
    });
    fireEvent.change(within(searchForm).getByRole("searchbox"), {
      target: { value: "reencontro" },
    });
    fireEvent.submit(searchForm);
    fireEvent.click(
      await screen.findByRole("button", {
        name: "Abrir ocorrência 1 em Capítulo 2",
      }),
    );

    expect(
      await screen.findByRole("textbox", { name: "Conteúdo de Capítulo 2" }),
    ).toBeInTheDocument();
    expect(search).toHaveBeenCalledWith({
      projectPath: project.path,
      query: "reencontro",
    });
  });

  it("removes peripheral workspace regions in focus mode", async () => {
    render(<WritingWorkspace gateway={createGateway()} project={project} />);
    await screen.findByRole("textbox", { name: "Conteúdo de Capítulo 1" });

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

  it("flushes pending changes before closing the project", async () => {
    const gateway = createGateway();
    const onCloseProject = vi.fn();
    render(
      <WritingWorkspace
        autosaveDebounceMs={60_000}
        gateway={gateway}
        onCloseProject={onCloseProject}
        project={project}
      />,
    );
    await screen.findByRole("textbox", { name: "Conteúdo de Capítulo 1" });
    fireEvent.click(screen.getByRole("button", { name: "Quebra de cena" }));

    fireEvent.click(screen.getByRole("button", { name: "Fechar projeto" }));

    await waitFor(() => expect(onCloseProject).toHaveBeenCalledOnce());
    expect(gateway.saveDocument).toHaveBeenCalledOnce();
  });
});
