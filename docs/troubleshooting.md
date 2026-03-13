# Troubleshooting — capiba-zed

## Como verificar os logs do Zed

```bash
grep -i "capiba\|context.server\|timeout\|extension" ~/Library/Logs/Zed/Zed.log | tail -30
```

---

## Context server request timeout (60s)

**Sintoma:** O log do Zed mostra:

```text
cancelled csp request task for "initialize" id 0 which took over 60s
capiba-mcp context server failed to start: Context server request timeout
```

**Causa:** O rmcp 0.16 com tokio stdio bufferiza o stdout sem flush automático.
O Zed aguarda a resposta do `initialize` e dá timeout porque ela fica presa no buffer.

**Solução:** O `capiba-mcp` deve ser executado via o wrapper `capiba-mcp-run`, que
usa um pipeline com `tee` para forçar o flush. Verifique se o wrapper está instalado:

```bash
cat /usr/local/bin/capiba-mcp-run
# deve conter: tee ... | capiba-mcp | tee ...
```

Se não existir:

```bash
./scripts/build.sh
```

E confirme que o `settings.json` aponta para o wrapper, não para o binário:

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

---

## Slash commands não aparecem em outros repositórios

**Sintoma:** Os comandos `/capiba-*` aparecem no repositório `capiba-zed` mas não
em outros projetos.

**Causa:** A extensão é instalada como "dev extension" e só fica ativa quando
`capiba-zed` está no workspace.

**Solução 1 (recomendada):** Adicionar `capiba-zed` ao workspace do projeto:

```text
File > Add Folder to Project → selecione capiba-zed/
```

**Solução 2:** Publicar a extensão no Zed Extension Registry (v1.0).

> `./scripts/install.sh` **não resolve** esse problema — o Zed não aceita WASM
> compilado externamente para carregar slash commands.

---

## Extensão não aparece na lista após install.sh

**Causa:** O Zed ignora extensões copiadas manualmente para
`~/.config/zed/extensions/`. Ele só carrega extensões da pasta
`~/Library/Application Support/Zed/extensions/installed/` (macOS), e exige que
o WASM seja gerado internamente (via `zed: install dev extension`).

**Solução:** Usar `zed: install dev extension` no command palette com `capiba-zed`
aberto no workspace.

---

## MCP funciona no capiba-zed mas não no capiba-core

**Causa:** O `context_server_command()` da extensão WASM é chamado apenas quando a
extensão está carregada. Se o MCP está configurado apenas via extensão (não via
`settings.json`), ele não inicia em outros repositórios.

**Solução:** Configurar o MCP globalmente no `settings.json` conforme descrito em
[setup.md](setup.md). Isso desacopla o MCP server da extensão.

---

## `capiba-mcp --version` retorna `zsh: killed` (exit 137)

**Causa:** O binário em `~/.cargo/bin/capiba-mcp` ou `/usr/local/bin/capiba-mcp`
é uma versão antiga corrompida ou incompatível.

**Diagnóstico:**

```bash
# Verificar qual binário o shell encontra
which capiba-mcp

# Testar o binário compilado localmente (deve funcionar)
./target/release/capiba-mcp --version
```

**Solução:**

```bash
./scripts/build.sh
# Se ~/.cargo/bin/capiba-mcp existir, substituir também:
cp target/release/capiba-mcp ~/.cargo/bin/capiba-mcp
```

---

## Toggle do MCP desativado no painel do agente

**Sintoma:** O indicador do `capiba-mcp` no Agent Panel fica cinza.

**Causa:** A extensão precisa de `zed_extension_api >= 0.5` e da implementação
do método `context_server_command()`.

**Verificar:**

```bash
grep 'zed_extension_api' Cargo.toml
# deve ser: zed_extension_api = "0.5"
```

---

## Usando o capiba-mcp no Claude Code

```bash
claude mcp add --scope user capiba-mcp /usr/local/bin/capiba-mcp-run
claude mcp list
# capiba-mcp: /usr/local/bin/capiba-mcp-run - ✓ Connected
```
