# Qualidade — capiba-zed

## Checklist antes de abrir PR

- [ ] `cargo clippy --all-targets` sem warnings
- [ ] `cargo test --all` passando
- [ ] Slash commands testadas manualmente no Zed (`zed: install dev extension`)
- [ ] `capiba_check_compat` rodado no código novo
- [ ] Princípios invioláveis verificados (especialmente V — IA não age sem confirmação)
- [ ] Checklists do `CONTRIBUTING.md` preenchidos

---

## Cobertura de testes

Mínimo exigido: **95% de cobertura de linhas** (verificado no pre-commit).

```bash
# Relatório visual (abre no browser)
cargo llvm-cov -p capiba-mcp --html --open

# Resumo no terminal
cargo llvm-cov -p capiba-mcp --summary-only
```

---

## Processo de contribuição

Este repositório segue as 5 fases do CONTRIBUTING.md da CAPIDATA.

```bash
# Depois de clonar:
/capiba-onboard             # entenda o projeto
/capiba-fase preparacao     # comece pela história e pelo sujeito
```

Use `/capiba-fase <nome>` para o guia de cada fase:

```text
preparacao       → nomear o problema e o sujeito
desenvolvimento  → par de programação contextualizado
garantia         → revisão técnica, ética e territorial
entrega          → release notes e versionamento
consolidacao     → memória coletiva e relato territorial
```
