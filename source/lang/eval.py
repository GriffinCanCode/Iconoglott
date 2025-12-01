"""Interpreter for the visual DSL."""

from dataclasses import dataclass, field
from .types import Node, Canvas, Shape, Style, Transform
from .lexer import Lexer
from .parser import Parser


@dataclass(slots=True)
class SceneState:
    """Evaluated scene state."""
    canvas: Canvas = field(default_factory=Canvas)
    defs: list[str] = field(default_factory=list)
    shapes: list[dict] = field(default_factory=list)
    _def_id: int = 0

    def next_id(self) -> str:
        self._def_id += 1
        return f"d{self._def_id}"

    def to_svg(self) -> str:
        """Render scene to SVG."""
        # Pre-render all shapes to collect defs
        rendered = [self._render_shape(s) for s in self.shapes]
        
        svg = (
            f'<svg xmlns="http://www.w3.org/2000/svg" '
            f'width="{self.canvas.width}" height="{self.canvas.height}">'
        )
        
        # Background
        svg += f'<rect width="100%" height="100%" fill="{self.canvas.fill}"/>'
        
        # Defs (gradients, filters) - now populated from pre-render
        if self.defs:
            svg += '<defs>' + ''.join(self.defs) + '</defs>'
        
        # Add pre-rendered shapes
        svg += ''.join(rendered)
            
        return svg + '</svg>'

    def _render_shape(self, s: dict, offset: tuple = (0, 0)) -> str:
        """Render shape to SVG element."""
        kind = s['kind']
        props = s['props']
        style = s['style']
        transform = s.get('transform', {})
        children = s.get('children', [])

        # Get position with offset
        x, y = props.get('at', (0, 0))
        x, y = x + offset[0], y + offset[1]

        # Build transform string
        tf = self._build_transform(transform, x, y)

        match kind:
            case 'rect':
                return self._render_rect(props, style, tf, x, y)
            case 'circle':
                return self._render_circle(props, style, tf, x, y)
            case 'ellipse':
                return self._render_ellipse(props, style, tf, x, y)
            case 'text':
                return self._render_text(props, style, tf, x, y)
            case 'line':
                return self._render_line(props, style, tf)
            case 'path':
                return self._render_path(props, style, tf)
            case 'group':
                return self._render_group(children, style, tf)
            case 'layout':
                return self._render_layout(props, children, style, tf)
            case _:
                return ''

    def _render_rect(self, props: dict, style: dict, tf: str, x: float, y: float) -> str:
        w, h = props.get('size', (100, 100))
        corner = style.get('corner', 0)
        attrs = f'x="{x}" y="{y}" width="{w}" height="{h}"'
        if corner:
            attrs += f' rx="{corner}"'
        return f'<rect {attrs}{self._style_attrs(style)}{tf}/>'

    def _render_circle(self, props: dict, style: dict, tf: str, x: float, y: float) -> str:
        r = props.get('radius', 50)
        return f'<circle cx="{x}" cy="{y}" r="{r}"{self._style_attrs(style)}{tf}/>'

    def _render_ellipse(self, props: dict, style: dict, tf: str, x: float, y: float) -> str:
        rx, ry = props.get('size', (50, 30))
        return f'<ellipse cx="{x}" cy="{y}" rx="{rx}" ry="{ry}"{self._style_attrs(style)}{tf}/>'

    def _render_text(self, props: dict, style: dict, tf: str, x: float, y: float) -> str:
        content = props.get('content', '')
        font = style.get('font', 'system-ui')
        size = style.get('font_size', 16)
        weight = style.get('font_weight', 'normal')
        anchor = style.get('text_anchor', 'start')
        fill = style.get('fill', '#000')
        
        attrs = (
            f'x="{x}" y="{y}" '
            f'font-family="{font}" font-size="{size}" font-weight="{weight}" '
            f'text-anchor="{anchor}" fill="{fill}"'
        )
        return f'<text {attrs}{tf}>{content}</text>'

    def _render_line(self, props: dict, style: dict, tf: str) -> str:
        x1, y1 = props.get('from', (0, 0))
        x2, y2 = props.get('to', (100, 100))
        stroke = style.get('stroke', '#000')
        width = style.get('stroke_width', 1)
        return f'<line x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}" stroke="{stroke}" stroke-width="{width}"{tf}/>'

    def _render_path(self, props: dict, style: dict, tf: str) -> str:
        d = props.get('content', '')
        return f'<path d="{d}"{self._style_attrs(style)}{tf}/>'

    def _render_group(self, children: list, style: dict, tf: str) -> str:
        inner = ''.join(self._render_shape(c) for c in children)
        return f'<g{self._style_attrs(style)}{tf}>{inner}</g>'

    def _render_layout(self, props: dict, children: list, style: dict, tf: str) -> str:
        direction = props.get('direction', 'vertical')
        gap = props.get('gap', 0)
        x, y = props.get('at', (0, 0))
        
        inner = ''
        offset = 0
        for c in children:
            if direction == 'vertical':
                inner += self._render_shape(c, (x, y + offset))
                h = c['props'].get('size', (0, 40))[1] if 'size' in c['props'] else c['props'].get('radius', 20) * 2
                offset += h + gap
            else:
                inner += self._render_shape(c, (x + offset, y))
                w = c['props'].get('size', (40, 0))[0] if 'size' in c['props'] else c['props'].get('radius', 20) * 2
                offset += w + gap
                
        return f'<g{tf}>{inner}</g>'

    def _build_transform(self, transform: dict, x: float = 0, y: float = 0) -> str:
        """Build SVG transform attribute."""
        parts = []
        
        if t := transform.get('translate'):
            parts.append(f'translate({t[0]},{t[1]})')
        if r := transform.get('rotate'):
            ox, oy = transform.get('origin', (x, y))
            parts.append(f'rotate({r},{ox},{oy})')
        if s := transform.get('scale'):
            parts.append(f'scale({s[0]},{s[1]})')
            
        return f' transform="{" ".join(parts)}"' if parts else ''

    def _style_attrs(self, style: dict) -> str:
        """Build SVG style attributes."""
        parts = []
        
        # Handle gradient fill
        if grad := style.get('gradient'):
            grad_id = self._add_gradient(grad)
            parts.append(f'fill="url(#{grad_id})"')
        elif fill := style.get('fill'):
            parts.append(f'fill="{fill}"')
            
        if stroke := style.get('stroke'):
            parts.append(f'stroke="{stroke}"')
            parts.append(f'stroke-width="{style.get("stroke_width", 1)}"')
            
        if (opacity := style.get('opacity')) and opacity < 1:
            parts.append(f'opacity="{opacity}"')
            
        # Handle shadow via filter
        if shadow := style.get('shadow'):
            filter_id = self._add_shadow_filter(shadow)
            parts.append(f'filter="url(#{filter_id})"')
            
        return (' ' + ' '.join(parts)) if parts else ''

    def _add_gradient(self, grad: dict) -> str:
        """Add gradient to defs and return ID."""
        gid = self.next_id()
        gtype = grad.get('type', 'linear')
        
        if gtype == 'linear':
            angle = grad.get('angle', 90)
            import math
            rad = math.radians(angle - 90)  # Adjust for SVG coordinate system
            x2 = 50 + 50 * math.cos(rad)
            y2 = 50 + 50 * math.sin(rad)
            self.defs.append(
                f'<linearGradient id="{gid}" x1="0%" y1="0%" x2="{x2:.1f}%" y2="{y2:.1f}%">'
                f'<stop offset="0%" stop-color="{grad["from"]}"/>'
                f'<stop offset="100%" stop-color="{grad["to"]}"/>'
                f'</linearGradient>'
            )
        else:
            self.defs.append(
                f'<radialGradient id="{gid}">'
                f'<stop offset="0%" stop-color="{grad["from"]}"/>'
                f'<stop offset="100%" stop-color="{grad["to"]}"/>'
                f'</radialGradient>'
            )
        return gid

    def _add_shadow_filter(self, shadow: dict) -> str:
        """Add drop shadow filter to defs and return ID."""
        fid = self.next_id()
        self.defs.append(
            f'<filter id="{fid}" x="-50%" y="-50%" width="200%" height="200%">'
            f'<feDropShadow dx="{shadow["x"]}" dy="{shadow["y"]}" '
            f'stdDeviation="{shadow["blur"]}" flood-color="{shadow["color"]}"/>'
            f'</filter>'
        )
        return fid


