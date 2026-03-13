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

```text
Camada:      Ferramental (não é IaaS/PaaS/SaaS — é infraestrutura de dev)
Depende de:  .github/estatuto (Pacto Fundante)
             .github/decisoes (ledger público)
             capiba-core/spec → AINDA NÃO EXISTE, integração prevista para v0.2
Consumido por: desenvolvedores do ecossistema no Zed
```

---

## Stack

| Crate              | Target          | Função                              |
| ------------------ | --------------- | ----------------------------------- |
| `capiba-prompts`   | puro Rust       | Prompts e constantes compartilhados |
| `capiba-zed` (lib) | `wasm32-wasip1` | Extensão Zed — slash commands       |
| `capiba-mcp` (bin) | host nativo     | Servidor MCP — tools e resources    |

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

```text
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

| URI                     | Conteúdo                                |
| ----------------------- | --------------------------------------- |
| `capiba://principios`   | Os 7 princípios invioláveis (embutidos) |
| `capiba://pacto`        | Pacto Fundante da CAPIDATA              |
| `capiba://contributing` | Processo de contribuição em 5 fases     |
| `capiba://decisoes`     | Ledger público de decisões              |

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
- [ ] **MCP Agnóstico**: código não importa `anthropic`, `claude-api` ou SDKs de modelo específico
- [ ] **Teste multi-cliente**: tool testado com pelo menos 2 clientes MCP (Claude + outro)

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

## Agnóstico de modelo — Implementação MCP pura

`capiba-zed` **não depende de SDKs específicos de Claude ou Anthropic** — apenas do protocolo MCP v1.0 oficialmente publicado. Isso garante que:

1. **O servidor funciona com qualquer cliente MCP**: Claude, Grok, clientes customizados, CLI
2. **Nenhum lock-in de modelo**: quebras de compatibilidade do SDK Anthropic não afetam capiba-zed
3. **Compatível com o Princípio II (Anti-colonial)**: ferramenta não fica dependente de extrator específico

### Checklist de pureza MCP

Ao modificar `mcp-server/`:

- ✅ Usar apenas `rmcp` (SDK MCP oficial Rust)
- ✅ Expor tools com `inputSchema` JSON-Schema completo
- ✅ Nenhuma suposição sobre inteligência do cliente
- ✅ Testar com múltiplos clientes (Claude + outro MCP client, mínimo)
- ❌ Nunca importar `anthropic_sdk`, `claude-api`, ou equivalentes
- ❌ Nunca acoplaria prompt à "inteligência" do modelo específico

### Arquitetura de transporte

```text
┌─────────────────────────────────┐
│   Clientes MCP                  │
│   (Claude, Grok, CLI, custom)   │
├─────────────────────────────────┤
│   MCP via stdio (spec v1.0)      │  ← agnóstico
├─────────────────────────────────┤
│   capiba-mcp server (Rust)       │
│   (rmcp 0.16, tokio, serde)      │  ← puro
├─────────────────────────────────┤
│   Capiba Protocol Engine         │
│   (tools + resources)            │  ← lógica
└─────────────────────────────────┘
```

O fluxo é unidirecional: cliente → stdio → server. Nada de callbacks para o modelo.

### Evolução esperada

Se a Anthropic descontinuar o `rmcp` ou se surgir alternativa oficial melhor, capiba-zed troca o SDK **mas não o protocolo** — MCP é público. Tools, resources, argumentos permanecem os mesmos.

---

## Variáveis de ambiente

| Variável            | Padrão | Uso                                           |
| ------------------- | ------ | --------------------------------------------- |
| `CAPIBA_ROOT`       | `CWD`  | Raiz do repositório para leitura de resources |
| `ANTHROPIC_API_KEY` | —      | Configurado no Zed settings, não aqui         |

---

## Roadmap

```text
v0.1 — Estado atual (pós-revisão de escopo)
  [x] 8 slash commands das 5 fases
  [x] Leitura de contexto do worktree (CLAUDE.md, CONTRIBUTING)
  [x] MCP server com 5 tools e 4 resources
  [x] Princípios invioláveis embutidos no crate capiba-prompts
  [x] capiba://principios como resource (substituiu capiba://spec/core)
  [x] capiba_get_decisao busca recursiva (sem hardcode de ano)
  [x] Prompts centralizados em capiba-prompts (fonte única)
  [x] Testes de fumaça no MCP server

v0.2 — Agnóstico de modelo + capiba-core
  [ ] MCP agnóstico: remover qualquer import de anthropic/claude-api
  [ ] Transporte stdio puro (spec v1.0, nenhum callback para modelo)
  [ ] Resource capiba://spec/core real (quando capiba-core existir)
  [ ] capiba_check_compat com análise semântica real
  [ ] Resource capiba://delta para cálculo de δ efetivo
  [ ] /capiba-compat gerando COMPAT.md real contra módulos do Core
  [ ] Teste multi-cliente: tools funcionando em Claude + Grok (ou CLI client)
  [ ] Auditoria de agnóstico: verificar que rmcp 0.16 é a única dependência MCP

v0.3 — Agent Skills
  [ ] Subagentes por fase (agente de preparação, de garantia, etc.)
  [ ] Integração com CapibaGov (criar issues e decisões via MCP)
  [ ] Suporte a múltiplos repositórios no mesmo workspace
  [ ] Suporte a protocolos MCP alternativos (se spec evoluir)
  [ ] Teste de compatibilidade: agent com múltiplos clientes MCP

v1.0 — Publicação
  [ ] Publicar no Zed Extension Registry
  [ ] Documentação completa em pt-BR
  [ ] CLAUDE.md padronizado em todos os repos do ecossistema
  [ ] Garantia de agnóstico: publicar conformance test suite público
  [ ] Certificação MCP v1.0: verificação independente de pureza
```

---

_github.com/capidata/capiba-zed — Pernambuco, 2026_
