export {
  EDITOR_SCHEMA_VERSION,
  InvalidEditorContentError,
  createEditorExtensions,
  createEmptyEditorContent,
  editorSchema,
  isEditorContent,
  isSafeEditorLink,
  parseEditorContent,
} from "./editorSchema";

export type { JSONContent as EditorContent } from "@tiptap/core";

export { SCENE_BREAK_NODE_NAME, SceneBreak } from "./sceneBreak";
