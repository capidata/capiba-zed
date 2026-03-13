# Troubleshooting — capiba-zed

## Como verificar os logs do Zed

```bash
grep -i "capiba\|context.server\|timeout" ~/Library/Logs/Zed/Zed.log | tail -20
```

---

## Toggle do MCP desativado no painel do agente

**Sintoma:** O indicador do `capiba-mcp` no Agent Panel fica cinza e não ativa.

**Causa:** A extensão precisa da `zed_extension_api >= 0.5` e da implementação
do método `context_server_command()`. Versões anteriores (0.4) não expõem esse
método, e o Zed não consegue iniciar o servidor via extensão.

**Solução:**

Verifique o `Cargo.toml` da extensão:

```toml
zed_extension_api = "0.5"
```

Confirme que `src/lib.rs` implementa `context_server_command()` no `impl zed::Extension`:

```rust
fn context_server_command(
    &mut self,
    context_server_id: &ContextServerId,
    _project: &Project,
) -> Result<Command> {
    match context_server_id.as_ref() {
        "capiba-mcp" => Ok(Command {
            command: "capiba-mcp".to_string(),
            args: vec![],
            env: vec![],
        }),
        id => Err(format!("servidor MCP desconhecido: {id}")),
    }
}
```

Depois rebuild e reinstale:

```bash
cargo build --release --target wasm32-wasip1
# Command Palette → "zed: install dev extension"
```

---

## Context server request timeout (60s)

**Sintoma:** O log do Zed mostra:

```text
cancelled csp request task for "initialize" id 0 which took over 60s
capiba-mcp context server failed to start: Context server request timeout
```

**Diagnóstico:** Verifique se o servidor responde ao protocolo MCP:

```bash
echo '{"jsonrpc":"2.0","id":0,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"0.1"}}}' \
  | capiba-mcp
# Deve retornar JSON com serverInfo em < 2 segundos
```

**Causa mais comum:** O binário em `/usr/local/bin/capiba-mcp` está desatualizado.

**Solução:** Recompile e reinstale:

```bash
cargo build --release -p capiba-mcp
sudo cp target/release/capiba-mcp /usr/local/bin/capiba-mcp
```

Reinicie o Zed completamente.

---

## Formato incorreto em `context_servers` no settings.json

**Sintoma:** O log mostra:

```text
data did not match any variant of untagged enum ContextServerSettingsContent
```

**Causa:** A entrada em `context_servers` usa formato inválido
(ex: `"remote": false`, ou `"enabled": true` sem `"source"`).

**Solução:** Use `"source": "custom"` com caminho absoluto:

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
