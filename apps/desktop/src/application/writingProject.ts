import { invoke } from "@tauri-apps/api/core";
import { z } from "zod";

import type { DocumentId, ProjectId } from "@bookmaker/domain";
import {
  EDITOR_SCHEMA_VERSION,
  parseEditorContent,
  type EditorContent,
} from "@bookmaker/editor-core";

const documentIdSchema = z
  .string()
  .min(1)
  .transform((value) => value as DocumentId);

const projectIdSchema = z
  .string()
  .min(1)
  .transform((value) => value as ProjectId);

const writingDocumentSchema = z
  .object({
    documentId: documentIdSchema,
    parentId: documentIdSchema.nullable(),
    title: z.string(),
    position: z.number().int().nonnegative(),
  })
  .strict();

const writingProjectSchema = z
  .object({
    projectId: projectIdSchema,
    path: z.string().min(1),
    title: z.string(),
    language: z.string().min(1),
    documents: z.array(writingDocumentSchema),
  })
  .strict();

const loadedDocumentContentSchema = z
  .object({
    documentId: documentIdSchema,
    schemaVersion: z.number().int().positive(),
    content: z.unknown(),
    updatedAt: z.string().min(1),
  })
  .strict();

const savedDocumentContentSchema = z
  .object({
    documentId: documentIdSchema,
    updatedAt: z.string().min(1),
  })
  .strict();

export type WritingDocument = z.infer<typeof writingDocumentSchema>;
export type WritingProject = z.infer<typeof writingProjectSchema>;

export interface LoadedDocumentContent {
  documentId: DocumentId;
  schemaVersion: number;
  content: EditorContent;
  updatedAt: string;
}

export interface SavedDocumentContent {
  documentId: DocumentId;
  updatedAt: string;
}

export interface WritingProjectGateway {
  loadProject(projectPath: string): Promise<WritingProject>;
  loadDocument(
    projectPath: string,
    documentId: DocumentId,
  ): Promise<LoadedDocumentContent>;
  saveDocument(
    projectPath: string,
    documentId: DocumentId,
    content: EditorContent,
  ): Promise<SavedDocumentContent>;
}

export class WritingProjectProtocolError extends Error {
  public readonly cause: unknown;

  public constructor(message: string, cause?: unknown) {
    super(message);
    this.name = "WritingProjectProtocolError";
    this.cause = cause;
  }
}

function parseResponse<Output>(
  schema: z.ZodType<Output>,
  response: unknown,
  operation: string,
): Output {
  const result = schema.safeParse(response);
  if (!result.success) {
    throw new WritingProjectProtocolError(
      `Native ${operation} returned an invalid response`,
      result.error,
    );
  }
  return result.data;
}

export async function loadWritingProject(
  projectPath: string,
): Promise<WritingProject> {
  const response: unknown = await invoke("load_writing_project", {
    input: { projectPath },
  });
  return parseResponse(writingProjectSchema, response, "project loading");
}

export async function loadDocumentContent(
  projectPath: string,
  documentId: DocumentId,
): Promise<LoadedDocumentContent> {
  const response: unknown = await invoke("load_document_content", {
    input: { projectPath, documentId },
  });
  const parsed = parseResponse(
    loadedDocumentContentSchema,
    response,
    "document loading",
  );
  if (parsed.schemaVersion !== EDITOR_SCHEMA_VERSION) {
    throw new WritingProjectProtocolError(
      `Unsupported editor schema version: ${parsed.schemaVersion}`,
    );
  }

  try {
    return {
      ...parsed,
      content: parseEditorContent(parsed.content),
    };
  } catch (error: unknown) {
    throw new WritingProjectProtocolError(
      "Native document loading returned invalid editor content",
      error,
    );
  }
}

export async function saveDocumentContent(
  projectPath: string,
  documentId: DocumentId,
  content: EditorContent,
): Promise<SavedDocumentContent> {
  const validatedContent = parseEditorContent(content);
  const response: unknown = await invoke("save_document_content", {
    input: {
      projectPath,
      documentId,
      schemaVersion: EDITOR_SCHEMA_VERSION,
      content: validatedContent,
    },
  });
  return parseResponse(savedDocumentContentSchema, response, "document saving");
}

export const nativeWritingProjectGateway: WritingProjectGateway = {
  loadProject: loadWritingProject,
  loadDocument: loadDocumentContent,
  saveDocument: saveDocumentContent,
};
