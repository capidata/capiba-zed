# CLAUDE.md — capiba-zed

Integração Claude + Zed IDE para o ecossistema Capiba.

---

## Sobre este repositório

`capiba-zed` é a extensão nativa do Zed que traz o Claude como assistente
do ecossistema Capiba — com slash commands para as 5 fases do processo de
contribuição, onboarding, ideação e um servidor MCP que expõe o protocolo
Capiba como contexto vivo para o agente.

---

## Lugar no ecossistema

```
Camada:      Ferramental (não é IaaS/PaaS/SaaS — é infraestrutura de dev)
Depende de:  .github/estatuto (Pacto Fundante)
             .github/decisoes (ledger público)
             capiba-core/spec → AINDA NÃO EXISTE, integração prevista para v0.2
Consumido por: desenvolvedores do ecossistema no Zed
```

---

## Stack

| Crate              | Target          | Função                                        |
| ------------------ | --------------- | --------------------------------------------- |
| `capiba-prompts`   | puro Rust       | Prompts e constantes compartilhados            |
| `capiba-zed` (lib) | `wasm32-wasip1` | Extensão Zed — slash commands                 |
| `capiba-mcp` (bin) | host nativo     | Servidor MCP — tools e resources              |

```toml
# dependências principais
capiba-prompts = { path = "prompts" }  # fonte única de prompts
zed_extension_api = "0.4"             # extensão WASM
rmcp = "0.16"                         # SDK MCP oficial
serde + serde_json = "1"              # serialização MCP
tokio = "1"                           # async no MCP server
anyhow = "1"                          # erros ergonômicos
```

---

## Estrutura

```
capiba-zed/
├── Cargo.toml              workspace (extensão + mcp-server + prompts)
├── extension.toml          manifesto Zed
├── src/
│   ├── lib.rs              slash commands (Extension trait)
│   └── prompts.rs          re-exporta capiba-prompts (compatibilidade)
├── prompts/
│   ├── Cargo.toml
│   └── src/lib.rs          fonte única: prompts + PRINCIPIOS + constantes
└── mcp-server/
    ├── Cargo.toml
    └── src/
        └── main.rs         servidor MCP via stdio (rmcp 0.16)
```

---

## Slash commands disponíveis

| Comando               | Fase | O que faz                           |
| --------------------- | ---- | ----------------------------------- |
| `/capiba-onboard`     | —    | Onboarding de novo contribuidor     |
| `/capiba-fase <nome>` | 1–5  | Guia da fase atual do processo      |
| `/capiba-historia`    | 1    | Escreve história de contribuição    |
| `/capiba-ideia`       | 1    | Sessão de ideação de nova feature   |
| `/capiba-revisar`     | 3    | Revisão ética e técnica do código   |
| `/capiba-teste`       | 3    | Gera testes contextualizados        |
| `/capiba-compat`      | 3    | Verifica compatibilidade com o Core |
| `/capiba-pr`          | 3–4  | Gera descrição de PR com checklists |

---

## MCP tools disponíveis

| Tool                    | O que faz                                                 |
| ----------------------- | --------------------------------------------------------- |
| `capiba_check_compat`   | Análise estática contra princípios do Core                |
| `capiba_get_principio`  | Retorna princípio inviolável (1–7) com implicação técnica |
| `capiba_get_decisao`    | Lê decisão do ledger por número                           |
| `capiba_gerar_historia` | Estrutura história de contribuição                        |
| `capiba_fase_atual`     | Detecta fase do processo pelo estado do worktree          |

## MCP resources disponíveis

| URI                     | Conteúdo                                      |
| ----------------------- | --------------------------------------------- |
| `capiba://principios`   | Os 7 princípios invioláveis (embutidos)       |
| `capiba://pacto`        | Pacto Fundante da CAPIDATA                    |
| `capiba://contributing` | Processo de contribuição em 5 fases           |
| `capiba://decisoes`     | Ledger público de decisões                    |

