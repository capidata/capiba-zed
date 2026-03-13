# capiba-zed

> Integração Claude + Zed IDE para o ecossistema Capiba

Extensão nativa do [Zed](https://zed.dev) que traz o Claude como assistente
do ecossistema **Capiba** — o protocolo anti-colonial de soberania, inteligência
e governança popular de dados.

Parte da [CAPIDATA](https://github.com/capidata) — comunidade construtora do protocolo.

---

## O que faz

**Slash commands para as 5 fases do processo de contribuição:**

```text
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

```text
Tools:     capiba_check_compat · capiba_get_principio · capiba_get_decisao
           capiba_gerar_historia · capiba_fase_atual

Resources: capiba://principios · capiba://pacto
           capiba://contributing · capiba://decisoes
```

---

## Uso

Abra qualquer repositório do ecossistema Capiba no Zed, pressione `Cmd+?`
e use os slash commands:

```text
/capiba-onboard               → começar aqui na primeira vez
/capiba-fase preparacao       → nomear o problema e o sujeito
/capiba-fase desenvolvimento  → par de programação contextualizado
/capiba-fase garantia         → revisão técnica, ética e territorial
/capiba-fase entrega          → release notes e versionamento
/capiba-fase consolidacao     → memória coletiva e relato territorial
```

```text
/capiba-revisar    → 🔴 violações / 🟡 melhorias / 🟢 sugestões
/capiba-compat     → checklist dos 8 princípios invioláveis
/capiba-pr         → descrição de PR com checklists técnico e ético
```

---

## Documentação

| Documento                                          | Conteúdo                                    |
| -------------------------------------------------- | ------------------------------------------- |
| [docs/setup.md](docs/setup.md)                     | Instalação, build, configuração do Zed      |
| [docs/architecture.md](docs/architecture.md)       | Estrutura, stack, decisões de design        |
| [docs/quality.md](docs/quality.md)                 | Testes, cobertura, processo de contribuição |
| [docs/troubleshooting.md](docs/troubleshooting.md) | Problemas conhecidos e soluções             |
| [docs/postmortems.md](docs/postmortems.md)         | Incidentes registrados                      |

---

## Licença

Distribuído sob a **Licença Ética Capiba** —
livre para commons, cooperativas e pesquisa;
vedado para vigilância, venda de dados e publicidade comportamental.

---

_github.com/capidata/capiba-zed — Pernambuco, 2026_
