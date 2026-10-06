export {
  EDITOR_SCHEMA_VERSION,
  InvalidEditorContentError,
  createEditorExtensions,
  createEmptyEditorContent,
  editorSchema,
  isEditorContent,
  parseEditorContent,
} from "./editorSchema";

export type { JSONContent as EditorContent } from "@tiptap/core";
