import type { Canvas, Shape, Node, GradientDef, ShadowDef } from './types';
import { defaultCanvas } from './types';
import { computeId, indexScene, diff, RenderCache, type SceneIndex, type DiffOp, type DiffResult } from './diff';

interface ShapeDict {
  id?: string;
  kind: string;
  props: Record<string, unknown>;
  style: Record<string, unknown>;
  transform: Record<string, unknown>;
  children: ShapeDict[];
}

export class SceneState {
  canvas: Canvas = defaultCanvas();
  defs: string[] = [];
  shapes: ShapeDict[] = [];
  private defId = 0;
  private cache = new RenderCache(512);
  private index: SceneIndex | null = null;
  private elementId = 0;

  private nextId(): string { return `d${++this.defId}`; }
  private nextElementId(): string { return `e${++this.elementId}`; }

  /** Get indexed representation for diffing */
  getIndex(): SceneIndex {
    if (!this.index) {
      this.index = indexScene(
        { canvas: this.canvas, shapes: this.shapes, defs: this.defs },
        s => this.renderShape(s)
      );
    }
    return this.index;
  }

  /** Diff against another scene state */
  diff(other: SceneState): DiffResult {
    return diff(
      this.getIndex(),
      { canvas: this.canvas, shapes: this.shapes, defs: this.defs },
      { canvas: other.canvas, shapes: other.shapes, defs: other.defs },
      s => this.renderShape(s)
    );
  }

  /** Invalidate cached index (call after mutations) */
  invalidate(): void {
    this.index = null;
  }

  toSvg(): string {
    const rendered = this.shapes.map(s => this.renderShape(s));

    let svg = `<svg xmlns="http://www.w3.org/2000/svg" width="${this.canvas.width}" height="${this.canvas.height}">`;
    svg += `<rect width="100%" height="100%" fill="${this.canvas.fill}"/>`;

    if (this.defs.length) svg += `<defs>${this.defs.join('')}</defs>`;
    svg += rendered.join('');
    return svg + '</svg>';
  }

  private renderShape(s: ShapeDict, offset: [number, number] = [0, 0]): string {
    const { kind, props, style, transform, children } = s;
    const at = (props.at as [number, number]) ?? [0, 0];
    const [x, y] = [at[0] + offset[0], at[1] + offset[1]];
    const tf = this.buildTransform(transform, x, y);

    switch (kind) {
      case 'rect': return this.renderRect(props, style, tf, x, y);
      case 'circle': return this.renderCircle(props, style, tf, x, y);
      case 'ellipse': return this.renderEllipse(props, style, tf, x, y);
      case 'text': return this.renderText(props, style, tf, x, y);
      case 'line': return this.renderLine(props, style, tf);
      case 'path': return this.renderPath(props, style, tf);
      case 'group': return this.renderGroup(children, style, tf);
      case 'layout': return this.renderLayout(props, children, style, tf);
      default: return '';
    }
  }

  private renderRect(props: Record<string, unknown>, style: Record<string, unknown>, tf: string, x: number, y: number): string {
    const [w, h] = (props.size as [number, number]) ?? [100, 100];
    const corner = style.corner as number ?? 0;
    let attrs = `x="${x}" y="${y}" width="${w}" height="${h}"`;
    if (corner) attrs += ` rx="${corner}"`;
    return `<rect ${attrs}${this.styleAttrs(style)}${tf}/>`;
  }

  private renderCircle(props: Record<string, unknown>, style: Record<string, unknown>, tf: string, x: number, y: number): string {
    const r = (props.radius as number) ?? 50;
    return `<circle cx="${x}" cy="${y}" r="${r}"${this.styleAttrs(style)}${tf}/>`;
  }

  private renderEllipse(props: Record<string, unknown>, style: Record<string, unknown>, tf: string, x: number, y: number): string {
    const [rx, ry] = (props.size as [number, number]) ?? [50, 30];
    return `<ellipse cx="${x}" cy="${y}" rx="${rx}" ry="${ry}"${this.styleAttrs(style)}${tf}/>`;
  }

