(function () {
  const ct = (document.contentType || "").toLowerCase();
  if (!ct.startsWith("text/markdown") && !ct.startsWith("text/x-markdown")) return;
  if (document.documentElement.dataset.mdRendered === "1") return;

  const initialSource = extractSource();
  if (initialSource == null) return;

  const isFile = isMarkdownFilePath(location.pathname);
  const state = { source: initialSource, mode: "view", editorReady: false };

  buildShell();
  if (isFile) buildToolbar();
  renderView();
  loadSidebar();
  document.documentElement.dataset.mdRendered = "1";

  window.addEventListener("message", (e) => {
    const d = e.data;
    if (!d || d.source !== "mdx-editor") return;
    if (d.type === "ready") {
      state.editorReady = true;
    } else if (d.type === "change") {
      state.source = d.value;
    } else if (d.type === "error") {
      setStatus("Editor error: " + d.message);
    }
  });

  function buildShell() {
    document.body.classList.add("md-rendered");
    document.body.innerHTML =
      '<main class="md-container">' +
        '<article class="markdown-body"></article>' +
      "</main>" +
      (isFile ? '<div class="md-toolbar" role="toolbar"></div>' : "");
  }

  function buildToolbar() {
    const tb = document.querySelector(".md-toolbar");
    tb.innerHTML =
      '<button id="md-toggle" type="button">Edit</button>' +
      '<button id="md-save" type="button" hidden>Save</button>' +
      '<span id="md-status" aria-live="polite"></span>';
    document.getElementById("md-toggle").addEventListener("click", toggleMode);
    document.getElementById("md-save").addEventListener("click", save);
    window.addEventListener("beforeunload", (e) => {
      if (state.mode === "edit" && pendingDirty()) {
        e.preventDefault();
        e.returnValue = "";
      }
    });
  }

  async function toggleMode() {
    if (state.mode === "view") await enterEdit();
    else enterView();
  }

  async function enterEdit() {
    state.mode = "edit";
    document.getElementById("md-toggle").textContent = "View";
    document.getElementById("md-save").hidden = false;
    setStatus("");

    const article = document.querySelector(".markdown-body");
    article.innerHTML = '<div id="mdx-editor-mount"></div>';

    try {
      await ensureBundleLoaded();
      window.postMessage(
        { target: "mdx-editor", type: "init", initialContent: state.source },
        "*",
      );
    } catch (e) {
      console.warn("[md-renderer] bundle failed to load, using textarea:", e);
      fallbackTextarea(article);
    }
  }

  function enterView() {
    if (state.editorReady) {
      window.postMessage({ target: "mdx-editor", type: "unmount" }, "*");
    } else {
      const ta = document.querySelector(".md-editor-fallback");
      if (ta) state.source = ta.value;
    }
    state.mode = "view";
    document.getElementById("md-toggle").textContent = "Edit";
    document.getElementById("md-save").hidden = true;
    renderView();
  }

  function renderView() {
    const article = document.querySelector(".markdown-body");
    try {
      article.innerHTML = marked.parse(state.source, { gfm: true, breaks: false });
    } catch (e) {
      article.textContent = state.source;
    }
    setTitleFromHeading();
  }

  async function save() {
    const btn = document.getElementById("md-save");
    btn.disabled = true;
    setStatus("Saving…");
    try {
      const res = await fetch(location.href, {
        method: "PUT",
        headers: { "Content-Type": "text/markdown; charset=utf-8" },
        body: state.source,
      });
      if (!res.ok) {
        setStatus("Save failed: HTTP " + res.status);
        return;
      }
      setStatus("Saved");
      enterView();
    } catch (e) {
      setStatus("Save failed: " + (e && e.message || e));
    } finally {
      btn.disabled = false;
    }
  }

  function ensureBundleLoaded() {
    if (window.__mdxBundleLoading) return window.__mdxBundleLoading;
    window.__mdxBundleLoading = new Promise((resolve, reject) => {
      const cssUrl = chrome.runtime.getURL("editor-bundle.css");
      const jsUrl = chrome.runtime.getURL("editor-bundle.js");

      const link = document.createElement("link");
      link.rel = "stylesheet";
      link.href = cssUrl;
      document.head.appendChild(link);

      const script = document.createElement("script");
      script.src = jsUrl;
      script.onload = () => resolve();
      script.onerror = () => reject(new Error("script load failed"));
      document.head.appendChild(script);
    });
    return window.__mdxBundleLoading;
  }

  function fallbackTextarea(article) {
    article.innerHTML = "";
    const ta = document.createElement("textarea");
    ta.className = "md-editor md-editor-fallback";
    ta.spellcheck = false;
    ta.value = state.source;
    ta.addEventListener("input", () => { state.source = ta.value; });
    article.appendChild(ta);
    ta.focus();
  }

  function pendingDirty() { return state.source !== initialSource; }

  function setStatus(s) {
    const el = document.getElementById("md-status");
    if (el) el.textContent = s;
  }

  function setTitleFromHeading() {
    const h = document.querySelector(".markdown-body h1, .markdown-body h2");
    if (h && h.textContent) document.title = h.textContent.trim();
  }

  function extractSource() {
    const pre = document.body && document.body.querySelector("pre");
    if (pre && document.body.children.length === 1) return pre.textContent;
    if (document.body) {
      const text = document.body.innerText || document.body.textContent || "";
      if (text.length > 0) return text;
    }
    return null;
  }

  function isMarkdownFilePath(pathname) {
    return /\.md$/i.test(pathname.replace(/\/+$/, ""));
  }

  async function loadSidebar() {
    let tree;
    try {
      const res = await fetch("/introspect", { credentials: "same-origin" });
      if (!res.ok) return;
      tree = await res.json();
    } catch (e) {
      console.warn("[md-renderer] /introspect failed:", e);
      return;
    }
    renderSidebar(tree);
  }

  function renderSidebar(tree) {
    const aside = document.createElement("aside");
    aside.className = "md-sidebar";

    const heading = document.createElement("div");
    heading.className = "md-sidebar-heading";
    heading.textContent = "Files";
    aside.appendChild(heading);

    const nav = document.createElement("nav");
    const ul = document.createElement("ul");
    ul.className = "md-tree";
    ul.appendChild(buildNode(tree));
    nav.appendChild(ul);
    aside.appendChild(nav);

    aside.addEventListener("click", (e) => {
      const chev = e.target.closest(".md-chevron");
      if (!chev) return;
      e.preventDefault();
      const dir = chev.closest(".md-dir");
      if (dir) dir.classList.toggle("is-open");
    });

    document.body.insertBefore(aside, document.body.firstChild);
  }

  function buildNode(node) {
    return node.type === "directory" ? buildDir(node) : buildFile(node);
  }

  function buildDir(dir) {
    const li = document.createElement("li");
    li.className = "md-dir";
    if (isAncestorOrSelf(dir.path)) li.classList.add("is-open");

    const row = document.createElement("div");
    row.className = "md-dir-row";

    const chev = document.createElement("span");
    chev.className = "md-chevron";
    row.appendChild(chev);

    const link = document.createElement("a");
    link.className = "md-link";
    link.href = encodePath(dir.path);
    link.textContent = dir.path === "/" ? "/" : dir.name + "/";
    if (currentPath() === dir.path) link.classList.add("selected");
    row.appendChild(link);

    li.appendChild(row);

    if (dir.children && dir.children.length) {
      const ul = document.createElement("ul");
      ul.className = "md-children";
      for (const child of dir.children) ul.appendChild(buildNode(child));
      li.appendChild(ul);
    }
    return li;
  }

  function buildFile(file) {
    const li = document.createElement("li");
    li.className = "md-file";
    const a = document.createElement("a");
    a.className = "md-link";
    a.href = encodePath(file.path);
    a.textContent = file.name;
    if (currentPath() === file.path) a.classList.add("selected");
    li.appendChild(a);
    return li;
  }

  function currentPath() {
    let p = location.pathname.replace(/\/+$/, "");
    try { p = decodeURIComponent(p); } catch (_) {}
    return p === "" ? "/" : p;
  }

  function encodePath(path) {
    return path.split("/").map(encodeURIComponent).join("/");
  }

  function isAncestorOrSelf(dirPath) {
    const cur = currentPath();
    if (dirPath === "/") return true;
    if (cur === dirPath) return true;
    return cur.startsWith(dirPath + "/");
  }
})();
