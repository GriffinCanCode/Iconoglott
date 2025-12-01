const KEYWORDS = /* @__PURE__ */ new Set(["canvas", "group", "stack", "row"]);
const SHAPES = /* @__PURE__ */ new Set(["rect", "circle", "ellipse", "line", "path", "polygon", "text", "image"]);
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
  "translate",
  "rotate",
  "scale",
  "origin",
  "width",
  "gap",
  "vertical",
  "horizontal",
  "linear",
  "radial",
  "d",
  "points",
  "href"
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
const DEFAULT_CODE = `// Iconoglott Visual DSL
// Try the shapes and styles below!

canvas 800x600 fill #0d1117

// Variables for reusable colors
$accent = #f97316
$cyan = #22d3ee
$purple = #a855f7

// Rectangle with gradient and shadow
rect at 80,80 size 200x140
  gradient linear $accent $purple 135
  corner 16
  shadow 0,12 20 #0004

// Circle with radial gradient
circle at 420,160 radius 70
  gradient radial $cyan #0d1117

// Ellipse shape
ellipse at 600,300 radius 60,35
  fill $purple
  opacity 0.8

// Polygon (triangle)
polygon points [200,400 280,280 360,400]
  fill $cyan
  stroke #fff 2

// Line
line from 450,400 to 650,280
  stroke $accent 3

// Text with styling
text "Iconoglott" at 80,520
  font "Space Grotesk" 32
  fill #fff
  bold

// Grouped elements
group "button"
  rect at 500,450 size 140x48
    fill $accent
    corner 24
  text "Explore \u2192" at 570,480
    font "Space Grotesk" 15
    fill #fff
    center

// Layout example
stack gap 12 at 80,300
  rect size 160x30
    fill #21262d
    corner 6
  rect size 140x30
    fill #21262d
    corner 6
  rect size 120x30
    fill #21262d
    corner 6`;
new IconoglottClient();
