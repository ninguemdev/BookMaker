import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import type { ProjectId } from "@bookmaker/domain";

import type { ProjectLauncherActions } from "../../application/projectLauncher";
import type {
  WritingProject,
  WritingProjectGateway,
} from "../../application/writingProject";
import { ProjectLauncher } from "./ProjectLauncher";

const project: WritingProject = {
  projectId: "00000000-0000-7000-8000-000000000000" as ProjectId,
  path: "C:/Livros/Meu livro.bookmaker",
  title: "Meu livro",
  language: "pt-BR",
  documents: [],
};

function createGateway(): WritingProjectGateway {
  return {
    loadProject: vi.fn(async () => project),
    loadDocument: vi.fn(),
    saveDocument: vi.fn(),
  };
}

function createLauncherActions(): ProjectLauncherActions {
  return {
    selectNewProjectParent: vi.fn(async () => "C:/Livros"),
    selectExistingProject: vi.fn(async () => project.path),
    createProject: vi.fn(async () => project),
  };
}

describe("ProjectLauncher", () => {
  it("creates the first .bookmaker project inside a folder selected by the user", async () => {
    const launcherActions = createLauncherActions();
    const onOpen = vi.fn();
    render(
      <ProjectLauncher
        gateway={createGateway()}
        launcherActions={launcherActions}
        onOpen={onOpen}
      />,
    );

    fireEvent.change(screen.getByLabelText("Título do livro"), {
      target: { value: "  Meu livro  " },
    });
    fireEvent.click(screen.getByRole("button", { name: "Escolher pasta…" }));

    expect(await screen.findByDisplayValue("C:/Livros")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Criar livro" }));

    expect(launcherActions.createProject).toHaveBeenCalledWith({
      parentDirectory: "C:/Livros",
      title: "Meu livro",
      language: "pt-BR",
    });
    await waitFor(() => expect(onOpen).toHaveBeenCalledWith(project));
  });

  it("selects an existing .bookmaker folder before opening it", async () => {
    const gateway = createGateway();
    const launcherActions = createLauncherActions();
    render(
      <ProjectLauncher
        gateway={gateway}
        launcherActions={launcherActions}
        onOpen={vi.fn()}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Procurar…" }));

    expect(await screen.findByDisplayValue(project.path)).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Abrir projeto" }));
    expect(gateway.loadProject).toHaveBeenCalledWith(project.path);
  });

  it("keeps the selected path unchanged when the folder dialog is canceled", async () => {
    const launcherActions = createLauncherActions();
    vi.mocked(launcherActions.selectNewProjectParent).mockResolvedValue(null);
    render(
      <ProjectLauncher
        gateway={createGateway()}
        launcherActions={launcherActions}
        onOpen={vi.fn()}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Escolher pasta…" }));

    expect(
      await screen.findByPlaceholderText("Escolha uma pasta no seu computador"),
    ).toHaveValue("");
  });
});
