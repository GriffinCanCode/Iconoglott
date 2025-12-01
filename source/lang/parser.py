"""Parser for the visual DSL with error reporting."""

from typing import Iterator
from .types import Token, TokenType, Node, Canvas, Shape, Style, Transform, ParseError
from .errors import ErrorCode, ErrorInfo, ErrorList


class Parser:
    """Parse token stream into AST with error collection."""

    SHAPES = {'rect', 'circle', 'ellipse', 'line', 'path', 'polygon', 'text', 'image'}
    STYLE_PROPS = {'fill', 'stroke', 'opacity', 'corner', 'shadow', 'gradient', 'blur'}
    TEXT_PROPS = {'font', 'bold', 'italic', 'center', 'middle', 'end'}
    TRANSFORM_PROPS = {'translate', 'rotate', 'scale', 'origin'}

    def __init__(self, tokens: Iterator[Token]):
        self.tokens = list(tokens)
        self.pos = 0
        self.variables: dict[str, Token] = {}
        self.errors: list[ParseError] = []
        self.error_infos: ErrorList = []

    @property
    def current(self) -> Token | None:
        return self.tokens[self.pos] if self.pos < len(self.tokens) else None

    @property
    def peek_next(self) -> Token | None:
        return self.tokens[self.pos + 1] if self.pos + 1 < len(self.tokens) else None

    def advance(self) -> Token | None:
        tok = self.current
        self.pos += 1
        return tok

    def skip_newlines(self):
        while self.current and self.current.type == TokenType.NEWLINE:
            self.advance()

    def resolve(self, tok: Token):
        """Resolve variable references."""
        if tok.type == TokenType.VAR:
            if tok.value in self.variables:
                return self.variables[tok.value].value
            self._error(ErrorCode.PARSE_UNDEFINED_VAR, f"Undefined variable: {tok.value}", tok)
            return tok.value
        return tok.value

    def error(self, msg: str, tok: Token | None = None):
        """Record a parse error. Legacy method for backward compatibility."""
        self._error(ErrorCode.PARSE_UNEXPECTED_TOKEN, msg, tok)

    def _error(self, code: ErrorCode, msg: str, tok: Token | None = None):
        """Record a typed parse error."""
        t = tok or self.current
        line, col = (t.line, t.col) if t else (0, 0)
        self.errors.append(ParseError(msg, line, col, code))
        self.error_infos.append(ErrorInfo(code, msg, line, col))

    def expect(self, ttype: TokenType, msg: str, code: ErrorCode = ErrorCode.PARSE_EXPECTED_VALUE) -> Token | None:
        """Expect a token type, record error if not found."""
        if self.current and self.current.type == ttype:
            return self.advance()
        self._error(code, msg)
        return None

    def parse(self) -> Node:
        """Parse into a scene AST."""
        root = Node("scene")
        self.skip_newlines()

        while self.current and self.current.type != TokenType.EOF:
            if node := self._parse_statement():
                root.children.append(node)
            self.skip_newlines()

        return root

    def _parse_statement(self) -> Node | None:
        """Parse a top-level statement."""
        if not self.current:
            return None

        # Variable assignment: $name = value
        if self.current.type == TokenType.VAR:
            return self._parse_variable()

        if self.current.type != TokenType.IDENT:
            self._error(ErrorCode.PARSE_UNEXPECTED_TOKEN, f"Expected command, got {self.current.type.name}")
            self.advance()
            return None

        cmd = self.current.value
        self.advance()

        match cmd:
            case "canvas":
                return self._parse_canvas()
            case "group":
                return self._parse_group()
            case "stack" | "row":
                return self._parse_layout(cmd)
            case _ if cmd in self.SHAPES:
                return self._parse_shape(cmd)
            case _:
                self._error(ErrorCode.PARSE_UNKNOWN_COMMAND, f"Unknown command: {cmd}")
                return None

    def _parse_variable(self) -> Node:
        """Parse variable assignment."""
        name_tok = self.advance()
        name = name_tok.value
        if self.current and self.current.type == TokenType.EQUALS:
            self.advance()
            if self.current:
                self.variables[name] = self.current
                self.advance()
            else:
                self._error(ErrorCode.PARSE_EMPTY_VALUE, "Expected value after '='", name_tok)
        else:
            self._error(ErrorCode.PARSE_MISSING_EQUALS, "Expected '=' in variable assignment", name_tok)
        return Node("variable", {"name": name, "value": self.variables.get(name)})

    def _parse_canvas(self) -> Node:
        """Parse canvas definition."""
        canvas = Canvas()
        
        if self.current and self.current.type == TokenType.PAIR:
            w, h = self.advance().value
            canvas.width, canvas.height = int(w), int(h)
        
        while self.current and self.current.type == TokenType.IDENT:
            prop = self.advance().value
            if prop == "fill" and self.current:
                canvas.fill = self.resolve(self.advance())

        return Node("canvas", canvas)

    def _parse_group(self) -> Node:
        """Parse group with optional name."""
        name = None
        if self.current and self.current.type == TokenType.STRING:
            name = self.advance().value

        shape = Shape(kind="group", props={"name": name})
        self.skip_newlines()
        
        if self.current and self.current.type == TokenType.INDENT:
            self.advance()
            shape = self._parse_block(shape)

        return Node("shape", shape)

    def _parse_layout(self, kind: str) -> Node:
        """Parse stack/row layout."""
        props = {"direction": "vertical" if kind == "stack" else "horizontal", "gap": 0}
        
        while self.current and self.current.type == TokenType.IDENT:
            prop = self.advance().value
            match prop:
                case "vertical" | "horizontal":
                    props["direction"] = prop
                case "gap" if self.current and self.current.type == TokenType.NUMBER:
                    props["gap"] = self.advance().value
                case "at" if self.current and self.current.type == TokenType.PAIR:
                    props["at"] = self.advance().value

        shape = Shape(kind="layout", props=props)
        self.skip_newlines()
        
        if self.current and self.current.type == TokenType.INDENT:
            self.advance()
            shape = self._parse_block(shape)

        return Node("shape", shape)

    def _parse_shape(self, kind: str) -> Node:
        """Parse shape with properties and nested style."""
        props = {}
        start_tok = self.tokens[self.pos - 1] if self.pos > 0 else None
        
        while self.current and self.current.type not in (TokenType.NEWLINE, TokenType.EOF):
            match self.current.type:
                case TokenType.PAIR:
                    val = self.advance().value
                    if "at" not in props:
                        props["at"] = val
                    elif "size" not in props:
                        props["size"] = val
                case TokenType.NUMBER:
                    val = self.advance().value
                    if kind == "circle" and "radius" not in props:
                        props["radius"] = val
                    elif "width" not in props:
                        props["width"] = val
                case TokenType.STRING:
                    props["content"] = self.advance().value
                case TokenType.LBRACKET:
                    # Parse polygon points: [x,y x,y x,y]
                    if kind == "polygon":
                        props["points"] = self._parse_points()
                    else:
                        self.advance()
                case TokenType.IDENT:
                    key = self.advance().value
                    match key:
                        case "at" if self.current and self.current.type == TokenType.PAIR:
                            props["at"] = self.advance().value
                        case "size" if self.current and self.current.type == TokenType.PAIR:
                            props["size"] = self.advance().value
                        case "radius" if self.current and self.current.type == TokenType.PAIR:
                            # ellipse: radius x,y
                            props["radius"] = self.advance().value
                        case "radius" if self.current and self.current.type == TokenType.NUMBER:
                            props["radius"] = self.advance().value
                        case "from" if self.current and self.current.type == TokenType.PAIR:
                            props["from"] = self.advance().value
                        case "to" if self.current and self.current.type == TokenType.PAIR:
                            props["to"] = self.advance().value
                        case "d" if self.current and self.current.type == TokenType.STRING:
                            # path d "M 0 0 L 100 100"
                            props["d"] = self.advance().value
                        case "points" if self.current and self.current.type == TokenType.LBRACKET:
                            props["points"] = self._parse_points()
                        case "href" if self.current and self.current.type == TokenType.STRING:
                            props["href"] = self.advance().value
                case TokenType.COLOR | TokenType.VAR:
                    if "fill" not in props:
                        props["fill"] = self.resolve(self.advance())
                case _:
                    self.advance()

        shape = Shape(kind=kind, props=props)
        self.skip_newlines()

        if self.current and self.current.type == TokenType.INDENT:
            self.advance()
            shape = self._parse_block(shape)

        return Node("shape", shape)

    def _parse_points(self) -> list[tuple[float, float]]:
        """Parse polygon points: [x,y x,y x,y]"""
        points = []
        self.advance()  # consume [
        while self.current and self.current.type != TokenType.RBRACKET:
            if self.current.type == TokenType.PAIR:
                points.append(self.advance().value)
            else:
                self.advance()
        if self.current and self.current.type == TokenType.RBRACKET:
            self.advance()
        else:
            self._error(ErrorCode.PARSE_MISSING_BRACKET, "Expected ']' to close points list")
        return points

    def _parse_block(self, shape: Shape) -> Shape:
        """Parse indented block of style/transform/children."""
        while self.current and self.current.type != TokenType.DEDENT:
            self.skip_newlines()
            if not self.current or self.current.type == TokenType.DEDENT:
                break

            if self.current.type == TokenType.IDENT:
                prop = self.current.value
                
                if prop in self.SHAPES:
                    if child := self._parse_statement():
                        shape.children.append(child.value)
                elif prop in self.STYLE_PROPS:
                    self._parse_style_prop(shape.style)
                elif prop in self.TEXT_PROPS:
                    self._parse_text_prop(shape.style)
                elif prop in self.TRANSFORM_PROPS:
                    self._parse_transform_prop(shape.transform)
                elif prop == "width" and self.peek_next and self.peek_next.type == TokenType.NUMBER:
                    self.advance()
                    shape.style.stroke_width = self.advance().value
                elif prop == "d" and self.peek_next and self.peek_next.type == TokenType.STRING:
                    # path d command in block
                    self.advance()
                    shape.props["d"] = self.advance().value
                elif prop == "points" and self.peek_next and self.peek_next.type == TokenType.LBRACKET:
                    self.advance()
                    shape.props["points"] = self._parse_points()
                else:
                    self.advance()
            else:
                self.advance()

        if self.current and self.current.type == TokenType.DEDENT:
            self.advance()

        return shape

    def _parse_style_prop(self, style: Style):
        """Parse a style property."""
        prop = self.advance().value
        
        match prop:
            case "fill":
                if self.current and self.current.type in (TokenType.COLOR, TokenType.VAR, TokenType.IDENT):
                    style.fill = self.resolve(self.advance())
                else:
                    self._error(ErrorCode.PARSE_EXPECTED_COLOR, "Expected color value after 'fill'")
            case "stroke":
                if self.current and self.current.type in (TokenType.COLOR, TokenType.VAR):
                    style.stroke = self.resolve(self.advance())
                else:
                    self._error(ErrorCode.PARSE_EXPECTED_COLOR, "Expected color value after 'stroke'")
                if self.current and self.current.type == TokenType.NUMBER:
                    style.stroke_width = self.advance().value
                if self.current and self.current.type == TokenType.IDENT and self.current.value == "width":
                    self.advance()
                    if self.current and self.current.type == TokenType.NUMBER:
                        style.stroke_width = self.advance().value
            case "opacity":
                if self.current and self.current.type == TokenType.NUMBER:
                    style.opacity = self.advance().value
                else:
                    self._error(ErrorCode.PARSE_EXPECTED_NUMBER, "Expected number after 'opacity'")
            case "corner":
                if self.current and self.current.type == TokenType.NUMBER:
                    style.corner = self.advance().value
                else:
                    self._error(ErrorCode.PARSE_EXPECTED_NUMBER, "Expected number after 'corner'")
            case "shadow":
                style.shadow = self._parse_shadow()
            case "gradient":
                style.gradient = self._parse_gradient()

    def _parse_text_prop(self, style: Style):
        """Parse text-specific property."""
        prop = self.advance().value
        
        match prop:
            case "font":
                if self.current and self.current.type == TokenType.STRING:
                    style.font = self.advance().value
                if self.current and self.current.type == TokenType.NUMBER:
                    style.font_size = self.advance().value
            case "bold":
                style.font_weight = "bold"
            case "italic":
                style.font_weight = "italic"
            case "center":
                style.text_anchor = "middle"
            case "end":
                style.text_anchor = "end"

    def _parse_transform_prop(self, transform: Transform):
        """Parse transform property."""
        prop = self.advance().value
        
        match prop:
            case "translate":
                if self.current and self.current.type == TokenType.PAIR:
                    transform.translate = self.advance().value
                else:
                    self._error(ErrorCode.PARSE_EXPECTED_PAIR, "Expected pair (x,y) after 'translate'")
            case "rotate":
                if self.current and self.current.type == TokenType.NUMBER:
                    transform.rotate = self.advance().value
                else:
                    self._error(ErrorCode.PARSE_EXPECTED_NUMBER, "Expected number after 'rotate'")
            case "scale":
                if self.current and self.current.type == TokenType.PAIR:
                    transform.scale = self.advance().value
                elif self.current and self.current.type == TokenType.NUMBER:
                    s = self.advance().value
                    transform.scale = (s, s)
                else:
                    self._error(ErrorCode.PARSE_EXPECTED_VALUE, "Expected number or pair after 'scale'")
            case "origin":
                if self.current and self.current.type == TokenType.PAIR:
                    transform.origin = self.advance().value
                else:
                    self._error(ErrorCode.PARSE_EXPECTED_PAIR, "Expected pair (x,y) after 'origin'")

    def _parse_shadow(self) -> dict:
        """Parse shadow: offset blur color."""
        shadow = {"x": 0, "y": 4, "blur": 8, "color": "#0004"}
        
        if self.current and self.current.type == TokenType.PAIR:
            shadow["x"], shadow["y"] = self.advance().value
        if self.current and self.current.type == TokenType.NUMBER:
            shadow["blur"] = self.advance().value
        if self.current and self.current.type == TokenType.COLOR:
            shadow["color"] = self.advance().value
            
        return shadow

    def _parse_gradient(self) -> dict:
        """Parse gradient: type from to."""
        gradient = {"type": "linear", "from": "#fff", "to": "#000", "angle": 90}
        
        while self.current and self.current.type in (TokenType.IDENT, TokenType.COLOR, TokenType.NUMBER):
            match self.current.type:
                case TokenType.IDENT:
                    val = self.advance().value
                    if val in ("linear", "radial"):
                        gradient["type"] = val
                    elif val == "from" and self.current and self.current.type == TokenType.COLOR:
                        gradient["from"] = self.advance().value
                    elif val == "to" and self.current and self.current.type == TokenType.COLOR:
                        gradient["to"] = self.advance().value
                case TokenType.COLOR:
                    if gradient["from"] == "#fff":
                        gradient["from"] = self.advance().value
                    else:
                        gradient["to"] = self.advance().value
                case TokenType.NUMBER:
                    gradient["angle"] = self.advance().value
                    
        return gradient