  private renderText(props: Record<string, unknown>, style: Record<string, unknown>, tf: string, x: number, y: number): string {
    const content = (props.content as string) ?? '';
    const font = (style.font as string) ?? 'system-ui';
    const size = (style.fontSize as number) ?? 16;
    const weight = (style.fontWeight as string) ?? 'normal';
    const anchor = (style.textAnchor as string) ?? 'start';
    const fill = (style.fill as string) ?? '#000';

    const attrs = `x="${x}" y="${y}" font-family="${font}" font-size="${size}" font-weight="${weight}" text-anchor="${anchor}" fill="${fill}"`;
    return `<text ${attrs}${tf}>${this.escapeHtml(content)}</text>`;
  }

  private renderLine(props: Record<string, unknown>, style: Record<string, unknown>, tf: string): string {
    const [x1, y1] = (props.from as [number, number]) ?? [0, 0];
    const [x2, y2] = (props.to as [number, number]) ?? [100, 100];
    const stroke = (style.stroke as string) ?? '#000';
    const width = (style.strokeWidth as number) ?? 1;
    return `<line x1="${x1}" y1="${y1}" x2="${x2}" y2="${y2}" stroke="${stroke}" stroke-width="${width}"${tf}/>`;
  }

  private renderPath(props: Record<string, unknown>, style: Record<string, unknown>, tf: string): string {
    const d = (props.content as string) ?? '';
    return `<path d="${d}"${this.styleAttrs(style)}${tf}/>`;
  }

  private renderGroup(children: ShapeDict[], style: Record<string, unknown>, tf: string): string {
    const inner = children.map(c => this.renderShape(c)).join('');
    return `<g${this.styleAttrs(style)}${tf}>${inner}</g>`;
  }

  private renderLayout(props: Record<string, unknown>, children: ShapeDict[], _style: Record<string, unknown>, tf: string): string {
    const direction = (props.direction as string) ?? 'vertical';
    const gap = (props.gap as number) ?? 0;
    const at = (props.at as [number, number]) ?? [0, 0];

    let inner = '';
    let offset = 0;

    for (const c of children) {
      const childOffset: [number, number] = direction === 'vertical'
        ? [at[0], at[1] + offset]
        : [at[0] + offset, at[1]];

      inner += this.renderShape(c, childOffset);

      const size = c.props.size as [number, number] | undefined;
      const radius = c.props.radius as number | undefined;

      if (direction === 'vertical') {
        const h = size ? size[1] : (radius ? radius * 2 : 40);
        offset += h + gap;
      } else {
        const w = size ? size[0] : (radius ? radius * 2 : 40);
        offset += w + gap;
      }
    }

    return `<g${tf}>${inner}</g>`;
  }

  private buildTransform(transform: Record<string, unknown>, x = 0, y = 0): string {
    const parts: string[] = [];

    if (transform.translate) {
      const [tx, ty] = transform.translate as [number, number];
      parts.push(`translate(${tx},${ty})`);
    }
    if (transform.rotate) {
      const r = transform.rotate as number;
      const origin = (transform.origin as [number, number]) ?? [x, y];
      parts.push(`rotate(${r},${origin[0]},${origin[1]})`);
    }
    if (transform.scale) {
      const [sx, sy] = transform.scale as [number, number];
      parts.push(`scale(${sx},${sy})`);
    }

    return parts.length ? ` transform="${parts.join(' ')}"` : '';
  }

