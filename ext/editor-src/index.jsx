import { createRoot } from "react-dom/client";
import {
  MDXEditor,
  headingsPlugin,
  listsPlugin,
  quotePlugin,
  thematicBreakPlugin,
  linkPlugin,
  linkDialogPlugin,
  imagePlugin,
  tablePlugin,
  markdownShortcutPlugin,
} from "@mdxeditor/editor";
import "@mdxeditor/editor/style.css";

const TARGET = "mdx-editor";
const MOUNT_ID = "mdx-editor-mount";

function post(type, data) {
  window.postMessage({ source: TARGET, type, ...data }, "*");
}

let currentRoot = null;

function mount(initialContent) {
  unmount();
  const container = document.getElementById(MOUNT_ID);
  if (!container) {
    post("error", { message: "no mount container" });
    return;
  }
  const root = createRoot(container);
  currentRoot = root;
  root.render(
    <div className="dark-theme dark-editor">
      <MDXEditor
        markdown={initialContent ?? ""}
        onChange={(v) => post("change", { value: v })}
        contentEditableClassName="mdx-editor-content"
        plugins={[
          headingsPlugin(),
          listsPlugin(),
          quotePlugin(),
          thematicBreakPlugin(),
          linkPlugin(),
          linkDialogPlugin(),
          imagePlugin(),
          tablePlugin(),
          markdownShortcutPlugin(),
        ]}
      />
    </div>,
  );
}

function unmount() {
  if (currentRoot) {
    currentRoot.unmount();
    currentRoot = null;
  }
}

window.addEventListener("message", (e) => {
  const d = e.data;
  if (!d || d.target !== TARGET) return;
  if (d.type === "init") mount(d.initialContent);
  else if (d.type === "unmount") unmount();
});

post("ready");
