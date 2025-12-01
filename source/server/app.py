"""FastAPI application with WebSocket support."""

import logging
from pathlib import Path
from fastapi import FastAPI, Request, HTTPException
from fastapi.staticfiles import StaticFiles
from fastapi.responses import FileResponse, JSONResponse
from .ws import router as ws_router
from lang import ErrorCode, ErrorInfo, Severity

logger = logging.getLogger(__name__)

# Resolve static dir - use playground build from npm package
STATIC_DIR = (Path(__file__).parent.parent.parent / "distribution" / "npm" / "playground" / "dist").resolve()
if not STATIC_DIR.exists():
    # Fallback for installed mode
    import importlib.resources as pkg_resources
    STATIC_DIR = Path(str(pkg_resources.files("iconoglott"))) / "static"


def _error_response(code: ErrorCode, msg: str, status: int = 500) -> JSONResponse:
    """Build a standardized error response."""
    return JSONResponse(
        status_code=status,
        content={
            "error": True,
            "message": msg,
            "errors": [ErrorInfo(code, msg, severity=Severity.ERROR).to_dict()]
        }
    )


def create_app() -> FastAPI:
    """Create and configure the FastAPI application."""
    app = FastAPI(title="Iconoglott", version="0.1.0")
    app.include_router(ws_router)
    
    @app.exception_handler(HTTPException)
    async def http_exception_handler(request: Request, exc: HTTPException):
        return _error_response(
            ErrorCode.WS_CONNECTION_ERROR, exc.detail, exc.status_code
        )
    
    @app.exception_handler(Exception)
    async def general_exception_handler(request: Request, exc: Exception):
        logger.exception("Unhandled exception")
        return _error_response(
            ErrorCode.EVAL_INVALID_SHAPE, "Internal server error", 500
        )
    
    @app.get("/health")
    async def health():
        return {"status": "ok", "version": "0.1.0"}

    # Mount static files at root (must be after other routes)
    if STATIC_DIR.exists():
        app.mount("/", StaticFiles(directory=STATIC_DIR, html=True), name="static")

    return app


app = create_app()

if __name__ == "__main__":
    import uvicorn
    uvicorn.run("server.app:app", host="0.0.0.0", port=8765, reload=True)

