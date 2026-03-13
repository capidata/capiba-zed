# Postmortems — capiba-zed

## 2026-03-13 — MCP não ativava no Zed

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
