import type { Token, Node, Canvas, Shape, Style, Transform, GradientDef } from './types';
import { defaultStyle, defaultTransform, defaultCanvas } from './types';

const SHAPES = new Set(['rect', 'circle', 'ellipse', 'line', 'path', 'polygon', 'text', 'image']);
const STYLE_PROPS = new Set(['fill', 'stroke', 'opacity', 'corner', 'shadow', 'gradient', 'blur']);
const TEXT_PROPS = new Set(['font', 'bold', 'italic', 'center', 'middle', 'end']);
const TRANSFORM_PROPS = new Set(['translate', 'rotate', 'scale', 'origin']);

export class Parser {
  private tokens: Token[];
  private pos = 0;
  private variables = new Map<string, Token>();

  constructor(tokens: Iterable<Token>) {
    this.tokens = [...tokens];
  }

  private get current(): Token | undefined { return this.tokens[this.pos]; }
  private get peekNext(): Token | undefined { return this.tokens[this.pos + 1]; }
  private advance(): Token | undefined { return this.tokens[this.pos++]; }

  private match(...types: string[]): boolean {
    return this.current !== undefined && types.includes(this.current.type);
  }

  private skipNewlines(): void {
    while (this.match('NEWLINE')) this.advance();
  }

  private resolve(tok: Token): string | number | [number, number] {
    if (tok.type === 'VAR') {
      const resolved = this.variables.get(tok.value as string);
      return resolved ? resolved.value : tok.value;
    }
    return tok.value;
  }

  parse(): Node {
    const root: Node = { type: 'scene', children: [] };
    this.skipNewlines();

    while (this.current && !this.match('EOF')) {
      const node = this.parseStatement();
      if (node) root.children.push(node);
      this.skipNewlines();
    }
    return root;
  }

  private parseStatement(): Node | undefined {
    if (!this.current) return;

    if (this.match('VAR')) return this.parseVariable();
    if (!this.match('IDENT')) { this.advance(); return; }

    const cmd = this.advance()!.value as string;

    if (cmd === 'canvas') return this.parseCanvas();
    if (cmd === 'group') return this.parseGroup();
    if (cmd === 'stack' || cmd === 'row') return this.parseLayout(cmd);
    if (SHAPES.has(cmd)) return this.parseShape(cmd);
  }

  private parseVariable(): Node {
    const name = this.advance()!.value as string;
    if (this.match('EQUALS')) {
      this.advance();
      if (this.current) {
        this.variables.set(name, this.current);
        this.advance();
      }
    }
    return { type: 'variable', value: { name, value: this.variables.get(name) }, children: [] };
  }

  private parseCanvas(): Node {
    const canvas = defaultCanvas();

    if (this.match('PAIR')) {
      const [w, h] = this.advance()!.value as [number, number];
      canvas.width = w;
      canvas.height = h;
    }

    while (this.match('IDENT')) {
      const prop = this.advance()!.value;
      if (prop === 'fill' && this.current) {
        canvas.fill = this.resolve(this.advance()!) as string;
      }
    }
    return { type: 'canvas', value: canvas, children: [] };
  }

  private parseGroup(): Node {
    let name: string | undefined;
    if (this.match('STRING')) name = this.advance()!.value as string;

    const shape: Shape = {
      kind: 'group',
      props: { name },
      style: defaultStyle(),
      transform: defaultTransform(),
      children: [],
    };

    this.skipNewlines();
    if (this.match('INDENT')) {
      this.advance();
      this.parseBlock(shape);
    }
    return { type: 'shape', value: shape, children: [] };
  }

  private parseLayout(kind: string): Node {
    const props: Record<string, unknown> = {
      direction: kind === 'stack' ? 'vertical' : 'horizontal',
      gap: 0,
    };

    while (this.match('IDENT')) {
      const prop = this.advance()!.value as string;
      if (prop === 'vertical' || prop === 'horizontal') props.direction = prop;
      else if (prop === 'gap' && this.match('NUMBER')) props.gap = this.advance()!.value;
      else if (prop === 'at' && this.match('PAIR')) props.at = this.advance()!.value;
    }

    const shape: Shape = {
      kind: 'layout',
      props,
      style: defaultStyle(),
      transform: defaultTransform(),
      children: [],
    };

    this.skipNewlines();
    if (this.match('INDENT')) {
      this.advance();
      this.parseBlock(shape);
    }
    return { type: 'shape', value: shape, children: [] };
  }

  private parseShape(kind: string): Node {
    const props: Record<string, unknown> = {};

    while (this.current && !this.match('NEWLINE', 'EOF')) {
      if (this.match('PAIR')) {
        const val = this.advance()!.value;
        if (!('at' in props)) props.at = val;
        else if (!('size' in props)) props.size = val;
      } else if (this.match('NUMBER')) {
        const val = this.advance()!.value;
        if (kind === 'circle' && !('radius' in props)) props.radius = val;
        else if (!('width' in props)) props.width = val;
      } else if (this.match('STRING')) {
        props.content = this.advance()!.value;
      } else if (this.match('IDENT')) {
        const key = this.advance()!.value as string;
        if (key === 'at' && this.match('PAIR')) props.at = this.advance()!.value;
        else if (key === 'size' && this.match('PAIR')) props.size = this.advance()!.value;
        else if (key === 'radius' && this.match('NUMBER')) props.radius = this.advance()!.value;
        else if (key === 'from' && this.match('PAIR')) props.from = this.advance()!.value;
        else if (key === 'to' && this.match('PAIR')) props.to = this.advance()!.value;
      } else if (this.match('COLOR', 'VAR')) {
        if (!('fill' in props)) props.fill = this.resolve(this.advance()!);
      } else {
        this.advance();
      }
    }

    const shape: Shape = {
      kind,
      props,
      style: defaultStyle(),
      transform: defaultTransform(),
      children: [],
    };

    this.skipNewlines();
    if (this.match('INDENT')) {
      this.advance();
      this.parseBlock(shape);
    }
    return { type: 'shape', value: shape, children: [] };
  }

