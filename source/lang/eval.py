"""Interpreter for the visual DSL using Rust core for lexing, parsing, and rendering."""

import logging
from dataclasses import dataclass, field
from .types import Node, Canvas, Shape, Style, Transform, CANVAS_SIZES
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
            # Get CanvasSize enum from Rust
            size = getattr(rust.CanvasSize, self.canvas.size.capitalize(), None)
            if size is None:
                size = rust.CanvasSize.from_name(self.canvas.size) or rust.CanvasSize.Medium
            scene = rust.Scene(size, self.canvas.fill)
            
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
            case 'graph':
                self._add_graph_to_scene(scene, s)

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

    def _add_graph_to_scene(self, scene, g: dict):
        """Add graph nodes and edges to the Rust scene."""
        nodes = g.get('nodes', [])
        edges = g.get('edges', [])
        
        # Render edges first (behind nodes)
        for e in edges:
            from_pt = e['from_pt']
            to_pt = e['to_pt']
            edge_style = e.get('edge_style', 'straight')
            arrow = e.get('arrow', 'forward')
            stroke = e.get('stroke', '#333')
            stroke_width = float(e.get('stroke_width', 2.0))
            label = e.get('label')
            
            # Create edge as path with proper styling
            path_d = self._compute_edge_path(from_pt, to_pt, edge_style)
            edge_style_obj = rust.Style(
                stroke=stroke,
                stroke_width=stroke_width,
                opacity=1.0
            )
            scene.add_path(rust.Path(path_d, edge_style_obj, None))
            
            # Add label if present
            if label:
                mx = (from_pt[0] + to_pt[0]) / 2
                my = (from_pt[1] + to_pt[1]) / 2 - 8
                label_style = rust.Style(fill='#666', opacity=1.0)
                scene.add_text(rust.Text(mx, my, str(label), 'system-ui', 12.0, 'normal', 'middle', label_style, None))
        
        # Render nodes
        for n in nodes:
            cx, cy = float(n['cx']), float(n['cy'])
            w, h = float(n['w']), float(n['h'])
            shape = n.get('shape', 'rect')
            node_style_dict = n.get('style', {})
            label = n.get('label')
            
            node_style = rust.Style(
                fill=node_style_dict.get('fill', '#3b82f6'),
                stroke=node_style_dict.get('stroke'),
                stroke_width=float(node_style_dict.get('stroke_width', 1.0)),
                opacity=float(node_style_dict.get('opacity', 1.0)),
                corner=float(node_style_dict.get('corner', 4.0))
            )
            
            match shape:
                case 'circle':
                    r = min(w, h) / 2
                    scene.add_circle(rust.Circle(cx, cy, r, node_style, None))
                case 'ellipse':
                    scene.add_ellipse(rust.Ellipse(cx, cy, w / 2, h / 2, node_style, None))
                case 'diamond':
                    # Create diamond as polygon
                    pts = [(cx, cy - h/2), (cx + w/2, cy), (cx, cy + h/2), (cx - w/2, cy)]
                    scene.add_polygon(rust.Polygon(pts, node_style, None))
                case _:  # rect
                    x, y = cx - w/2, cy - h/2
                    corner = float(node_style_dict.get('corner', 4.0))
                    scene.add_rect(rust.Rect(x, y, w, h, corner, node_style, None))
            
            # Add label
            if label:
                label_style = rust.Style(fill='#fff', opacity=1.0)
                scene.add_text(rust.Text(cx, cy, str(label), 'system-ui', 13.0, 'normal', 'middle', label_style, None))

    def _compute_edge_path(self, from_pt: tuple, to_pt: tuple, edge_style: str) -> str:
        """Compute SVG path data for an edge."""
        x1, y1 = from_pt
        x2, y2 = to_pt
        
        if edge_style == 'curved':
            mx = (x1 + x2) / 2
            my = (y1 + y2) / 2
            if abs(y2 - y1) > abs(x2 - x1):
                return f"M{x1},{y1} C{x1},{my} {x2},{my} {x2},{y2}"
            else:
                offset = max(abs(x2 - x1), abs(y2 - y1)) * 0.3
                return f"M{x1},{y1} C{mx},{y1 + offset} {mx},{y2 - offset} {x2},{y2}"
        elif edge_style == 'orthogonal':
            mx = (x1 + x2) / 2
            return f"M{x1},{y1} L{mx},{y1} L{mx},{y2} L{x2},{y2}"
        else:  # straight
            return f"M{x1},{y1} L{x2},{y2}"

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
            self.state.canvas = Canvas(size=c.get('size', 'medium'), fill=c['fill'])
        elif 'Shape' in ast:
            self._add_shape(ast['Shape'])
        elif 'Graph' in ast:
            self._add_graph(ast['Graph'])
        elif 'Variable' in ast:
            pass  # Variables handled during parsing

    def _add_shape(self, shape: dict):
        """Add shape to scene state."""
        self.state.shapes.append(self._shape_to_dict(shape))

    def _add_graph(self, graph: dict):
        """Add graph to scene state as a special shape."""
        self.state.shapes.append(self._graph_to_dict(graph))

    def _graph_to_dict(self, graph: dict) -> dict:
        """Convert Rust AST Graph to internal dict format."""
        layout = graph.get('layout', 'manual')
        direction = graph.get('direction', 'vertical')
        spacing = float(graph.get('spacing', 50.0))
        
        # Convert nodes
        nodes = []
        for n in graph.get('nodes', []):
            node_style = n.get('style', {})
            at = n.get('at')
            size = n.get('size', (80, 40))
            cx, cy = (float(at[0]), float(at[1])) if at else (0.0, 0.0)
            w, h = float(size[0]), float(size[1])
            nodes.append({
                'id': n['id'],
                'shape': n.get('shape', 'rect'),
                'cx': cx, 'cy': cy, 'w': w, 'h': h,
                'label': n.get('label'),
                'style': {
                    'fill': node_style.get('fill', '#3b82f6'),
                    'stroke': node_style.get('stroke'),
                    'stroke_width': float(node_style.get('stroke_width', 1.0)),
                    'opacity': float(node_style.get('opacity', 1.0)),
                    'corner': float(node_style.get('corner', 4.0)),
                }
            })
        
        # Apply layout if not manual
        if layout != 'manual' and nodes:
            nodes = self._apply_graph_layout(nodes, layout, direction, spacing)
        
        # Convert edges - resolve node references to coordinates
        node_map = {n['id']: n for n in nodes}
        edges = []
        for e in graph.get('edges', []):
            from_node = node_map.get(e['from'])
            to_node = node_map.get(e['to'])
            if from_node and to_node:
                from_pt, to_pt = self._compute_edge_anchors(from_node, to_node)
                edges.append({
                    'from_id': e['from'],
                    'to_id': e['to'],
                    'from_pt': from_pt,
                    'to_pt': to_pt,
                    'edge_style': e.get('style', 'straight'),
                    'arrow': e.get('arrow', 'forward'),
                    'label': e.get('label'),
                    'stroke': e.get('stroke', '#333'),
                    'stroke_width': float(e.get('stroke_width', 2.0)),
                })
        
        return {
            'kind': 'graph',
            'props': {'layout': layout, 'direction': direction, 'spacing': spacing},
            'style': {},
            'transform': {},
            'nodes': nodes,
            'edges': edges,
            'children': [],
        }

    def _apply_graph_layout(self, nodes: list, layout: str, direction: str, spacing: float) -> list:
        """Apply auto-layout to graph nodes."""
        is_vertical = direction != 'horizontal'
        
        if layout == 'hierarchical':
            pos = spacing
            for node in nodes:
                if is_vertical:
                    node['cy'] = pos + node['h'] / 2
                    node['cx'] = spacing * 2
                    pos += node['h'] + spacing
                else:
                    node['cx'] = pos + node['w'] / 2
                    node['cy'] = spacing * 2
                    pos += node['w'] + spacing
        elif layout == 'grid':
            cols = max(1, int(len(nodes) ** 0.5))
            for i, node in enumerate(nodes):
                row, col = i // cols, i % cols
                node['cx'] = spacing + col * (node['w'] + spacing) + node['w'] / 2
                node['cy'] = spacing + row * (node['h'] + spacing) + node['h'] / 2
        
        return nodes

    def _compute_edge_anchors(self, from_node: dict, to_node: dict) -> tuple:
        """Compute best anchor points for an edge between two nodes."""
        dx = to_node['cx'] - from_node['cx']
        dy = to_node['cy'] - from_node['cy']
        
        if abs(dy) > abs(dx):
            if dy > 0:
                from_pt = (from_node['cx'], from_node['cy'] + from_node['h'] / 2)
                to_pt = (to_node['cx'], to_node['cy'] - to_node['h'] / 2)
            else:
                from_pt = (from_node['cx'], from_node['cy'] - from_node['h'] / 2)
                to_pt = (to_node['cx'], to_node['cy'] + to_node['h'] / 2)
        else:
            if dx > 0:
                from_pt = (from_node['cx'] + from_node['w'] / 2, from_node['cy'])
                to_pt = (to_node['cx'] - to_node['w'] / 2, to_node['cy'])
            else:
                from_pt = (from_node['cx'] - from_node['w'] / 2, from_node['cy'])
                to_pt = (to_node['cx'] + to_node['w'] / 2, to_node['cy'])
        
        return from_pt, to_pt

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
