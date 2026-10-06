import { isAllowedUri } from "@tiptap/extension-link";

export function isSafeEditorLink(href: unknown): href is string {
  return (
    typeof href === "string" &&
    href.trim().length > 0 &&
    Boolean(isAllowedUri(href))
  );
}
