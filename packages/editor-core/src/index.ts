export {
  EDITOR_SCHEMA_VERSION,
  InvalidEditorContentError,
  createEditorExtensions,
  createEmptyEditorContent,
  editorSchema,
  isEditorContent,
  parseEditorContent,
} from "./editorSchema";
export { findDocumentMatches } from "./documentSearch";
export type { DocumentTextMatch } from "./documentSearch";

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
export { isSafeEditorLink } from "./link";
export {
  PasteSanitizer,
  sanitizePastedHtml,
  sanitizePastedText,
} from "./pasteSanitizer";
export { SCENE_BREAK_NODE_NAME, SceneBreak } from "./sceneBreak";
export { countDocumentWords, countWords } from "./wordCount";
