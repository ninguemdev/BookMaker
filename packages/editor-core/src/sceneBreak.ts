import { Node, canInsertNode, isNodeSelection } from "@tiptap/core";
import { NodeSelection, TextSelection } from "@tiptap/pm/state";

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
      insertSceneBreak:
        () =>
        ({ chain, state }) => {
          const sceneBreak = state.schema.nodes[this.name];
          if (sceneBreak === undefined || !canInsertNode(state, sceneBreak)) {
            return false;
          }

          const insertion = chain();
          const { selection } = state;

          if (isNodeSelection(selection)) {
            insertion.insertContentAt(selection.$to.pos, { type: this.name });
          } else {
            insertion.insertContent({ type: this.name });
          }

          return insertion
            .command(({ state: nextState, tr, dispatch }) => {
              if (dispatch === undefined) {
                return true;
              }

              const { $to } = tr.selection;
              const nodeAfter = $to.nodeAfter;

              if (nodeAfter?.isTextblock) {
                tr.setSelection(TextSelection.create(tr.doc, $to.pos + 1));
              } else if (nodeAfter?.isBlock) {
                tr.setSelection(NodeSelection.create(tr.doc, $to.pos));
              } else if (nodeAfter === null) {
                const paragraph = nextState.schema.nodes.paragraph?.create();
                if (paragraph !== undefined) {
                  const positionAfterSceneBreak = $to.end();
                  tr.insert(positionAfterSceneBreak, paragraph);
                  tr.setSelection(
                    TextSelection.create(tr.doc, positionAfterSceneBreak + 1),
                  );
                }
              }

              tr.scrollIntoView();
              return true;
            })
            .run();
        },
    };
  },
});
