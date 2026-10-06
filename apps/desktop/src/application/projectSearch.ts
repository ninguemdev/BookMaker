import { invoke } from "@tauri-apps/api/core";

import type { DocumentId } from "@bookmaker/domain";

export interface ProjectSearchInput {
  projectPath: string;
  query: string;
}

export interface ProjectSearchMatch {
  documentId: DocumentId;
  documentTitle: string;
  snippet: string;
  from: number;
  to: number;
}

export interface ProjectSearchResults {
  matches: ProjectSearchMatch[];
  truncated: boolean;
}

export class ProjectSearchProtocolError extends Error {
  public constructor() {
    super("Native project search returned an invalid response");
    this.name = "ProjectSearchProtocolError";
  }
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function parseMatch(value: unknown): ProjectSearchMatch {
  if (!isRecord(value)) {
    throw new ProjectSearchProtocolError();
  }

  const { documentId, documentTitle, snippet, from, to } = value;
  if (
    typeof documentId !== "string" ||
    documentId.length === 0 ||
    typeof documentTitle !== "string" ||
    typeof snippet !== "string" ||
    !Number.isSafeInteger(from) ||
    !Number.isSafeInteger(to) ||
    (from as number) < 0 ||
    (to as number) < (from as number)
  ) {
    throw new ProjectSearchProtocolError();
  }

  return {
    documentId: documentId as DocumentId,
    documentTitle,
    snippet,
    from: from as number,
    to: to as number,
  };
}

function parseResults(value: unknown): ProjectSearchResults {
  if (
    !isRecord(value) ||
    !Array.isArray(value.matches) ||
    typeof value.truncated !== "boolean"
  ) {
    throw new ProjectSearchProtocolError();
  }

  return {
    matches: value.matches.map(parseMatch),
    truncated: value.truncated,
  };
}

export async function searchProject(
  input: ProjectSearchInput,
): Promise<ProjectSearchResults> {
  const response: unknown = await invoke("search_project", { input });
  return parseResults(response);
}
