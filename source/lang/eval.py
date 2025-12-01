"""Interpreter for the visual DSL using Rust core for lexing, parsing, and rendering."""

import logging
from dataclasses import dataclass, field
from .types import Node, Canvas, Shape, Style, Transform
from .errors import ErrorCode, ErrorInfo, ErrorList, RenderError

logger = logging.getLogger(__name__)

# Rust core is REQUIRED - no Python fallback
try:
    import iconoglott_core as rust
except ImportError as e:
    raise ImportError(
        "Rust core module 'iconoglott_core' not found. "
        "Build it with: cd source/core && maturin develop --release"
    ) from e

# Validate Rust core exports
for attr in ['Lexer', 'Parser', 'Scene']:
    if not hasattr(rust, attr):
        raise ImportError(f"Rust core module is incomplete - missing {attr} class")


@dataclass(slots=True)
class SceneState:
    """Evaluated scene state."""
    canvas: Canvas = field(default_factory=Canvas)
    shapes: list[dict] = field(default_factory=list)
    errors: list = field(default_factory=list)
    error_infos: ErrorList = field(default_factory=list)
    _def_id: int = 0
    _gradients: list[tuple[str, dict]] = field(default_factory=list)
    _filters: list[tuple[str, dict]] = field(default_factory=list)

    def next_id(self) -> str:
        self._def_id += 1
        return f"d{self._def_id}"
    
    def add_error(self, code: ErrorCode, msg: str, line: int = 0, col: int = 0):
        """Add a structured error to the scene state."""
        self.error_infos.append(ErrorInfo(code, msg, line, col))

    def to_svg(self) -> str:
        """Render scene to SVG using Rust core."""
        try:
            scene = rust.Scene(self.canvas.width, self.canvas.height, self.canvas.fill)
            
            # Add shapes first (this populates _gradients and _filters)
            for s in self.shapes:
                self._add_shape(scene, s, (0, 0))
            
            # Now add collected gradients and filters
            for gid, grad in self._gradients:
                scene.add_gradient(rust.Gradient(
                    gid, grad.get('type', 'linear'),
                    grad.get('from', '#fff'), grad.get('to', '#000'),
                    float(grad.get('angle', 90.0))
                ))
            
            for fid, filt in self._filters:
                scene.add_filter(rust.Filter(
                    fid, filt.get('kind', 'shadow'),
                    float(filt.get('x', 0.0)), float(filt.get('y', 4.0)),
                    float(filt.get('blur', 8.0)), filt.get('color', '#0004')
                ))
            
            return scene.to_svg()
        except Exception as e:
            logger.exception("Rust rendering failed")
            self.add_error(ErrorCode.RENDER_RUST_ERROR, f"Render failed: {e}")
            return self._error_svg(str(e))

    def _error_svg(self, msg: str) -> str:
        """Return an error SVG when rendering fails."""
        escaped = msg.replace('&', '&amp;').replace('<', '&lt;').replace('>', '&gt;')
        return (
            f'<svg xmlns="http://www.w3.org/2000/svg" width="{self.canvas.width}" '
            f'height="{self.canvas.height}"><rect width="100%" height="100%" fill="#1a1a2e"/>'
            f'<text x="20" y="30" fill="#f85149" font-family="monospace" font-size="12">'
            f'Render Error: {escaped}</text></svg>'
        )

    def _add_shape(self, scene, s: dict, offset: tuple):
        """Add shape to Rust scene."""
        kind = s['kind']
        props = s['props']
        style = s['style']
        children = s.get('children', [])
        transform = self._make_transform(s.get('transform', {}))
        
        x, y = props.get('at', (0, 0))
        x, y = float(x + offset[0]), float(y + offset[1])
        
        rust_style = self._make_style(style)
        
        match kind:
            case 'rect':
                w, h = props.get('size', (100, 100))
                corner = float(style.get('corner', 0))
                scene.add_rect(rust.Rect(x, y, float(w), float(h), corner, rust_style, transform))
            case 'circle':
                r = float(props.get('radius', 50))
                scene.add_circle(rust.Circle(x, y, r, rust_style, transform))
            case 'ellipse':
                if 'radius' in props:
                    r = props['radius']
                    rx, ry = (float(r), float(r)) if isinstance(r, (int, float)) else (float(r[0]), float(r[1]))
                elif 'size' in props:
                    rx, ry = float(props['size'][0]), float(props['size'][1])
                else:
                    rx, ry = 50.0, 30.0
                scene.add_ellipse(rust.Ellipse(x, y, rx, ry, rust_style, transform))
            case 'line':
                x1, y1 = props.get('from', (0, 0))
                x2, y2 = props.get('to', (100, 100))
                scene.add_line(rust.Line(float(x1), float(y1), float(x2), float(y2), rust_style, transform))
            case 'path':
                d = props.get('d', props.get('content', ''))
                scene.add_path(rust.Path(str(d), rust_style, transform))
            case 'polygon':
                points = [(float(px), float(py)) for px, py in props.get('points', [])]
                scene.add_polygon(rust.Polygon(points, rust_style, transform))
            case 'text':
                content = str(props.get('content', ''))
                font = str(style.get('font') or 'system-ui')
                size = float(style.get('font_size', 16))
                weight = str(style.get('font_weight', 'normal'))
                anchor = str(style.get('text_anchor', 'start'))
                scene.add_text(rust.Text(x, y, content, font, size, weight, anchor, rust_style, transform))
            case 'image':
                w, h = props.get('size', (100, 100))
                href = str(props.get('href', ''))
                scene.add_image(rust.Image(x, y, float(w), float(h), href, transform))
            case 'group':
                for c in children:
                    self._add_shape(scene, c, (0, 0))
            case 'layout':
                self._add_layout(scene, props, children)

    def _make_style(self, style: dict) -> 'rust.Style':
        """Convert style dict to Rust Style."""
        fill = style.get('fill')
        stroke = style.get('stroke')
        
        # Handle gradient - register it and use url reference
        if grad := style.get('gradient'):
            gid = self.next_id()
            self._gradients.append((gid, grad))
            fill = f"url(#{gid})"
        
        # Handle shadow filter
        if shadow := style.get('shadow'):
            fid = self.next_id()
            self._filters.append((fid, {'kind': 'shadow', **shadow}))
        
        return rust.Style(
            fill=fill,
            stroke=stroke,
            stroke_width=float(style.get('stroke_width', 1.0)),
            opacity=float(style.get('opacity', 1.0)),
            corner=float(style.get('corner', 0.0))
        )

    def _make_transform(self, transform: dict) -> str | None:
        """Convert transform dict to SVG transform string."""
        if not transform:
            return None
        parts = []
        if translate := transform.get('translate'):
            tx, ty = (translate, 0) if isinstance(translate, (int, float)) else translate
            parts.append(f"translate({tx} {ty})")
        if (rotate := transform.get('rotate')) and rotate != 0:
            if origin := transform.get('origin'):
                ox, oy = origin
                parts.append(f"rotate({rotate} {ox} {oy})")
            else:
                parts.append(f"rotate({rotate})")
        if scale := transform.get('scale'):
            sx, sy = (scale, scale) if isinstance(scale, (int, float)) else scale
            parts.append(f"scale({sx} {sy})")
        return ' '.join(parts) if parts else None

    def _add_layout(self, scene, props: dict, children: list):
        """Add layout children with proper positioning."""
        direction = props.get('direction', 'vertical')
        gap = float(props.get('gap', 0))
        x, y = props.get('at', (0, 0))
        x, y = float(x), float(y)
        
        offset = 0.0
        for c in children:
            if direction == 'vertical':
                self._add_shape(scene, c, (x, y + offset))
                offset += self._measure_height(c) + gap
            else:
                self._add_shape(scene, c, (x + offset, y))
                offset += self._measure_width(c) + gap

    def _measure_width(self, s: dict) -> float:
        """Measure shape width for layout."""
        props = s['props']
        kind = s['kind']
        
        if 'size' in props:
            return float(props['size'][0])
        if 'radius' in props:
            r = props['radius']
            return float(r[0] if isinstance(r, tuple) else r) * 2
        if kind == 'text':
            content = props.get('content', '')
            size = float(s['style'].get('font_size', 16))
            return len(str(content)) * size * 0.6
        if kind == 'layout':
            return sum(self._measure_width(c) + float(props.get('gap', 0)) for c in s.get('children', []))
        return 40.0

    def _measure_height(self, s: dict) -> float:
        """Measure shape height for layout."""
        props = s['props']
        kind = s['kind']
        
        if 'size' in props:
            return float(props['size'][1])
        if 'radius' in props:
            r = props['radius']
            return float(r[1] if isinstance(r, tuple) else r) * 2
        if kind == 'text':
            return float(s['style'].get('font_size', 16)) * 1.2
        if kind == 'layout':
            gap = float(props.get('gap', 0))
            if props.get('direction') == 'vertical':
                return sum(self._measure_height(c) + gap for c in s.get('children', []))
            return max((self._measure_height(c) for c in s.get('children', [])), default=0.0)
        return 40.0