---

## Como buildar

```bash
# 0. Configurar pre-commit hooks (uma vez por clone)
pip install pre-commit && pre-commit install

# 1. Target WASM (uma vez)
rustup target add wasm32-wasip1

# 2. Extensão Zed
cargo build --release --target wasm32-wasip1

# 3. Servidor MCP
cargo build --release -p capiba-mcp

# 4. Instalar no Zed como dev extension
# Command Palette → "zed: install dev extension" → selecionar esta pasta
```

**Alternativa (hook bash):**
```bash
git config core.hooksPath .githooks
```

---

## Processo de contribuição

Todo trabalho segue as 5 fases do CONTRIBUTING.md.
Use `/capiba-fase <nome>` para guia de cada fase.

**Antes de abrir PR:**

- [ ] `cargo clippy --all-targets` sem warnings
- [ ] `cargo test --all` passando
- [ ] Slash commands testadas manualmente no Zed (`zed: install dev extension`)
- [ ] `capiba_check_compat` rodado no código novo
- [ ] Princípios invioláveis verificados (especialmente V — IA não age sem confirmação)

---

## Decisões de arquitetura

**rmcp como SDK MCP (v0.1)**
O `capiba-mcp` usa o SDK Rust oficial `rmcp 0.16` sobre stdio.
Ref: `.github/decisoes/2026/0002-capiba-mcp-sem-sdk.md`

**Prompts como constantes Rust (`capiba-prompts` crate)**
Alternativa considerada: arquivos `.md` lidos em runtime.
Decisão: constantes compiladas garantem funcionamento offline
e são compartilháveis entre a extensão WASM e o servidor nativo.

**`capiba://principios` em vez de `capiba://spec/core`**
O `capiba-core` não existe ainda. Os 7 princípios invioláveis
são embutidos no crate `capiba-prompts` e servidos diretamente.
Quando o `capiba-core` existir, o resource será adicionado.

**`CAPIBA_ROOT` como variável de ambiente**
O MCP server detecta a raiz do repositório via `CAPIBA_ROOT`
ou `CWD`. Isso permite uso em monorepos e workspaces aninhados.

---

## Variáveis de ambiente

| Variável            | Padrão | Uso                                           |
| ------------------- | ------ | --------------------------------------------- |
| `CAPIBA_ROOT`       | `CWD`  | Raiz do repositório para leitura de resources |
| `ANTHROPIC_API_KEY` | —      | Configurado no Zed settings, não aqui         |

---

## Roadmap

```
v0.1 — Estado atual (pós-revisão de escopo)
  [x] 8 slash commands das 5 fases
  [x] Leitura de contexto do worktree (CLAUDE.md, CONTRIBUTING)
  [x] MCP server com 5 tools e 4 resources
  [x] Princípios invioláveis embutidos no crate capiba-prompts
  [x] capiba://principios como resource (substituiu capiba://spec/core)
  [x] capiba_get_decisao busca recursiva (sem hardcode de ano)
  [x] Prompts centralizados em capiba-prompts (fonte única)
  [x] Testes de fumaça no MCP server

v0.2 — Quando capiba-core existir
  [ ] Resource capiba://spec/core real
  [ ] capiba_check_compat com análise semântica real
  [ ] Resource capiba://delta para cálculo de δ efetivo
  [ ] /capiba-compat gerando COMPAT.md real contra módulos do Core

v0.3 — Agent Skills
  [ ] Subagentes por fase (agente de preparação, de garantia, etc.)
  [ ] Integração com CapibaGov (criar issues e decisões via MCP)
  [ ] Suporte a múltiplos repositórios no mesmo workspace

v1.0 — Publicação
  [ ] Publicar no Zed Extension Registry
  [ ] Documentação completa em pt-BR
  [ ] CLAUDE.md padronizado em todos os repos do ecossistema
```

---

_github.com/capidata/capiba-zed — Pernambuco, 2026_
