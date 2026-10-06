import { invoke } from "@tauri-apps/api/core";
import { join } from "@tauri-apps/api/path";
import { open } from "@tauri-apps/plugin-dialog";
import { beforeEach, describe, expect, it, vi } from "vitest";

import {
  createWritingProject,
  selectExistingProject,
  selectNewProjectParent,
} from "./projectLauncher";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/path", () => ({ join: vi.fn() }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn() }));

const createdProject = {
  projectId: "00000000-0000-7000-8000-000000000000",
  path: "C:/Livros/Meu- livro-.bookmaker",
  title: "Meu: livro?",
  language: "pt-BR",
};

const writingProject = {
  ...createdProject,
  documents: [],
};

describe("project launcher application actions", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("creates a new .bookmaker directory below the selected parent", async () => {
    vi.mocked(join).mockResolvedValue(createdProject.path);
    vi.mocked(invoke)
      .mockResolvedValueOnce(createdProject)
      .mockResolvedValueOnce(writingProject);

    await expect(
      createWritingProject({
        parentDirectory: "C:/Livros",
        title: "Meu: livro?",
        language: "pt-BR",
      }),
    ).resolves.toEqual(writingProject);

    expect(join).toHaveBeenCalledWith("C:/Livros", "Meu- livro-.bookmaker");
    expect(invoke).toHaveBeenNthCalledWith(1, "create_project", {
      input: {
        destination: createdProject.path,
        title: "Meu: livro?",
        language: "pt-BR",
      },
    });
    expect(invoke).toHaveBeenNthCalledWith(2, "load_writing_project", {
      input: { projectPath: createdProject.path },
    });
  });

  it("opens native directory pickers for new and existing projects", async () => {
    vi.mocked(open)
      .mockResolvedValueOnce("C:/Livros")
      .mockResolvedValueOnce(createdProject.path);

    await expect(selectNewProjectParent()).resolves.toBe("C:/Livros");
    await expect(selectExistingProject()).resolves.toBe(createdProject.path);

    expect(open).toHaveBeenNthCalledWith(1, {
      directory: true,
      multiple: false,
      title: "Escolha onde salvar o novo livro",
    });
    expect(open).toHaveBeenNthCalledWith(2, {
      directory: true,
      multiple: false,
      title: "Escolha uma pasta .bookmaker",
    });
  });
});
