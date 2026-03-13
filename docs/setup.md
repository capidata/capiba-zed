# Setup — capiba-zed

## Pré-requisitos

- [Zed](https://zed.dev) instalado
- [Rust](https://rustup.rs) com target WASM:

  ```bash
  rustup target add wasm32-wasip1
  ```

- `ANTHROPIC_API_KEY` configurada nas settings do Zed

---

## Build

```bash
git clone https://github.com/capidata/capiba-zed
cd capiba-zed

# Ativar o pre-commit hook
pip install pre-commit && pre-commit install

# Instalar ferramentas de dev (cargo-llvm-cov e outras declaradas em Cargo.toml)
cargo xtask install-tools

# Build da extensão (WASM)
cargo build --release --target wasm32-wasip1

# Build do servidor MCP (nativo)
cargo build --release -p capiba-mcp
```

---

## Instalar no Zed

1. `Command Palette` (`Cmd+Shift+P`) → `zed: install dev extension`
2. Selecionar a pasta `capiba-zed/`
3. Reiniciar o Zed

---

## Configurar o Zed

### ANTHROPIC_API_KEY

Em `~/.config/zed/settings.json` (Linux/macOS) ou `%APPDATA%\Zed\settings.json` (Windows):

```json
{
  "env": {
    "ANTHROPIC_API_KEY": "seu-api-key-aqui"
  }
}
```

Ou via variável de ambiente do sistema:

```bash
export ANTHROPIC_API_KEY="seu-api-key-aqui"
zed
```

### capiba-mcp

Em `~/.config/zed/settings.json`, adicione com `"source": "custom"` e o caminho completo do binário:

```json
{
  "context_servers": {
    "capiba-mcp": {
      "source": "custom",
      "command": "/usr/local/bin/capiba-mcp",
      "args": [],
      "env": {}
    }
  }
}
```

Se instalou o binário em outro caminho, ajuste `"command"` para o caminho completo.

---

## Instalar o `capiba-mcp` via release

Baixe o binário correspondente ao seu sistema em [Releases](https://github.com/capidata/capiba-zed/releases).

**macOS (Apple Silicon — M1/M2/M3):**

```bash
curl -L https://github.com/capidata/capiba-zed/releases/download/v0.1.0/capiba-mcp-macos-aarch64 -o capiba-mcp
chmod +x capiba-mcp
sudo mv capiba-mcp /usr/local/bin/
```

**Linux (x86_64):**

```bash
curl -L https://github.com/capidata/capiba-zed/releases/download/v0.1.0/capiba-mcp-linux-x86_64 -o capiba-mcp
chmod +x capiba-mcp
sudo mv capiba-mcp /usr/local/bin/
```

**Linux (ARM):**

```bash
curl -L https://github.com/capidata/capiba-zed/releases/download/v0.1.0/capiba-mcp-linux-aarch64 -o capiba-mcp
chmod +x capiba-mcp
sudo mv capiba-mcp /usr/local/bin/
```

**Windows (x86_64):**

```powershell
Invoke-WebRequest -Uri "https://github.com/capidata/capiba-zed/releases/download/v0.1.0/capiba-mcp-windows-x86_64.exe" `
  -OutFile capiba-mcp.exe
Move-Item capiba-mcp.exe "C:\Program Files\capiba\capiba-mcp.exe"
```

**Windows (ARM):**

```powershell
Invoke-WebRequest -Uri "https://github.com/capidata/capiba-zed/releases/download/v0.1.0/capiba-mcp-windows-aarch64.exe" `
  -OutFile capiba-mcp.exe
Move-Item capiba-mcp.exe "C:\Program Files\capiba\capiba-mcp.exe"
```

Verifique a instalação:

```bash
capiba-mcp --version
# capiba-mcp 0.1.0
```

> `capiba-mcp` é um servidor MCP via stdio. É usado automaticamente pela extensão Zed — não execute diretamente.

---

## Criar um novo release

Toda tag `v*` dispara o workflow de CD que builda a extensão WASM e compila `capiba-mcp` para 6 plataformas.

```bash
git tag v0.2.0
git push origin v0.2.0
```
