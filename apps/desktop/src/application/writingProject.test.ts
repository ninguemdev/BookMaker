import type { DocumentId } from "@bookmaker/domain";
import { beforeEach, describe, expect, it, vi } from "vitest";

import {
  loadDocumentContent,
  loadWritingProject,
  saveDocumentContent,
  WritingProjectProtocolError,
} from "./writingProject";

const invoke = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({ invoke }));

const documentId = "00000000-0000-7000-8000-000000000001" as DocumentId;

describe("writing project application boundary", () => {
  beforeEach(() => invoke.mockReset());

  it("loads and validates the project tree", async () => {
    invoke.mockResolvedValue({
      projectId: "00000000-0000-7000-8000-000000000000",
      path: "C:/Livros/Teste.bookmaker",
      title: "Teste",
      language: "pt-BR",
      documents: [
        {
          documentId,
          parentId: null,
          title: "Capítulo",
          position: 0,
        },
      ],
    });

    await expect(
      loadWritingProject("C:/Livros/Teste.bookmaker"),
    ).resolves.toMatchObject({ title: "Teste", documents: [{ documentId }] });
    expect(invoke).toHaveBeenCalledWith("load_writing_project", {
      input: { projectPath: "C:/Livros/Teste.bookmaker" },
    });
  });

  it("loads content only after validating its schema", async () => {
    invoke.mockResolvedValue({
      documentId,
      schemaVersion: 1,
      content: {
        type: "doc",
        content: [
          { type: "paragraph", content: [{ type: "text", text: "Olá" }] },
        ],
      },
      updatedAt: "2026-10-06T12:00:00.000Z",
    });

    await expect(
      loadDocumentContent("C:/Livros/Teste.bookmaker", documentId),
    ).resolves.toMatchObject({ documentId, content: { type: "doc" } });
  });

  it("rejects malformed native content", async () => {
    invoke.mockResolvedValue({
      documentId,
      schemaVersion: 1,
      content: { type: "unknown" },
      updatedAt: "2026-10-06T12:00:00.000Z",
    });

    await expect(
      loadDocumentContent("C:/Livros/Teste.bookmaker", documentId),
    ).rejects.toBeInstanceOf(WritingProjectProtocolError);
  });

  it("validates and saves editor content with the current schema version", async () => {
    invoke.mockResolvedValue({
      documentId,
      updatedAt: "2026-10-06T12:01:00.000Z",
    });
    const content = {
      type: "doc",
      content: [
        { type: "paragraph", content: [{ type: "text", text: "Café" }] },
      ],
    };

    await expect(
      saveDocumentContent("C:/Livros/Teste.bookmaker", documentId, content),
    ).resolves.toEqual({
      documentId,
      updatedAt: "2026-10-06T12:01:00.000Z",
    });
    expect(invoke).toHaveBeenCalledWith("save_document_content", {
      input: {
        projectPath: "C:/Livros/Teste.bookmaker",
        documentId,
        schemaVersion: 1,
        content,
      },
    });
  });
});
