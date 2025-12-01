"""FastAPI application with WebSocket support."""

from pathlib import Path
from fastapi import FastAPI
from fastapi.staticfiles import StaticFiles
from fastapi.responses import FileResponse
from .ws import router as ws_router

# Resolve static dir relative to this file (works in dev and installed mode)
STATIC_DIR = (Path(__file__).parent.parent / "static").resolve()
if not STATIC_DIR.exists():
    # Fallback for installed package
    import importlib.resources as pkg_resources
    STATIC_DIR = Path(str(pkg_resources.files("iconoglott"))) / "static"


def create_app() -> FastAPI:
    """Create and configure the FastAPI application."""
    app = FastAPI(title="Iconoglott", version="0.1.0")
    app.include_router(ws_router)
    
    if STATIC_DIR.exists():
        app.mount("/static", StaticFiles(directory=STATIC_DIR), name="static")

    @app.get("/")
    async def index():
        return FileResponse(STATIC_DIR / "index.html")

    return app


app = create_app()

if __name__ == "__main__":
    import uvicorn
    uvicorn.run("server.app:app", host="0.0.0.0", port=8765, reload=True)