class Interpreter:
    """Evaluate DSL AST into renderable state."""

    def __init__(self):
        self.state = SceneState()

    def eval(self, source: str) -> SceneState:
        """Interpret source code and return scene state."""
        self.state = SceneState()
        lexer = Lexer(source)
        parser = Parser(lexer.tokenize())
        ast = parser.parse()
        self._eval_node(ast)
        return self.state

    def _eval_node(self, node: Node):
        """Recursively evaluate AST nodes."""
        match node.type:
            case "scene":
                for child in node.children:
                    self._eval_node(child)
            case "canvas":
                self.state.canvas = node.value
            case "variable":
                pass  # Already handled in parser
            case "shape":
                self._add_shape(node.value)

    def _add_shape(self, shape: Shape):
        """Add shape to scene state."""
        self.state.shapes.append(self._shape_to_dict(shape))

    def _shape_to_dict(self, shape: Shape) -> dict:
        """Convert Shape to dict for rendering."""
        return {
            'kind': shape.kind,
            'props': shape.props,
            'style': {
                'fill': shape.style.fill or shape.props.get('fill'),
                'stroke': shape.style.stroke,
                'stroke_width': shape.style.stroke_width,
                'opacity': shape.style.opacity,
                'corner': shape.style.corner,
                'font': shape.style.font,
                'font_size': shape.style.font_size,
                'font_weight': shape.style.font_weight,
                'text_anchor': shape.style.text_anchor,
                'shadow': shape.style.shadow,
                'gradient': shape.style.gradient,
            },
            'transform': {
                'translate': shape.transform.translate,
                'rotate': shape.transform.rotate,
                'scale': shape.transform.scale,
                'origin': shape.transform.origin,
            },
            'children': [self._shape_to_dict(c) for c in shape.children],
        }
