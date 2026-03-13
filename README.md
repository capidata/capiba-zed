# capiba-zed

> Integração Claude + Zed IDE para o ecossistema Capiba

Extensão nativa do [Zed](https://zed.dev) que traz o Claude como assistente
do ecossistema **Capiba** — o protocolo anti-colonial de soberania, inteligência
e governança popular de dados.

Parte da [CAPIDATA](https://github.com/capidata) — comunidade construtora do protocolo.

---

## O que faz

**Slash commands para as 5 fases do processo de contribuição:**

```
/capiba-onboard               → onboarding de novo contribuidor
/capiba-fase <fase>           → guia da fase atual do processo
/capiba-historia              → escrever história de contribuição
/capiba-ideia                 → sessão de ideação de nova feature
/capiba-revisar               → revisão ética e técnica do código
/capiba-teste                 → gerar testes contextualizados
/capiba-compat                → verificar compatibilidade com o Core
/capiba-pr                    → gerar descrição de PR com checklists
```

**Servidor MCP com acesso ao protocolo:**

```
Tools:     capiba_check_compat · capiba_get_principio · capiba_get_decisao
           capiba_gerar_historia · capiba_fase_atual

Resources: capiba://principios · capiba://pacto
           capiba://contributing · capiba://decisoes
```

---

## Instalação

### Pré-requisitos

- [Zed](https://zed.dev) instalado
- [Rust](https://rustup.rs) com target WASM:
  ```bash
  rustup target add wasm32-wasip1
  ```
- `ANTHROPIC_API_KEY` configurada nas settings do Zed

### Como instalar

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

No Zed:

1. `Command Palette` → `zed: install dev extension`
2. Selecionar a pasta `capiba-zed/`
3. Abrir o painel Assistant (`Ctrl+?` ou `Cmd+?`)
4. Digitar `/capiba-onboard` para começar

---

## Uso

### Onboarding

Ao entrar em qualquer repositório do ecossistema pela primeira vez:

```
/capiba-onboard
```

O assistente apresenta o projeto, identifica sua área de interesse
e sugere os primeiros passos concretos.

### Guia das 5 fases

```
/capiba-fase preparacao       → nomear o problema e o sujeito
/capiba-fase desenvolvimento  → par de programação contextualizado
/capiba-fase garantia         → revisão técnica, ética e territorial
/capiba-fase entrega          → release notes e versionamento
/capiba-fase consolidacao     → memória coletiva e relato territorial
```

### Ideação

```
/capiba-ideia
```

Sessão estruturada para propor nova feature ou solução —
do problema real ao template de Issue formatado.

### Revisão antes do PR

```
/capiba-revisar    → alerta 🔴 violações / 🟡 melhorias / 🟢 sugestões
/capiba-compat     → checklist dos 7 princípios invioláveis
/capiba-pr         → descrição de PR com checklists técnico e ético
```

---

## Como o contexto funciona

A extensão lê automaticamente os arquivos do worktree aberto:

```
CLAUDE.md              → contexto do projeto (arquitetura, decisões, padrões)
CONTRIBUTING.md        → processo de contribuição em 5 fases
```

O servidor MCP expõe recursos adicionais para o Claude Agent:

```
capiba://principios    → os 7 princípios invioláveis do Pacto Fundante
capiba://pacto         → Pacto Fundante da CAPIDATA
capiba://contributing  → processo de contribuição
capiba://decisoes      → ledger público de decisões
```

Defina `CAPIBA_ROOT` para apontar para a raiz do repositório
se estiver usando em monorepo ou workspace aninhado.

---

## Estrutura do repositório

```
capiba-zed/
├── Cargo.toml              workspace (extensão + mcp-server + prompts)
├── extension.toml          manifesto Zed
├── CLAUDE.md               contexto para o Claude Agent
├── src/
│   └── lib.rs              slash commands (Extension trait)
├── prompts/
│   └── src/lib.rs          prompts e constantes compartilhados (crate puro)
└── mcp-server/
    ├── Cargo.toml
    └── src/
        └── main.rs         servidor MCP via stdio (rmcp)
```

---

## Contribuindo

Este repositório segue o mesmo processo de contribuição de todo o ecossistema Capiba.

```bash
# Depois de clonar:
/capiba-onboard             # entenda o projeto
/capiba-fase preparacao     # comece pela história e pelo sujeito
```

Antes de abrir PR:

- `cargo clippy --all-targets` sem warnings
- `cargo test --all` passando
- Slash commands testadas manualmente no Zed
- Checklists do `CONTRIBUTING.md` preenchidos

### Cobertura de testes

```bash
# Relatório visual (abre no browser)
cargo llvm-cov -p capiba-mcp --html --open

# Resumo no terminal
cargo llvm-cov -p capiba-mcp --summary-only
```

Mínimo exigido: **90% de cobertura de linhas** (verificado no pre-commit).

---

## Licença

Distribuído sob a **Licença Ética Capiba** —
livre para commons, cooperativas e pesquisa;
vedado para vigilância, venda de dados e publicidade comportamental.

---

_github.com/capidata/capiba-zed — Pernambuco, 2026_
