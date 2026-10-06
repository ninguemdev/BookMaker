import { join } from "@tauri-apps/api/path";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { z } from "zod";

import { loadWritingProject, type WritingProject } from "./writingProject";

const createdProjectSchema = z
  .object({
    projectId: z.string().min(1),
    path: z.string().min(1),
    title: z.string(),
    language: z.string().min(1),
  })
  .strict();

export interface CreateProjectRequest {
  parentDirectory: string;
  title: string;
  language: string;
}

export interface ProjectLauncherActions {
  selectNewProjectParent(): Promise<string | null>;
  selectExistingProject(): Promise<string | null>;
  createProject(request: CreateProjectRequest): Promise<WritingProject>;
}

function projectDirectoryName(title: string): string {
  const titleWithoutControlCharacters = Array.from(title.trim(), (character) =>
    character.charCodeAt(0) < 32 ? "-" : character,
  ).join("");
  const safeTitle = titleWithoutControlCharacters
    .replace(/[<>:"/\\|?*]/g, "-")
    .replace(/[. ]+$/g, "")
    .trim();

  return `${safeTitle || "Novo livro"}.bookmaker`;
}

export async function selectNewProjectParent(): Promise<string | null> {
  return open({
    directory: true,
    multiple: false,
    title: "Escolha onde salvar o novo livro",
  });
}

export async function selectExistingProject(): Promise<string | null> {
  return open({
    directory: true,
    multiple: false,
    title: "Escolha uma pasta .bookmaker",
  });
}

export async function createWritingProject({
  parentDirectory,
  title,
  language,
}: CreateProjectRequest): Promise<WritingProject> {
  const destination = await join(parentDirectory, projectDirectoryName(title));
  const response: unknown = await invoke("create_project", {
    input: { destination, title, language },
  });
  const createdProject = createdProjectSchema.parse(response);

  return loadWritingProject(createdProject.path);
}

export const nativeProjectLauncherActions: ProjectLauncherActions = {
  selectNewProjectParent,
  selectExistingProject,
  createProject: createWritingProject,
};
