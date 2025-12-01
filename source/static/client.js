/**
 * Iconoglott WebSocket Client
 * Real-time DSL rendering
 */

class IconoglottClient {
  constructor() {
    this.ws = null;
    this.editor = document.getElementById('editor');
    this.canvas = document.getElementById('canvas');
    this.status = document.getElementById('status');
    this.debounceTimer = null;
    this.init();
  }

  init() {
    this.connect();
    this.editor.addEventListener('input', () => this.onInput());
    
    // Initial render with placeholder
    setTimeout(() => {
      if (this.editor.value) this.send(this.editor.value);
    }, 500);
  }

  connect() {
    const proto = location.protocol === 'https:' ? 'wss:' : 'ws:';
    this.ws = new WebSocket(`${proto}//${location.host}/ws`);

    this.ws.onopen = () => {
      this.status.classList.add('connected');
      if (this.editor.value) this.send(this.editor.value);
    };

    this.ws.onclose = () => {
      this.status.classList.remove('connected');
      setTimeout(() => this.connect(), 2000);
    };

    this.ws.onmessage = (e) => this.onMessage(e);
  }

  onInput() {
    clearTimeout(this.debounceTimer);
    this.debounceTimer = setTimeout(() => {
      this.send(this.editor.value);
    }, 100);
  }

  send(source) {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify({ type: 'source', payload: source }));
    }
  }

  onMessage(e) {
    try {
      const msg = JSON.parse(e.data);
      if (msg.type === 'render' && msg.svg) {
        this.canvas.innerHTML = msg.svg;
      } else if (msg.type === 'error') {
        console.error('Render error:', msg.message);
      }
    } catch (err) {
      console.error('Parse error:', err);
    }
  }
}

new IconoglottClient();

