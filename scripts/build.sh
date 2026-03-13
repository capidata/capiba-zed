#!/bin/bash
set -e

echo "🔨 Building capiba-mcp..."
cargo build --release -p capiba-mcp

BINARY="target/release/capiba-mcp"
DEST="$HOME/.local/bin/capiba-mcp"

if [ -f "$BINARY" ]; then
    mkdir -p "$(dirname "$DEST")"
    cp -f "$BINARY" "$DEST" || true
    chmod +x "$DEST"

    # Wrapper script para garantir flush correto do stdout via pipeline
    WRAPPER="$(dirname "$DEST")/capiba-mcp-run"
    cat > "$WRAPPER" << SCRIPT
#!/bin/bash
tee /tmp/capiba-mcp-in.log | "$DEST" 2>/tmp/capiba-mcp-err.log | tee /tmp/capiba-mcp-out.log
SCRIPT
    chmod +x "$WRAPPER"

    # Instalar wrapper em /usr/local/bin como fallback (usado pelo WASM quando HOME está vazio)
    if sudo cp "$WRAPPER" /usr/local/bin/capiba-mcp-run 2>/dev/null; then
        echo "✅ Wrapper (global): /usr/local/bin/capiba-mcp-run"
    fi

    VERSION=$(grep '^version' mcp-server/Cargo.toml | head -1 | cut -d'"' -f2)
    echo "✅ Installed: $DEST (v$VERSION)"
    echo "✅ Wrapper:   $WRAPPER"
else
    echo "❌ Error: binary not found at $BINARY"
    exit 1
fi
