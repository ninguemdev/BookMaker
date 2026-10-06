import { beforeEach, describe, expect, it, vi } from "vitest";

import { ProjectSearchProtocolError, searchProject } from "./projectSearch";

const invoke = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({ invoke }));

describe("project search application boundary", () => {
  beforeEach(() => {
    invoke.mockReset();
  });

  it("invokes the native command and validates its response", async () => {
    invoke.mockResolvedValue({
      matches: [
        {
          documentId: "document-1",
          documentTitle: "Capítulo",
          snippet: "Um termo encontrado.",
          from: 4,
          to: 9,
        },
      ],
      truncated: false,
    });

    await expect(
      searchProject({
        projectPath: "C:/Livros/Teste.bookmaker",
        query: "termo",
      }),
    ).resolves.toMatchObject({
      matches: [{ documentId: "document-1", from: 4, to: 9 }],
      truncated: false,
    });
    expect(invoke).toHaveBeenCalledWith("search_project", {
      input: {
        projectPath: "C:/Livros/Teste.bookmaker",
        query: "termo",
      },
    });
  });

  it.each([
    [null],
    [{ matches: "invalid", truncated: false }],
    [
      {
        matches: [
          {
            documentId: "document-1",
            documentTitle: "Capítulo",
            snippet: "Texto",
            from: 9,
            to: 4,
          },
        ],
        truncated: false,
      },
    ],
  ])("rejects malformed native response %#", async (response) => {
    invoke.mockResolvedValue(response);

    await expect(
      searchProject({ projectPath: "project.bookmaker", query: "texto" }),
    ).rejects.toBeInstanceOf(ProjectSearchProtocolError);
  });
});
