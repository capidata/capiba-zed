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

## Configurando o Zed

### 1. Instalar a extensão como dev extension

**Clonar o repositório:**
```bash
git clone https://github.com/capidata/capiba-zed
cd capiba-zed
```

**No Zed:**
1. Abrir `Command Palette` (`Cmd+Shift+P` ou `Ctrl+Shift+P`)
2. Digitar `zed: install dev extension`
3. Selecionar a pasta `capiba-zed`
4. Reiniciar o Zed

### 2. Configurar ANTHROPIC_API_KEY

**Em `~/.config/zed/settings.json` (Linux/macOS) ou `%APPDATA%\Zed\settings.json` (Windows):**

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

### 3. Configurar capiba-mcp no Zed

**Em `~/.config/zed/settings.json`:**

```json
{
  "assistant": {
    "default_model": {
      "provider": "anthropic",
      "model": "claude-3-5-sonnet-20241022"
    }
  }
}
```

Se o `capiba-mcp` estiver instalado em `/usr/local/bin/capiba-mcp`, o Zed o encontrará automaticamente via a extensão.

Se instalou em local customizado, configure em `extension.toml`:
```toml
[context_servers.capiba-mcp]
command = "/caminho/completo/capiba-mcp"
args = []
```

### 4. Usar a extensão

**No Zed, abra qualquer repositório do ecossistema Capiba e:**

1. Pressione `Cmd+?` (macOS) ou `Ctrl+?` (Linux/Windows) para abrir o Assistant
2. Digite um dos slash commands:
   - `/capiba-onboard` — Começar
   - `/capiba-fase preparacao` — Guia da Fase 1
   - `/capiba-historia` — Escrever história de contribuição
   - `/capiba-revisar` — Revisar código

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

Mínimo exigido: **95% de cobertura de linhas** (verificado no pre-commit).

---

## Releases e Downloads

Toda tag `v*` (ex: `v0.1.0`) dispara o workflow de Continuous Deployment que:

1. Builda a extensão Zed em WASM
2. Compila `capiba-mcp` para 6 plataformas:
3. - `capiba-mcp-linux-aarch64` (ARM)
   - `capiba-mcp-linux-x86_64` (Intel/AMD)
   - `capiba-mcp-macos-aarch64` (ARM)
   - `capiba-mcp-macos-x86_64` (Intel)
   - `capiba-mcp-windows-aarch64.exe` (ARM)
   - `capiba-mcp-windows-x86_64.exe` (Intel/AMD)
3. Publica uma GitHub Release com todos os binários

**Para criar um novo release:**

```bash
git tag v0.2.0
git push origin v0.2.0
```

Os artefatos estarão disponíveis em [Releases](https://github.com/capidata/capiba-zed/releases).

### Como instalar o `capiba-mcp`

Baixe o binário correspondente ao seu sistema em [Releases](https://github.com/capidata/capiba-zed/releases) e torne-o executável.

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

**macOS (Apple Silicon — M1/M2/M3):**
```bash
curl -L https://github.com/capidata/capiba-zed/releases/download/v0.1.0/capiba-mcp-macos-aarch64 -o capiba-mcp
chmod +x capiba-mcp
sudo mv capiba-mcp /usr/local/bin/
```

**macOS (Intel):**
```bash
curl -L https://github.com/capidata/capiba-zed/releases/download/v0.1.0/capiba-mcp-macos-x86_64 -o capiba-mcp
chmod +x capiba-mcp
sudo mv capiba-mcp /usr/local/bin/
```

**Windows (x86_64):**
```powershell
# Baixar em %USERPROFILE%\Downloads:
Invoke-WebRequest -Uri "https://github.com/capidata/capiba-zed/releases/download/v0.1.0/capiba-mcp-windows-x86_64.exe" `
  -OutFile capiba-mcp.exe

# Mover para PATH (ex: C:\Program Files\capiba):
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

capiba-mcp --help
# Mostra como usar o servidor
```

**Nota:** `capiba-mcp` é um servidor MCP que roda via stdio. Não execute sem contexto — ele é usado automaticamente pela extensão Zed ou por clientes MCP compatíveis.

---

## Licença

Distribuído sob a **Licença Ética Capiba** —
livre para commons, cooperativas e pesquisa;
vedado para vigilância, venda de dados e publicidade comportamental.

---

_github.com/capidata/capiba-zed — Pernambuco, 2026_
