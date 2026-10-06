import {
  Node,
  getSchema,
  type Extensions,
  type JSONContent,
} from "@tiptap/core";
import { isAllowedUri } from "@tiptap/extension-link";
import StarterKit from "@tiptap/starter-kit";

import {
  BookImage,
  IMAGE_NODE_NAME,
  assertValidImageAttributes,
} from "./image";
import { SCENE_BREAK_NODE_NAME, SceneBreak } from "./sceneBreak";

export const EDITOR_SCHEMA_VERSION = 1;

const BookDocument = Node.create({
  name: "doc",
  topNode: true,
  content: "block*",
});

export function createEditorExtensions(): Extensions {
  return [
    BookDocument,
    StarterKit.configure({
      code: false,
      codeBlock: false,
      document: false,
      heading: {
        levels: [1, 2, 3],
      },
      horizontalRule: false,
      strike: false,
    }),
    BookImage,
    SceneBreak,
  ];
}

export const editorSchema = getSchema(createEditorExtensions());

export function createEmptyEditorContent(): JSONContent {
  return {
    type: "doc",
    content: [],
  };
}

export class InvalidEditorContentError extends Error {
  public readonly cause: unknown;

  public constructor(cause: unknown) {
    super(
      `Editor content does not conform to schema version ${EDITOR_SCHEMA_VERSION}`,
    );
    this.name = "InvalidEditorContentError";
    this.cause = cause;
  }
}

const nodeJsonKeys = new Set(["type", "attrs", "content", "marks", "text"]);
const markJsonKeys = new Set(["type", "attrs"]);

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function assertKnownKeys(
  value: Record<string, unknown>,
  allowedKeys: ReadonlySet<string>,
  valueName: string,
): void {
  for (const key of Object.keys(value)) {
    if (!allowedKeys.has(key)) {
      throw new RangeError(`${valueName} contains unsupported field: ${key}`);
    }
  }
}

function assertKnownAttributes(
  value: unknown,
  allowedAttributes: ReadonlySet<string>,
  valueName: string,
): void {
  if (value === undefined) {
    return;
  }

  if (!isRecord(value)) {
    throw new TypeError(`${valueName} attributes must be an object`);
  }

  assertKnownKeys(value, allowedAttributes, `${valueName} attributes`);
}

function assertKnownMarkShape(value: unknown): void {
  if (!isRecord(value)) {
    throw new TypeError("Editor mark must be an object");
  }

  assertKnownKeys(value, markJsonKeys, "Editor mark");

  if (typeof value.type !== "string") {
    throw new TypeError("Editor mark type must be a string");
  }

  const markType = editorSchema.marks[value.type];
  if (markType === undefined) {
    throw new RangeError(`Unsupported editor mark: ${value.type}`);
  }

  assertKnownAttributes(
    value.attrs,
    new Set(Object.keys(markType.spec.attrs ?? {})),
    `Editor mark ${value.type}`,
  );
}

function assertKnownNodeShape(value: unknown): void {
  if (!isRecord(value)) {
    throw new TypeError("Editor node must be an object");
  }

  assertKnownKeys(value, nodeJsonKeys, "Editor node");

  if (typeof value.type !== "string") {
    throw new TypeError("Editor node type must be a string");
  }

  const nodeType = editorSchema.nodes[value.type];
  if (nodeType === undefined) {
    throw new RangeError(`Unsupported editor node: ${value.type}`);
  }

  assertKnownAttributes(
    value.attrs,
    new Set(Object.keys(nodeType.spec.attrs ?? {})),
    `Editor node ${value.type}`,
  );

  if (value.type === "heading" && isRecord(value.attrs)) {
    const level = value.attrs.level;
    if (level !== undefined && level !== 1 && level !== 2 && level !== 3) {
      throw new RangeError("Editor heading level must be 1, 2, or 3");
    }
  }

  if (
    (value.type === IMAGE_NODE_NAME || value.type === SCENE_BREAK_NODE_NAME) &&
    (value.content !== undefined ||
      value.marks !== undefined ||
      value.text !== undefined)
  ) {
    throw new RangeError(
      "Atomic editor nodes cannot contain text, marks, or children",
    );
  }

  if (value.type === IMAGE_NODE_NAME) {
    assertValidImageAttributes(value.attrs);
  }

  if (value.marks !== undefined) {
    if (!Array.isArray(value.marks)) {
      throw new TypeError("Editor node marks must be an array");
    }
    value.marks.forEach(assertKnownMarkShape);
  }

  if (value.content !== undefined) {
    if (!Array.isArray(value.content)) {
      throw new TypeError("Editor node content must be an array");
    }
    value.content.forEach(assertKnownNodeShape);
  }
}

function assertSafeLinks(
  document: ReturnType<typeof editorSchema.nodeFromJSON>,
): void {
  document.descendants((node) => {
    for (const mark of node.marks) {
      if (mark.type.name !== "link") {
        continue;
      }

      const href: unknown = mark.attrs.href;
      if (!isSafeEditorLink(href)) {
        throw new RangeError("Editor link contains an unsafe URL");
      }
    }
  });
}

export function isSafeEditorLink(href: unknown): href is string {
  return (
    typeof href === "string" &&
    href.trim().length > 0 &&
    Boolean(isAllowedUri(href))
  );
}

export function parseEditorContent(value: unknown): JSONContent {
  try {
    assertKnownNodeShape(value);
    const document = editorSchema.nodeFromJSON(value);

    if (document.type !== editorSchema.topNodeType) {
      throw new RangeError("Editor content root must be a doc node");
    }

    document.check();
    assertSafeLinks(document);
    return document.toJSON();
  } catch (error: unknown) {
    if (error instanceof InvalidEditorContentError) {
      throw error;
    }

    throw new InvalidEditorContentError(error);
  }
}

export function isEditorContent(value: unknown): value is JSONContent {
  try {
    parseEditorContent(value);
    return true;
  } catch {
    return false;
  }
}
