# Postmortems — capiba-zed

## 2026-03-13 — Slash commands e MCP não funcionavam fora do capiba-zed

**Reportado por:** Silvano Neto
**Duração:** ~4h de troubleshooting
**Severidade:** Slash commands e MCP inoperantes em qualquer repositório exceto capiba-zed

### Sintomas

1. `/capiba-*` não aparecia no autocomplete em `capiba-core`
2. Após investigação, `Context server request timeout` (60s) no MCP
3. `capiba-mcp --version` retornava `zsh: killed` (exit 137) de locais específicos

### Causas raiz (em cascata)

**1. Extensão instalada em path errado pelo `install.sh`**
O script instalava em `~/.config/zed/extensions/` mas o Zed no macOS usa
`~/Library/Application Support/Zed/extensions/installed/`. Mesmo com os arquivos
corretos no lugar certo, o Zed ignora WASM compilado externamente para slash commands.
Solução: `zed: install dev extension` é o único método funcional para dev.

**2. Extensão dev não funciona em outros workspaces**
Com `zed: install dev extension`, slash commands só aparecem quando `capiba-zed` está
no workspace ativo. Não é um bug — é o comportamento esperado de extensões dev no Zed.
Solução: adicionar `capiba-zed` ao workspace via `File > Add Folder to Project`.

**3. WASM gerado externamente incompatível**
O `cargo build --target wasm32-wasip1` gera um WASM module. O Zed exige um WASM
Component. Tentativa de converter com `wasm-tools component new` falhou porque o
módulo usa WASI Preview 1 e precisa de adapter específico.
Solução: usar exclusivamente `zed: install dev extension`.

**4. Buffering do stdout no rmcp 0.16**
O binário `capiba-mcp` responde corretamente ao protocolo MCP quando testado via
terminal, mas o Zed dá timeout porque o rmcp/tokio bufferiza o stdout sem flush
automático. O Zed aguarda 60s pela resposta do `initialize` e desiste.
Descoberta: ao usar um script wrapper com pipeline `tee`, o flush era forçado e
o MCP funcionava. `stdbuf -o0` não funciona porque o tokio não usa a libc para I/O.
Solução: wrapper `capiba-mcp-run` com `tee /tmp/in.log | capiba-mcp | tee /tmp/out.log`.

**5. `HOME` vazio no contexto WASM**
`std::env::var("HOME")` retorna string vazia no sandbox WASM do Zed. O
`context_server_command()` montava o path como `/.local/bin/capiba-mcp-run`
(inexistente). `std::fs::metadata()` também é bloqueado pelo sandbox.
Solução: usar `/usr/local/bin/capiba-mcp-run` como path fixo, instalado pelo
`build.sh` com `sudo`.

**6. MCP não disponível fora do contexto da extensão**
Mesmo com tudo funcionando, o MCP só iniciava quando a extensão estava carregada
(capiba-zed no workspace). Para uso global, precisa ser configurado independentemente.
Solução: entrada em `~/.config/zed/settings.json` com o formato correto:

```json
{ "context_servers": { "capiba-mcp": { "command": { "path": "...", "args": [] } } } }
```

### Aprendizados

- O Zed não aceita WASM compilado externamente — `zed: install dev extension` é mandatório
- Extensões dev só funcionam no workspace onde foram instaladas
- O rmcp 0.16 não faz flush automático do stdout — wrapper com `tee` é necessário
- `std::env::var("HOME")` e `std::fs::metadata()` não funcionam no sandbox WASM do Zed
- O formato correto do `settings.json` para context servers é `{"command": {"path": "...", "args": []}}`
- O Zed loga todos os erros de MCP/extensão em `~/Library/Logs/Zed/Zed.log`

### Arquivos alterados

```text
src/lib.rs              context_server_command: /usr/local/bin/capiba-mcp-run (path fixo)
extension.toml          schema_version 0 → 1; command: capiba-mcp-run
scripts/build.sh        cria wrapper capiba-mcp-run; instala em /usr/local/bin com sudo
scripts/install.sh      mantido mas não resolve slash commands (documentado)
~/.config/zed/settings.json   context_servers: capiba-mcp com path para o wrapper
CLAUDE.md               processo de instalação real + limitações conhecidas
docs/setup.md           reescrito com processo correto
docs/troubleshooting.md reescrito com problemas reais encontrados
docs/postmortems.md     este registro
```

---

## 2026-03-13 — MCP não ativava no Zed (incidente inicial)

**Reportado por:** Silvano Neto
**Duração:** ~2h de troubleshooting
**Severidade:** MCP completamente inoperante após instalação inicial

### Sintomas

1. Toggle do `capiba-mcp` no Agent Panel ficava desativado e não ativava
2. Após tentativa de forçar ativação: `Context server request timeout` (60s)
3. Log do Zed: `data did not match any variant of untagged enum ContextServerSettingsContent`

### Causa raiz

Três problemas independentes, todos necessários para o funcionamento:

**1. `context_server_command()` não implementado (bloqueador principal)**
O `src/lib.rs` usava `zed_extension_api = "0.4"` e não implementava o método
`context_server_command()`. A partir da v0.5 desse crate, o Zed exige que a
extensão implemente esse método para iniciar servidores MCP. Sem ele, o Zed
loga `context_server_command not implemented` e não tenta iniciar o servidor.

**2. Binário `capiba-mcp` desatualizado em `/usr/local/bin/`**
Após a correção da extensão WASM, o servidor iniciava mas travava na
inicialização MCP. O binário instalado era de uma compilação anterior. O novo
binário respondia corretamente ao protocolo (testado via stdin/stdout direto),
mas o Zed usava o binário antigo que não respondia.

**3. Formato inválido no `settings.json`**
A entrada `context_servers` usava `"remote": false` e `"enabled": true` sem
`"source"`, formato não reconhecido pelo Zed. O campo `"source": "custom"` com
caminho absoluto é obrigatório para servidores configurados manualmente.

### Resolução

1. `Cargo.toml`: `zed_extension_api = "0.4"` → `"0.5"`
2. `src/lib.rs`: implementado `context_server_command()` no `impl zed::Extension`
3. Rebuild WASM: `cargo build --release --target wasm32-wasip1`
4. Rebuild binário: `cargo build --release -p capiba-mcp`
5. Reinstalar binário: `sudo cp target/release/capiba-mcp /usr/local/bin/capiba-mcp`
6. `settings.json`: entrada corrigida para `"source": "custom"` com caminho absoluto

### Aprendizados

- A `zed_extension_api 0.4` e `0.5` são incompatíveis no suporte a MCP: a v0.4
  não suporta `context_server_command()` nem `context_server_configuration()`
- O Zed não exibe erros de MCP na UI — todos os diagnósticos estão em
  `~/Library/Logs/Zed/Zed.log`
- O binário em `/usr/local/bin/` não é atualizado automaticamente pelo build —
  precisa ser copiado manualmente após cada rebuild que altere o servidor MCP
- O formato `"source": "custom"` não está documentado de forma proeminente;
  formatos alternativos (`"enabled": true`, `"remote": false`) são silenciosamente
  rejeitados

### Arquivos alterados

```text
Cargo.toml              zed_extension_api 0.4 → 0.5
src/lib.rs              +context_server_command() + imports Command, ContextServerId, Project
~/.config/zed/settings.json   context_servers: source: custom + command absoluto
docs/setup.md           instrução correta de configuração do settings.json
docs/troubleshooting.md três cenários de falha documentados
```