  private parseBlock(shape: Shape): void {
    while (this.current && !this.match('DEDENT')) {
      this.skipNewlines();
      if (!this.current || this.match('DEDENT')) break;

      if (this.match('IDENT')) {
        const prop = this.current.value as string;

        if (SHAPES.has(prop)) {
          const child = this.parseStatement();
          if (child?.value) shape.children.push(child.value as Shape);
        } else if (STYLE_PROPS.has(prop)) {
          this.parseStyleProp(shape.style);
        } else if (TEXT_PROPS.has(prop)) {
          this.parseTextProp(shape.style);
        } else if (TRANSFORM_PROPS.has(prop)) {
          this.parseTransformProp(shape.transform);
        } else if (prop === 'width' && this.peekNext?.type === 'NUMBER') {
          this.advance();
          shape.style.strokeWidth = this.advance()!.value as number;
        } else {
          this.advance();
        }
      } else {
        this.advance();
      }
    }

    if (this.match('DEDENT')) this.advance();
  }

  private parseStyleProp(style: Style): void {
    const prop = this.advance()!.value as string;

    switch (prop) {
      case 'fill':
        if (this.match('COLOR', 'VAR', 'IDENT'))
          style.fill = this.resolve(this.advance()!) as string;
        break;
      case 'stroke':
        if (this.match('COLOR', 'VAR'))
          style.stroke = this.resolve(this.advance()!) as string;
        if (this.match('NUMBER'))
          style.strokeWidth = this.advance()!.value as number;
        if (this.match('IDENT') && this.current!.value === 'width') {
          this.advance();
          if (this.match('NUMBER')) style.strokeWidth = this.advance()!.value as number;
        }
        break;
      case 'opacity':
        if (this.match('NUMBER')) style.opacity = this.advance()!.value as number;
        break;
      case 'corner':
        if (this.match('NUMBER')) style.corner = this.advance()!.value as number;
        break;
      case 'shadow':
        style.shadow = this.parseShadow();
        break;
      case 'gradient':
        style.gradient = this.parseGradient();
        break;
    }
  }

  private parseTextProp(style: Style): void {
    const prop = this.advance()!.value as string;

    switch (prop) {
      case 'font':
        if (this.match('STRING')) style.font = this.advance()!.value as string;
        if (this.match('NUMBER')) style.fontSize = this.advance()!.value as number;
        break;
      case 'bold': style.fontWeight = 'bold'; break;
      case 'italic': style.fontWeight = 'italic'; break;
      case 'center': style.textAnchor = 'middle'; break;
      case 'end': style.textAnchor = 'end'; break;
    }
  }

  private parseTransformProp(transform: Transform): void {
    const prop = this.advance()!.value as string;

    switch (prop) {
      case 'translate':
        if (this.match('PAIR')) transform.translate = this.advance()!.value as [number, number];
        break;
      case 'rotate':
        if (this.match('NUMBER')) transform.rotate = this.advance()!.value as number;
        break;
      case 'scale':
        if (this.match('PAIR')) transform.scale = this.advance()!.value as [number, number];
        else if (this.match('NUMBER')) {
          const s = this.advance()!.value as number;
          transform.scale = [s, s];
        }
        break;
      case 'origin':
        if (this.match('PAIR')) transform.origin = this.advance()!.value as [number, number];
        break;
    }
  }

  private parseShadow() {
    const shadow = { x: 0, y: 4, blur: 8, color: '#0004' };
    if (this.match('PAIR')) [shadow.x, shadow.y] = this.advance()!.value as [number, number];
    if (this.match('NUMBER')) shadow.blur = this.advance()!.value as number;
    if (this.match('COLOR')) shadow.color = this.advance()!.value as string;
    return shadow;
  }

  private parseGradient(): GradientDef {
    const gradient: GradientDef = { type: 'linear', from: '#fff', to: '#000', angle: 90 };

    while (this.match('IDENT', 'COLOR', 'NUMBER')) {
      if (this.match('IDENT')) {
        const val = this.advance()!.value as string;
        if (val === 'linear' || val === 'radial') gradient.type = val;
        else if (val === 'from' && this.match('COLOR')) gradient.from = this.advance()!.value as string;
        else if (val === 'to' && this.match('COLOR')) gradient.to = this.advance()!.value as string;
      } else if (this.match('COLOR')) {
        if (gradient.from === '#fff') gradient.from = this.advance()!.value as string;
        else gradient.to = this.advance()!.value as string;
      } else if (this.match('NUMBER')) {
        gradient.angle = this.advance()!.value as number;
      }
    }
    return gradient;
  }
}
