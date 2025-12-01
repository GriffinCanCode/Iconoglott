const KEYWORDS = /* @__PURE__ */ new Set(["canvas", "group", "stack", "row", "graph", "node", "edge"]);
const SHAPES = /* @__PURE__ */ new Set(["rect", "circle", "ellipse", "line", "path", "polygon", "text", "image", "arc", "curve", "diamond"]);
const PROPERTIES = /* @__PURE__ */ new Set([
  "at",
  "size",
  "radius",
  "from",
  "to",
  "fill",
  "stroke",
  "opacity",
  "corner",
  "shadow",
  "gradient",
  "blur",
  "font",
  "bold",
  "italic",
  "center",
  "end",
  "middle",
  "translate",
  "rotate",
  "scale",
  "origin",
  "width",
  "height",
  "gap",
  "vertical",
  "horizontal",
  "linear",
  "radial",
  "d",
  "points",
  "href",
  "label",
  "shape",
  "spacing",
  "curved",
  "straight",
  "orthogonal",
  "hierarchical",
  "force",
  "grid",
  "tree",
  "manual",
  "justify",
  "align",
  "wrap",
  "start",
  "smooth",
  "sharp",
  "closed",
  "padding",
  "anchor",
  "auto"
]);
class IconoglottClient {
  constructor() {
    this.ws = null;
    this.editor = document.getElementById("editor");
    this.highlight = document.getElementById("highlight");
    this.canvas = document.getElementById("canvas");
    this.status = document.getElementById("status");
    this.lineNumbers = document.getElementById("lineNumbers");
    this.errorPanel = document.getElementById("errorPanel");
    this.errorCount = document.getElementById("errorCount");
    this.debounceTimer = null;
    this.errors = [];
    this.init();
  }
  init() {
    this.connect();
    this.editor.addEventListener("input", () => this.onInput());
    this.editor.addEventListener("scroll", () => this.syncScroll());
    this.editor.addEventListener("keydown", (e) => this.handleKey(e));
    this.editor.value = DEFAULT_CODE;
    this.updateHighlight();
    this.updateLineNumbers();
    setTimeout(() => {
      if (this.editor.value) this.send(this.editor.value);
    }, 300);
  }
  connect() {
    const proto = location.protocol === "https:" ? "wss:" : "ws:";
    this.ws = new WebSocket(`${proto}//${location.host}/ws`);
    this.ws.onopen = () => {
      this.status.classList.add("connected");
      if (this.editor.value) this.send(this.editor.value);
    };
    this.ws.onclose = () => {
      this.status.classList.remove("connected");
      setTimeout(() => this.connect(), 2e3);
    };
    this.ws.onmessage = (e) => this.onMessage(e);
  }
  onInput() {
    this.updateHighlight();
    this.updateLineNumbers();
    clearTimeout(this.debounceTimer);
    this.debounceTimer = setTimeout(() => {
      this.send(this.editor.value);
    }, 80);
  }
  handleKey(e) {
    if (e.key === "Tab") {
      e.preventDefault();
      const start = this.editor.selectionStart;
      const end = this.editor.selectionEnd;
      this.editor.value = this.editor.value.substring(0, start) + "  " + this.editor.value.substring(end);
      this.editor.selectionStart = this.editor.selectionEnd = start + 2;
      this.onInput();
    }
  }
  syncScroll() {
    this.highlight.scrollTop = this.editor.scrollTop;
    this.highlight.scrollLeft = this.editor.scrollLeft;
    this.lineNumbers.scrollTop = this.editor.scrollTop;
  }
  updateLineNumbers() {
    const lines = this.editor.value.split("\n").length;
    const errorLines = new Set(this.errors.map((e) => e.line));
    let html = "";
    for (let i = 1; i <= lines; i++) {
      const hasError = errorLines.has(i - 1);
      html += `<div class="line${hasError ? " error" : ""}">${i}</div>`;
    }
    this.lineNumbers.innerHTML = html;
  }
  updateHighlight() {
    const code = this.editor.value;
    this.highlight.innerHTML = this.highlightCode(code);
  }
  highlightCode(code) {
    const lines = code.split("\n");
    return lines.map((line) => this.highlightLine(line)).join("\n");
  }
  highlightLine(line) {
    const commentIdx = line.indexOf("//");
    if (commentIdx !== -1) {
      const before = line.substring(0, commentIdx);
      const comment = line.substring(commentIdx);
      return this.highlightTokens(before) + `<span class="comment">${this.escapeHtml(comment)}</span>`;
    }
    return this.highlightTokens(line);
  }
  highlightTokens(line) {
    const tokens = [];
    let remaining = line;
    let pos = 0;
    while (remaining.length > 0) {
      const wsMatch = remaining.match(/^(\s+)/);
      if (wsMatch) {
        tokens.push({ type: "plain", value: wsMatch[1] });
        remaining = remaining.slice(wsMatch[1].length);
        continue;
      }
      const varMatch = remaining.match(/^(\$[a-zA-Z_][a-zA-Z0-9_]*)/);
      if (varMatch) {
        tokens.push({ type: "variable", value: varMatch[1] });
        remaining = remaining.slice(varMatch[1].length);
        continue;
      }
      const colorMatch = remaining.match(/^(#[0-9a-fA-F]{3,8})\b/);
      if (colorMatch) {
        tokens.push({ type: "color", value: colorMatch[1] });
        remaining = remaining.slice(colorMatch[1].length);
        continue;
      }
      const strMatch = remaining.match(/^("[^"]*"|'[^']*')/);
      if (strMatch) {
        tokens.push({ type: "string", value: strMatch[1] });
        remaining = remaining.slice(strMatch[1].length);
        continue;
      }
      const pairMatch = remaining.match(/^(-?\d+\.?\d*[,x]-?\d+\.?\d*)/);
      if (pairMatch) {
        tokens.push({ type: "number", value: pairMatch[1] });
        remaining = remaining.slice(pairMatch[1].length);
        continue;
      }
      const numMatch = remaining.match(/^(-?\d+\.?\d*)/);
      if (numMatch) {
        tokens.push({ type: "number", value: numMatch[1] });
        remaining = remaining.slice(numMatch[1].length);
        continue;
      }
      const identMatch = remaining.match(/^([a-zA-Z_][a-zA-Z0-9_-]*)/);
      if (identMatch) {
        const word = identMatch[1];
        let type = "plain";
        if (KEYWORDS.has(word)) type = "keyword";
        else if (SHAPES.has(word)) type = "shape";
        else if (PROPERTIES.has(word)) type = "property";
        tokens.push({ type, value: word });
        remaining = remaining.slice(word.length);
        continue;
      }
      const opMatch = remaining.match(/^([=:\->\[\]])/);
      if (opMatch) {
        tokens.push({ type: "operator", value: opMatch[1] });
        remaining = remaining.slice(opMatch[1].length);
        continue;
      }
      tokens.push({ type: "plain", value: remaining[0] });
      remaining = remaining.slice(1);
    }
    return tokens.map((t) => `<span class="${t.type}">${this.escapeHtml(t.value)}</span>`).join("");
  }
  escapeHtml(str) {
    return str.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
  }
  send(source) {
    var _a;
    if (((_a = this.ws) == null ? void 0 : _a.readyState) === WebSocket.OPEN) {
      this.ws.send(JSON.stringify({ type: "source", payload: source }));
    }
  }
  onMessage(e) {
    try {
      const msg = JSON.parse(e.data);
      if (msg.type === "render" && msg.svg) {
        this.canvas.innerHTML = msg.svg;
      }
      this.errors = msg.errors || [];
      this.updateErrors();
      this.updateLineNumbers();
    } catch (err) {
      console.error("Parse error:", err);
    }
  }
  updateErrors() {
    const count = this.errors.length;
    this.errorCount.textContent = `${count} error${count !== 1 ? "s" : ""}`;
    this.errorCount.classList.toggle("visible", count > 0);
    if (count > 0) {
      this.errorPanel.innerHTML = this.errors.map(
        (e) => `<div class="error-item" data-line="${e.line}">
          <span class="line-ref">L${e.line + 1}:</span>${this.escapeHtml(e.message)}
        </div>`
      ).join("");
      this.errorPanel.classList.add("visible");
      this.errorPanel.querySelectorAll(".error-item").forEach((el) => {
        el.addEventListener("click", () => {
          const line = parseInt(el.dataset.line);
          this.goToLine(line);
        });
      });
    } else {
      this.errorPanel.classList.remove("visible");
      this.errorPanel.innerHTML = "";
    }
  }
  goToLine(lineNum) {
    const lines = this.editor.value.split("\n");
    let pos = 0;
    for (let i = 0; i < lineNum && i < lines.length; i++) {
      pos += lines[i].length + 1;
    }
    this.editor.focus();
    this.editor.selectionStart = this.editor.selectionEnd = pos;
    const lineHeight = parseFloat(getComputedStyle(this.editor).lineHeight);
    this.editor.scrollTop = lineNum * lineHeight - this.editor.clientHeight / 2;
  }
}
const DEFAULT_CODE = `// Iconoglott — Visual DSL for Vector Graphics
// This graph shows how the rendering pipeline works

canvas giant fill #0a0f1a

// Color palette
$bg = #0a0f1a
$panel = #151d2e
$accent = #10b981
$blue = #3b82f6
$purple = #8b5cf6
$rose = #f43f5e
$amber = #f59e0b
$cyan = #06b6d4
$text = #e2e8f0

// ═══════════════════════════════════════════════
// ICONOGLOTT ARCHITECTURE GRAPH
// ═══════════════════════════════════════════════

graph hierarchical vertical spacing 65
  // Input layer
  node "DSL Source" at 256,35 size 100x36
    shape rect
    fill $blue
    label "DSL Source"
  
  // Processing layer
  node "Lexer" at 150,120 size 85x36
    shape rect
    fill $purple
    label "Lexer"
  
  node "Parser" at 256,120 size 85x36
    shape rect
    fill $purple
    label "Parser"
  
  node "Symbols" at 362,120 size 85x36
    shape rect
    fill $purple
    label "Symbols"
  
  // AST layer
  node "AST" at 256,205 size 90x36
    shape diamond
    fill $accent
    label "AST"
  
  // Render layer
  node "Layout" at 150,290 size 85x36
    shape rect
    fill $amber
    label "Layout"
  
  node "Render" at 256,290 size 85x36
    shape rect
    fill $amber
    label "Render"
  
  node "Cache" at 362,290 size 85x36
    shape ellipse
    fill $rose
    label "Cache"
  
  // Output
  node "SVG" at 256,375 size 90x36
    shape rect
    fill $cyan
    label "SVG"
  
  // Edges - Pipeline flow
  edge "DSL Source" -> "Lexer" curved $text 1.5
  edge "DSL Source" -> "Parser" curved $text 1.5
  edge "DSL Source" -> "Symbols" curved $text 1.5
  edge "Lexer" -> "AST" curved $text 1.5
  edge "Parser" -> "AST" curved $text 1.5
  edge "Symbols" -> "AST" curved $text 1.5
  edge "AST" -> "Layout" curved $text 1.5
  edge "AST" -> "Render" curved $text 1.5
  edge "AST" -> "Cache" curved $text 1.5
  edge "Layout" -> "SVG" curved $text 1.5
  edge "Render" -> "SVG" curved $text 1.5
  edge "Cache" -> "Render" curved $rose 1

// ═══════════════════════════════════════════════
// FEATURE SHOWCASE
// ═══════════════════════════════════════════════

// Title
text "Iconoglott" at 30,460
  font "Space Grotesk" 28
  fill #fff
  bold

text "Visual DSL for Vector Graphics" at 30,488
  font "Space Grotesk" 13
  fill #64748b

// Feature cards using layout
row gap 16 at 30,510
  // Variables card
  rect size 110x55
    fill $panel
    corner 8
    stroke #1e293b 1
  
  // Layouts card  
  rect size 110x55
    fill $panel
    corner 8
    stroke #1e293b 1
  
  // Graphs card
  rect size 110x55
    fill $panel
    corner 8
    stroke #1e293b 1
  
  // Transforms card
  rect size 110x55
    fill $panel
    corner 8
    stroke #1e293b 1

// Card labels
text "$vars" at 62,533
  font "JetBrains Mono" 11
  fill $accent
  center

text "layouts" at 172,533
  font "JetBrains Mono" 11
  fill $blue
  center

text "graphs" at 282,533
  font "JetBrains Mono" 11
  fill $purple
  center

text "transforms" at 395,533
  font "JetBrains Mono" 11
  fill $amber
  center

// Card descriptions
text "Reusable colors" at 62,550
  font "Space Grotesk" 9
  fill #64748b
  center

text "Stack & row" at 172,550
  font "Space Grotesk" 9
  fill #64748b
  center

text "Nodes & edges" at 282,550
  font "Space Grotesk" 9
  fill #64748b
  center

text "Rotate & scale" at 395,550
  font "Space Grotesk" 9
  fill #64748b
  center

// Decorative elements
circle at 480,470 radius 25
  gradient radial #10b981 #0a0f1a
  opacity 0.6

circle at 495,490 radius 15
  gradient radial #3b82f6 #0a0f1a
  opacity 0.4`;
new IconoglottClient();
