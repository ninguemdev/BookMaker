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

export {
  BookImage,
  IMAGE_ALIGNMENTS,
  IMAGE_NODE_NAME,
  IMAGE_WIDTH_PRESETS,
  createImageNode,
} from "./image";
export type {
  BookImageAttributes,
  ImageAlignment,
  ImageWidthPreset,
  InsertImageOptions,
} from "./image";
export { SCENE_BREAK_NODE_NAME, SceneBreak } from "./sceneBreak";
