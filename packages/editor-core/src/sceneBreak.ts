import { Node } from "@tiptap/core";

import { insertBlockNodeWithTrailingParagraph } from "./insertBlockNode";

declare module "@tiptap/core" {
  interface Commands<ReturnType> {
    sceneBreak: {
      insertSceneBreak: () => ReturnType;
    };
  }
}

export const SCENE_BREAK_NODE_NAME = "sceneBreak";

export const SceneBreak = Node.create({
  name: SCENE_BREAK_NODE_NAME,
  group: "block",
  atom: true,

  parseHTML() {
    return [{ tag: "hr[data-scene-break]" }];
  },

  renderHTML() {
    return ["hr", { "data-scene-break": "" }];
  },

  addCommands() {
    return {
      insertSceneBreak: () => (commandProps) =>
        insertBlockNodeWithTrailingParagraph(commandProps, {
          nodeName: this.name,
        }),
    };
  },
});