  private styleAttrs(style: Record<string, unknown>): string {
    const parts: string[] = [];

    const grad = style.gradient as GradientDef | undefined;
    if (grad) {
      const gradId = this.addGradient(grad);
      parts.push(`fill="url(#${gradId})"`);
    } else if (style.fill) {
      parts.push(`fill="${style.fill}"`);
    }

    if (style.stroke) {
      parts.push(`stroke="${style.stroke}"`);
      parts.push(`stroke-width="${style.strokeWidth ?? 1}"`);
    }

    if (style.opacity !== undefined && (style.opacity as number) < 1) {
      parts.push(`opacity="${style.opacity}"`);
    }

    const shadow = style.shadow as ShadowDef | undefined;
    if (shadow) {
      const filterId = this.addShadowFilter(shadow);
      parts.push(`filter="url(#${filterId})"`);
    }

    return parts.length ? ` ${parts.join(' ')}` : '';
  }

  private addGradient(grad: GradientDef): string {
    const id = this.nextId();

    if (grad.type === 'linear') {
      const angle = grad.angle ?? 90;
      const rad = (angle - 90) * Math.PI / 180;
      const x2 = 50 + 50 * Math.cos(rad);
      const y2 = 50 + 50 * Math.sin(rad);
      this.defs.push(
        `<linearGradient id="${id}" x1="0%" y1="0%" x2="${x2.toFixed(1)}%" y2="${y2.toFixed(1)}%">` +
        `<stop offset="0%" stop-color="${grad.from}"/>` +
        `<stop offset="100%" stop-color="${grad.to}"/>` +
        `</linearGradient>`
      );
    } else {
      this.defs.push(
        `<radialGradient id="${id}">` +
        `<stop offset="0%" stop-color="${grad.from}"/>` +
        `<stop offset="100%" stop-color="${grad.to}"/>` +
        `</radialGradient>`
      );
    }
    return id;
  }

  private addShadowFilter(shadow: ShadowDef): string {
    const id = this.nextId();
    this.defs.push(
      `<filter id="${id}" x="-50%" y="-50%" width="200%" height="200%">` +
      `<feDropShadow dx="${shadow.x}" dy="${shadow.y}" stdDeviation="${shadow.blur}" flood-color="${shadow.color}"/>` +
      `</filter>`
    );
    return id;
  }

  private escapeHtml(s: string): string {
    return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
  }
}

export class Interpreter {
  private state = new SceneState();

  eval(source: string, Lexer: typeof import('./lexer').Lexer, Parser: typeof import('./parser').Parser): SceneState {
    this.state = new SceneState();
    const lexer = new Lexer(source);
    const parser = new Parser(lexer.tokenize());
    const ast = parser.parse();
    this.evalNode(ast);
    return this.state;
  }

  private evalNode(node: Node): void {
    switch (node.type) {
      case 'scene':
        for (const child of node.children) this.evalNode(child);
        break;
      case 'canvas':
        this.state.canvas = node.value as Canvas;
        break;
      case 'shape':
        this.addShape(node.value as Shape);
        break;
    }
  }

  private addShape(shape: Shape): void {
    const idx = this.state.shapes.length;
    const dict = this.shapeToDict(shape, idx);
    this.state.shapes.push(dict);
    this.state.invalidate();
  }

  private shapeToDict(shape: Shape, idx: number): ShapeDict {
    const id = shape.id ?? computeId(idx, shape.kind, shape.props).toString(16);
    return {
      id,
      kind: shape.kind,
      props: shape.props,
      style: {
        fill: shape.style.fill ?? shape.props.fill,
        stroke: shape.style.stroke,
        strokeWidth: shape.style.strokeWidth,
        opacity: shape.style.opacity,
        corner: shape.style.corner,
        font: shape.style.font,
        fontSize: shape.style.fontSize,
        fontWeight: shape.style.fontWeight,
        textAnchor: shape.style.textAnchor,
        shadow: shape.style.shadow,
        gradient: shape.style.gradient,
      },
      transform: {
        translate: shape.transform.translate,
        rotate: shape.transform.rotate,
        scale: shape.transform.scale,
        origin: shape.transform.origin,
      },
      children: shape.children.map((c, i) => this.shapeToDict(c, i)),
    };
  }
}

