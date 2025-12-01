# PyPI Build Makefile
# Builds the iconoglott package with Rust core bundled

PYTHON_SRC := ../../source/lang
PYTHON_DEST := python/iconoglott

.PHONY: all clean sync build publish test

all: build

# Sync Python source from source/lang to python/iconoglott
sync:
	@echo "Syncing Python source..."
	@rm -rf $(PYTHON_DEST)
	@mkdir -p $(PYTHON_DEST)
	@cp -r $(PYTHON_SRC)/*.py $(PYTHON_DEST)/
	@cp -r $(PYTHON_SRC)/tools $(PYTHON_DEST)/
	@# Remove __pycache__ directories
	@find $(PYTHON_DEST) -type d -name __pycache__ -exec rm -rf {} + 2>/dev/null || true
	@echo "Synced to $(PYTHON_DEST)"

# Build wheels
build: sync
	@echo "Building wheels with maturin..."
	maturin build --release

# Build and install locally for testing
develop: sync
	maturin develop --release

# Publish to PyPI
publish: sync
	maturin publish

# Publish to TestPyPI
publish-test: sync
	maturin publish --repository testpypi

# Run tests
test: develop
	pytest ../../source/tests -v

# Clean build artifacts
clean:
	@rm -rf $(PYTHON_DEST)
	@rm -rf target/
	@rm -rf *.egg-info/
	@rm -rf dist/
	@echo "Cleaned build artifacts"

