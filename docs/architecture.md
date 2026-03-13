# Arquitetura — capiba-zed

## Lugar no ecossistema

```text
Camada:       Ferramental (infraestrutura de dev)
Depende de:   .github/estatuto (Pacto Fundante)
              .github/decisoes (ledger público)
              capiba-core/spec → ainda não existe, previsto para v0.2
Consumido por: desenvolvedores do ecossistema no Zed
```

---

## Stack

| Crate              | Target          | Função                              |
| ------------------ | --------------- | ----------------------------------- |
| `capiba-prompts`   | puro Rust       | Prompts e constantes compartilhados |
| `capiba-zed` (lib) | `wasm32-wasip1` | Extensão Zed — slash commands       |
| `capiba-mcp` (bin) | host nativo     | Servidor MCP — tools e resources    |

Dependências principais:

```toml
zed_extension_api = "0.5"             # extensão WASM
rmcp = "0.16"                         # SDK MCP oficial (stdio)
serde + serde_json = "1"              # serialização MCP
tokio = "1"                           # async no MCP server
```

---

## Estrutura do repositório

```text
capiba-zed/
├── Cargo.toml              workspace (extensão + mcp-server + prompts)
├── extension.toml          manifesto Zed
├── CLAUDE.md               contexto para o Claude Agent
├── src/
│   └── lib.rs              slash commands + context_server_command (Extension trait)
├── prompts/
│   └── src/lib.rs          fonte única: prompts + PRINCIPIOS + constantes
├── mcp-server/
│   ├── Cargo.toml
│   └── src/
│       └── main.rs         servidor MCP via stdio (rmcp 0.16)
└── docs/
    ├── architecture.md     este arquivo
    ├── setup.md            instalação e configuração
    ├── quality.md          testes e processo de contribuição
    ├── troubleshooting.md  problemas conhecidos e soluções
    └── postmortems.md      incidentes registrados
```

---

## Como o contexto funciona

A extensão WASM lê automaticamente os arquivos do worktree aberto:

```text
CLAUDE.md              → contexto do projeto (arquitetura, decisões, padrões)
CONTRIBUTING.md        → processo de contribuição em 5 fases
```

O servidor MCP expõe recursos adicionais via protocol:

```text
capiba://principios    → os 8 princípios invioláveis do Pacto Fundante
capiba://pacto         → Pacto Fundante da CAPIDATA
capiba://contributing  → processo de contribuição
capiba://decisoes      → ledger público de decisões
```

Defina `CAPIBA_ROOT` para apontar para a raiz do repositório
se estiver usando em monorepo ou workspace aninhado.

---

## Decisões de arquitetura

**rmcp como SDK MCP**
O `capiba-mcp` usa o SDK Rust oficial `rmcp 0.16` sobre stdio.
Ref: `.github/decisoes/2026/0002-capiba-mcp-sem-sdk.md`

**Prompts como constantes Rust (`capiba-prompts` crate)**
Alternativa considerada: arquivos `.md` lidos em runtime.
Decisão: constantes compiladas garantem funcionamento offline
e são compartilháveis entre a extensão WASM e o servidor nativo.

**`capiba://principios` em vez de `capiba://spec/core`**
O `capiba-core` não existe ainda. Os 8 princípios invioláveis
são embutidos no crate `capiba-prompts` e servidos diretamente.
Quando o `capiba-core` existir, o resource será adicionado.

**`CAPIBA_ROOT` como variável de ambiente**
O MCP server detecta a raiz do repositório via `CAPIBA_ROOT`
ou `CWD`. Isso permite uso em monorepos e workspaces aninhados.

**`context_server_command()` na extensão WASM**
A partir da `zed_extension_api 0.5`, servidores MCP fornecidos por
extensões precisam implementar `context_server_command()` no `Extension`
trait. Sem esse método, o Zed não inicia o servidor. Complementarmente,
o settings.json deve usar `"source": "custom"` com caminho absoluto.

---

## Variáveis de ambiente

| Variável            | Padrão | Uso                                           |
| ------------------- | ------ | --------------------------------------------- |
| `CAPIBA_ROOT`       | `CWD`  | Raiz do repositório para leitura de resources |
| `ANTHROPIC_API_KEY` | —      | Configurado no Zed settings, não aqui         |

---

## Roadmap

```text
v0.1 — Estado atual
  [x] 8 slash commands das 5 fases
  [x] Leitura de contexto do worktree (CLAUDE.md, CONTRIBUTING)
  [x] MCP server com 5 tools e 4 resources
  [x] 8 princípios invioláveis embutidos no crate capiba-prompts
  [x] context_server_command() implementado (zed_extension_api 0.5)
  [x] capiba_get_decisao busca recursiva (sem hardcode de ano)

v0.2 — Quando capiba-core existir
  [ ] Resource capiba://spec/core real
  [ ] capiba_check_compat com análise semântica real
  [ ] Resource capiba://delta para cálculo de δ efetivo

v0.3 — Agent Skills
  [ ] Subagentes por fase
  [ ] Integração com CapibaGov (criar issues e decisões via MCP)
  [ ] Suporte a múltiplos repositórios no mesmo workspace

v1.0 — Publicação
  [ ] Publicar no Zed Extension Registry
  [ ] Documentação completa em pt-BR
```
