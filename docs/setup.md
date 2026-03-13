# Setup — capiba-zed

## Pré-requisitos

- [Zed](https://zed.dev) instalado
- [Rust](https://rustup.rs) com target WASM:

  ```bash
  rustup target add wasm32-wasip1
  ```

- `wasm-tools` (necessário para converter WASM module em component):

  ```bash
  cargo install wasm-tools
  ```

- `ANTHROPIC_API_KEY` configurada nas settings do Zed

---

## Build e instalação

```bash
git clone https://github.com/capidata/capiba-zed
cd capiba-zed

# Ativar pre-commit hooks
pip install pre-commit && pre-commit install

# Build e instalar servidor MCP (binário + wrapper)
./scripts/build.sh
```

O `build.sh` instala o binário em `~/.local/bin/capiba-mcp` e o wrapper em
`~/.local/bin/capiba-mcp-run` e `/usr/local/bin/capiba-mcp-run`.

---

## Instalar a extensão no Zed (slash commands)

O Zed não aceita WASM compilado externamente. Use o command palette:

1. Abra o Zed com o diretório `capiba-zed/` no workspace
2. `Command Palette` (`Cmd+Shift+P`) → `zed: install dev extension`

> **Nota:** A extensão dev funciona apenas quando `capiba-zed` está no workspace.
> Para usar slash commands em outros repositórios, abra ambos no mesmo workspace:
> `File > Add Folder to Project → capiba-zed`

---

## Configurar o MCP server globalmente

Para que o `capiba-mcp` funcione em **qualquer repositório**, adicione ao
`~/.config/zed/settings.json`:

```json
{
  "context_servers": {
    "capiba-mcp": {
      "command": {
        "path": "/usr/local/bin/capiba-mcp-run",
        "args": []
      }
    }
  }
}
```

Reinicie o Zed após essa configuração.

> O comando deve apontar para o **wrapper** `capiba-mcp-run`, não para o binário
> `capiba-mcp` diretamente. Ver [troubleshooting](troubleshooting.md) para detalhes.

---

## Configurar o `ANTHROPIC_API_KEY`

Em `~/.config/zed/settings.json`:

```json
{
  "env": {
    "ANTHROPIC_API_KEY": "seu-api-key-aqui"
  }
}
```

Ou via variável de ambiente do sistema antes de abrir o Zed:

```bash
export ANTHROPIC_API_KEY="seu-api-key-aqui"
zed
```

---

## Usando o capiba-mcp no Claude Code

O `capiba-mcp` funciona em qualquer cliente MCP, incluindo o Claude Code:

```bash
# Adicionar globalmente (todos os projetos)
claude mcp add --scope user capiba-mcp /usr/local/bin/capiba-mcp-run

# Verificar conexão
claude mcp list
# capiba-mcp: /usr/local/bin/capiba-mcp-run - ✓ Connected
```

---

## Criar um novo release

Toda tag `v*` dispara o workflow de CD:

```bash
git tag v0.2.0
git push origin v0.2.0
```
