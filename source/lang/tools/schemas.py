"""Function calling schemas for AI integrations."""

DSL_REFERENCE = """Iconoglott DSL Syntax Reference:

CANVAS:
  canvas giant fill #1a1a2e

VARIABLES:
  $accent = #e94560
  $size = 40

SHAPES:
  rect at 100,100 size 200x150
    fill #e94560
    stroke #fff 2
    corner 12
    shadow 0,8 20 #0008

  circle at 400,300 radius 80
    fill #0f3460
    gradient radial #4ecdc4 #0f3460

  ellipse at 300,200 size 60x40
    fill #fff

  text "Hello World" at 50,50
    font "Fira Code" 24
    fill #fff
    bold
    center

  line from 0,0 to 100,100
    stroke #fff 2

  path "M 10 80 Q 95 10 180 80"
    stroke #e94560 3

  polygon points [0,0 50,0 25,50]
    fill #e94560

STYLE PROPERTIES:
  fill #color or fill $var
  stroke #color width
  corner 12 (border radius)
  opacity 0.5
  gradient linear/radial #from #to
  shadow x,y blur #color

TEXT PROPERTIES:
  font "Name" size
  bold / italic
  center / end

TRANSFORMS:
  rotate 45
  scale 1.5,1.5
  translate 10,20
  origin 125,125

GROUPING:
  group "name"
    rect at 0,0 size 100x50
    text "Button" at 50,30

LAYOUT:
  stack vertical gap 20 at 50,100
    circle radius 10 fill #f00
    circle radius 10 fill #0f0

  row horizontal gap 10 at 50,200
    rect size 30x30 fill #f00
    rect size 30x30 fill #0f0
"""


def get_openai_schema() -> dict:
    """Get OpenAI function calling schema for iconoglott.

    Returns:
        dict: OpenAI-compatible function schema

    Example:
        >>> schema = get_openai_schema()
        >>> response = openai.chat.completions.create(
        ...     model="gpt-4",
        ...     messages=[...],
        ...     tools=[{"type": "function", "function": schema}]
        ... )
    """
    return {
        "name": "render_iconoglott",
        "description": f"""Render vector graphics using the iconoglott visual DSL.

Use this tool to create diagrams, UI mockups, data visualizations, icons, and other SVG graphics.

{DSL_REFERENCE}""",
        "parameters": {
            "type": "object",
            "properties": {
                "code": {
                    "type": "string",
                    "description": "The iconoglott DSL code to render. Must start with a canvas definition."
                }
            },
            "required": ["code"]
        }
    }