class Interpreter:
    """Evaluate DSL AST into renderable state using Rust core."""

    def __init__(self):
        self.state = SceneState()

    def eval(self, source: str) -> SceneState:
        """Interpret source code using Rust lexer/parser and return scene state."""
        self.state = SceneState()
        
        # Tokenize with Rust lexer
        lexer = rust.Lexer(source)
        tokens = lexer.py_tokenize()
        
        # Parse with Rust parser - returns native Python objects directly
        parser = rust.Parser(tokens)
        ast = parser.parse_py()
        
        # Collect parse errors
        for err in parser.get_errors():
            self.state.add_error(
                ErrorCode.PARSE_UNEXPECTED_TOKEN,
                err.message,
                err.line,
                err.col
            )
        
        # Evaluate the AST
        try:
            self._eval_ast(ast)
        except Exception as e:
            logger.exception("Evaluation failed")
            self.state.add_error(ErrorCode.EVAL_INVALID_SHAPE, f"Evaluation error: {e}")
        
        return self.state

    def _eval_ast(self, ast: dict):
        """Recursively evaluate AST nodes."""
        if 'Scene' in ast:
            for child in ast['Scene']:
                self._eval_ast(child)
        elif 'Canvas' in ast:
            c = ast['Canvas']
            self.state.canvas = Canvas(c['width'], c['height'], c['fill'])
        elif 'Shape' in ast:
            self._add_shape(ast['Shape'])
        elif 'Variable' in ast:
            pass  # Variables handled during parsing

    def _add_shape(self, shape: dict):
        """Add shape to scene state."""
        self.state.shapes.append(self._shape_to_dict(shape))


    def _shape_to_dict(self, shape: dict) -> dict:
        """Convert Rust AST Shape to dict for rendering."""
        style = shape.get('style', {})
        transform = shape.get('transform', {})
        props = shape.get('props', {})
        
        # Get fill from style or props
        fill = style.get('fill')
        if not fill and 'fill' in props:
            fill = props['fill']
        
        return {
            'kind': shape['kind'],
            'props': props,
            'style': {
                'fill': fill,
                'stroke': style.get('stroke'),
                'stroke_width': style.get('stroke_width', 1.0),
                'opacity': style.get('opacity', 1.0),
                'corner': style.get('corner', 0.0),
                'font': style.get('font'),
                'font_size': style.get('font_size', 16.0),
                'font_weight': style.get('font_weight', 'normal'),
                'text_anchor': style.get('text_anchor', 'start'),
                'shadow': shape.get('shadow'),
                'gradient': shape.get('gradient'),
            },
            'transform': {
                'translate': transform.get('translate'),
                'rotate': transform.get('rotate', 0.0),
                'scale': transform.get('scale'),
                'origin': transform.get('origin'),
            },
            'children': [self._shape_to_dict(c) for c in shape.get('children', [])],
        }
