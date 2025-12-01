"""LangChain tool integration for iconoglott.

Requires: pip install iconoglott[langchain]
"""

from typing import Optional, Type
from .schemas import DSL_REFERENCE

try:
    from langchain_core.tools import BaseTool
    from langchain_core.callbacks import CallbackManagerForToolRun
    from pydantic import BaseModel, Field
    LANGCHAIN_AVAILABLE = True
except ImportError:
    LANGCHAIN_AVAILABLE = False
    BaseTool = object
    BaseModel = object
    Field = lambda *a, **kw: None
    CallbackManagerForToolRun = None


class IconoglottInput(BaseModel if LANGCHAIN_AVAILABLE else object):
    """Input schema for iconoglott tool."""
    code: str = Field(description="The iconoglott DSL code to render")


class IconoglottTool(BaseTool if LANGCHAIN_AVAILABLE else object):
    """LangChain tool for rendering iconoglott DSL to SVG.

    Example:
        >>> from langchain.agents import initialize_agent, AgentType
        >>> from langchain_openai import ChatOpenAI
        >>> from iconoglott.tools import IconoglottTool
        >>>
        >>> llm = ChatOpenAI(model="gpt-4")
        >>> tools = [IconoglottTool()]
        >>> agent = initialize_agent(tools, llm, agent=AgentType.OPENAI_FUNCTIONS)
        >>>
        >>> agent.run("Create a simple logo with a red circle and white text")
    """

    name: str = "iconoglott"
    description: str = f"""Render vector graphics using the iconoglott visual DSL.
Use this for diagrams, UI mockups, visualizations, icons, and SVG graphics.

{DSL_REFERENCE}"""
    args_schema: Type[BaseModel] = IconoglottInput
    return_direct: bool = False

    def _run(
        self,
        code: str,
        run_manager: Optional[CallbackManagerForToolRun] = None,
    ) -> str:
        """Render iconoglott DSL code to SVG."""
        from .. import render
        try:
            return render(code)
        except Exception as e:
            return f"Error rendering iconoglott: {e}"

    async def _arun(
        self,
        code: str,
        run_manager: Optional[CallbackManagerForToolRun] = None,
    ) -> str:
        """Async version - just calls sync since rendering is fast."""
        return self._run(code, run_manager)


def create_tool() -> "IconoglottTool":
    """Create a LangChain tool for iconoglott rendering.

    Returns:
        IconoglottTool: Ready-to-use LangChain tool

    Raises:
        ImportError: If langchain-core is not installed

    Example:
        >>> from iconoglott.tools import create_tool
        >>> tool = create_tool()
        >>>
        >>> # Use with LangChain agent
        >>> from langchain.agents import create_openai_functions_agent
        >>> agent = create_openai_functions_agent(llm, [tool], prompt)
    """
    if not LANGCHAIN_AVAILABLE:
        raise ImportError(
            "LangChain integration requires langchain-core. "
            "Install with: pip install iconoglott[langchain]"
        )
    return IconoglottTool()

