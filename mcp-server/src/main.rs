// capiba-mcp v0.1 — servidor MCP usando o SDK oficial rmcp

use anyhow::Result;
use capiba_prompts::*;
use rmcp::serde_json::Value;
use rmcp::{
    handler::server::router::tool::ToolRouter, handler::server::wrapper::Parameters, model::*,
    service::RequestContext, tool, tool_handler, tool_router, transport::io::stdio,
    ErrorData as McpError, ServerHandler, ServiceExt,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// ─── Servidor ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct CapibaMcp {
    root: PathBuf,
    tool_router: ToolRouter<Self>,
}

impl CapibaMcp {
    fn new() -> Self {
        let root = std::env::var("CAPIBA_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());
        Self::with_root(root)
    }

    fn with_root(root: PathBuf) -> Self {
        Self {
            root,
            tool_router: Self::tool_router(),
        }
    }

    fn read_file(&self, rel: &str) -> Result<String, McpError> {
        let path = self.root.join(rel);
        std::fs::read_to_string(&path).map_err(|_| {
            McpError::internal_error(format!("arquivo não encontrado: {}", path.display()), None)
        })
    }

    fn read_resource_text(&self, uri: &str) -> Result<String, McpError> {
        match uri {
            "capiba://principios" => Ok(principios_texto()),
            "capiba://pacto" => self.read_file(".github/estatuto/pacto-fundante-v0.1.md"),
            "capiba://contributing" => self.read_file("CONTRIBUTING.md"),
            "capiba://decisoes" => self.read_file(".github/decisoes/README.md"),
            uri => Err(McpError::invalid_params(
                format!("recurso desconhecido: {uri}"),
                None,
            )),
        }
    }
}

// ─── Parâmetros das tools ─────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, JsonSchema)]
struct CheckCompatParams {
    code: String,
    language: Option<String>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct GetPrincipioParams {
    numero: u8,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct GetDecisaoParams {
    id: u32,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct GerarHistoriaParams {
    sujeito: String,
    acao: String,
    objetivo: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct FaseAtualParams {
    worktree_path: String,
}

// ─── Lógica pura ──────────────────────────────────────────────────────────────

fn check_compat_logic(code: &str, lang: &str) -> String {
    let mut alertas: Vec<String> = Vec::new();

    if code.contains("process::exit") || code.contains("std::process::exit") {
        alertas.push("🟡 `process::exit` pode interromper a sincronização de dados locais".into());
    }
    if code.to_lowercase().contains("unwrap()") && lang == "rust" {
        alertas.push(
            "🟡 `unwrap()` em código de produção pode travar o nó em hardware modesto".into(),
        );
    }
    if code.contains("http://") {
        alertas.push("🔴 Conexão HTTP não cifrada — viola soberania de dados em trânsito".into());
    }
    if code.to_lowercase().contains("println!") && code.to_lowercase().contains("senha") {
        alertas.push("🔴 Possível vazamento de dado sensível em log".into());
    }

    if alertas.is_empty() {
        "🟢 Nenhum alerta encontrado na análise de padrões.\n\
         Revise manualmente os 8 princípios com `capiba_get_principio`."
            .to_string()
    } else {
        format!("## Alertas de compatibilidade\n\n{}", alertas.join("\n"))
    }
}

fn get_principio_logic(numero: u8) -> String {
    PRINCIPIOS
        .get((numero as usize).saturating_sub(1))
        .copied()
        .unwrap_or("Princípio não encontrado. Informe um número entre 1 e 8.")
        .to_string()
}

fn gerar_historia_logic(sujeito: &str, acao: &str, objetivo: &str) -> String {
    format!(
        "## História de Contribuição\n\n\
         **Como** {},\n\
         **quero** {},\n\
         **para que** {}.\n\n\
         ### Critérios de aceitação\n\n\
         - [ ] ...\n\n\
         ### Cenários de uso\n\n\
         - Caso feliz: ...\n\
         - Sem internet: ...\n\
         - Hardware modesto: ...\n\n\
         ### Impacto no δ efetivo\n\n\
         ...",
        sujeito, acao, objetivo
    )
}

fn principios_texto() -> String {
    PRINCIPIOS
        .iter()
        .enumerate()
        .map(|(i, p)| format!("## Princípio {}\n\n{}", i + 1, p))
        .collect::<Vec<_>>()
        .join("\n\n---\n\n")
}

fn get_prompt_texto(name: &str, fase: Option<&str>) -> Result<String, McpError> {
    match name {
        "capiba-onboard" => Ok(PROMPT_ONBOARD.to_string()),
        "capiba-revisar" => Ok(PROMPT_REVISAR.to_string()),
        "capiba-pr" => Ok(PROMPT_PR.to_string()),
        "capiba-fase" => {
            let f = fase.unwrap_or("preparacao");
            match f {
                "preparacao" => Ok(PROMPT_FASE_1.to_string()),
                "desenvolvimento" => Ok(PROMPT_FASE_2.to_string()),
                "garantia" => Ok(PROMPT_FASE_3.to_string()),
                "entrega" => Ok(PROMPT_FASE_4.to_string()),
                "consolidacao" => Ok(PROMPT_FASE_5.to_string()),
                f => Err(McpError::invalid_params(
                    format!("fase desconhecida: '{f}'"),
                    None,
                )),
            }
        }
        name => Err(McpError::invalid_params(
            format!("prompt desconhecido: {name}"),
            None,
        )),
    }
}

fn detectar_fase(path: &Path, has_staged: bool) -> (&'static str, &'static str) {
    if path.join("COMPAT.md").exists() {
        (
            "5 — Consolidação",
            "COMPAT.md presente — consolidar aprendizados e atualizar docs",
        )
    } else if path.join("CHANGELOG.md").exists()
        && std::fs::read_to_string(path.join("CHANGELOG.md"))
            .unwrap_or_default()
            .contains("Unreleased")
    {
        (
            "4 — Entrega",
            "CHANGELOG com Unreleased — preparar release notes e tag",
        )
    } else if has_staged {
        (
            "3 — Garantia",
            "Há arquivos staged — abrir PR com checklists",
        )
    } else if path.join("src").exists() || path.join("lib").exists() {
        (
            "2 — Desenvolvimento",
            "Código presente — continue construindo",
        )
    } else {
        (
            "1 — Preparação",
            "Nenhum código ainda — comece pela história e pelo sujeito",
        )
    }
}

fn capiba_resources_list() -> Vec<Resource> {
    vec![
        make_resource(
            "capiba://principios",
            "8 Princípios Invioláveis",
            "Os princípios do Pacto Fundante com implicações técnicas para cada um",
        ),
        make_resource(
            "capiba://pacto",
            "Pacto Fundante da CAPIDATA",
            "Princípios invioláveis, governança e processo de contribuição",
        ),
        make_resource(
            "capiba://contributing",
            "CONTRIBUTING.md",
            "Processo de contribuição em 5 fases",
        ),
        make_resource(
            "capiba://decisoes",
            "Ledger de Decisões",
            "Registro público de todas as decisões da CAPIDATA",
        ),
    ]
}

fn capiba_prompts_list() -> Vec<Prompt> {
    vec![
        Prompt::new(
            "capiba-onboard",
            Some("Onboarding de novo contribuidor no ecossistema Capiba"),
            None,
        ),
        Prompt::new(
            "capiba-fase",
            Some("Guia de uma das 5 fases do processo de contribuição"),
            Some(vec![PromptArgument {
                name: "fase".into(),
                title: None,
                description: Some(
                    "preparacao | desenvolvimento | garantia | entrega | consolidacao".into(),
                ),
                required: Some(true),
            }]),
        ),
        Prompt::new(
            "capiba-revisar",
            Some("Revisão ética e técnica de código contra os princípios do Pacto Fundante"),
            None,
        ),
        Prompt::new(
            "capiba-pr",
            Some("Geração de descrição de PR com checklists técnico e ético"),
            None,
        ),
    ]
}

// ─── Tools ────────────────────────────────────────────────────────────────────

#[tool_router]
impl CapibaMcp {
    #[tool(
        description = "Verifica se um trecho de código respeita os princípios invioláveis do Pacto Fundante"
    )]
    async fn capiba_check_compat(&self, Parameters(p): Parameters<CheckCompatParams>) -> String {
        check_compat_logic(&p.code, p.language.as_deref().unwrap_or("desconhecida"))
    }

    #[tool(description = "Retorna um princípio inviolável do Pacto Fundante com explicação")]
    async fn capiba_get_principio(&self, Parameters(p): Parameters<GetPrincipioParams>) -> String {
        get_principio_logic(p.numero)
    }

    #[tool(description = "Recupera uma decisão formal do ledger público da CAPIDATA")]
    async fn capiba_get_decisao(&self, Parameters(p): Parameters<GetDecisaoParams>) -> String {
        find_decisao(&self.root.join(".github/decisoes"), p.id)
            .unwrap_or_else(|| format!("Decisão #{} não encontrada no ledger.", p.id))
    }

    #[tool(description = "Estrutura uma história de contribuição no formato padrão da CAPIDATA")]
    async fn capiba_gerar_historia(
        &self,
        Parameters(p): Parameters<GerarHistoriaParams>,
    ) -> String {
        gerar_historia_logic(&p.sujeito, &p.acao, &p.objetivo)
    }

    #[tool(
        description = "Detecta a fase atual do processo de contribuição baseado nos arquivos do worktree"
    )]
    async fn capiba_fase_atual(&self, Parameters(p): Parameters<FaseAtualParams>) -> String {
        let path = PathBuf::from(&p.worktree_path);
        let has_staged = path.join(".git").exists()
            && std::process::Command::new("git")
                .args(["-C", &p.worktree_path, "diff", "--cached", "--name-only"])
                .output()
                .map(|o| !o.stdout.is_empty())
                .unwrap_or(false);
        let (fase, proximo) = detectar_fase(&path, has_staged);
        format!("**Fase detectada:** {fase}\n\n**Próximo passo:** {proximo}")
    }
}

// ─── ServerHandler ────────────────────────────────────────────────────────────

#[tool_handler]
impl ServerHandler for CapibaMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Servidor MCP do protocolo Capiba — expõe tools e resources para o ecossistema CAPIDATA."
                    .into(),
            ),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .enable_prompts()
                .build(),
            server_info: Implementation {
                name: "capiba-mcp".into(),
                version: "0.1.0".into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<rmcp::RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        Ok(ListResourcesResult::with_all_items(capiba_resources_list()))
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<rmcp::RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        let text = self.read_resource_text(request.uri.as_str())?;
        Ok(ReadResourceResult {
            contents: vec![ResourceContents::TextResourceContents {
                uri: request.uri,
                mime_type: Some("text/markdown".into()),
                text,
                meta: None,
            }],
        })
    }

    async fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<rmcp::RoleServer>,
    ) -> Result<ListPromptsResult, McpError> {
        Ok(ListPromptsResult::with_all_items(capiba_prompts_list()))
    }

    async fn get_prompt(
        &self,
        request: GetPromptRequestParams,
        _context: RequestContext<rmcp::RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        let fase = request
            .arguments
            .as_ref()
            .and_then(|a| a.get("fase"))
            .and_then(Value::as_str);
        let texto = get_prompt_texto(request.name.as_str(), fase)?;
        Ok(GetPromptResult {
            description: None,
            messages: vec![PromptMessage::new_text(PromptMessageRole::User, texto)],
        })
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn make_resource(uri: &str, name: &str, description: &str) -> Resource {
    Annotated::new(
        RawResource {
            uri: uri.into(),
            name: name.into(),
            title: None,
            description: Some(description.into()),
            mime_type: Some("text/markdown".into()),
            size: None,
            icons: None,
            meta: None,
        },
        None,
    )
}

/// Busca recursivamente uma decisão pelo número em qualquer subdiretório de `dir`.
fn find_decisao(dir: &Path, id: u32) -> Option<String> {
    let entries = std::fs::read_dir(dir).ok()?;
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if path.is_dir() {
            if let Some(found) = find_decisao(&path, id) {
                return Some(found);
            }
        } else if name.starts_with(&format!("{:04}-", id)) {
            return std::fs::read_to_string(&path).ok();
        }
    }
    None
}

// ─── Main ─────────────────────────────────────────────────────────────────────
// LCOV_EXCL_START — não testável sem servidor MCP ativo

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("capiba_mcp=info")),
        )
        .init();

    let service = CapibaMcp::new()
        .serve(stdio())
        .await
        .inspect_err(|e| tracing::error!("erro ao iniciar servidor: {e}"))?;

    service.waiting().await?;
    Ok(())
}
// LCOV_EXCL_STOP

// ─── Testes ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn tmp(suffix: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("capiba_{suffix}"));
        fs::create_dir_all(&d).unwrap();
        d
    }

    // ── PRINCIPIOS ────────────────────────────────────────────────────────────

    #[test]
    fn principios_sao_oito() {
        assert_eq!(PRINCIPIOS.len(), 8);
    }

    #[test]
    fn principio_i_contem_soberania() {
        assert!(PRINCIPIOS[0].contains("Soberania"));
    }

    #[test]
    fn principio_viii_contem_autocorrecao() {
        assert!(PRINCIPIOS[7].contains("Autocorreção"));
    }

    // ── get_principio_logic ───────────────────────────────────────────────────

    #[test]
    fn get_principio_1_retorna_soberania() {
        assert!(get_principio_logic(1).contains("Soberania"));
    }

    #[test]
    fn get_principio_8_retorna_autocorrecao() {
        assert!(get_principio_logic(8).contains("Autocorreção"));
    }

    #[test]
    fn get_principio_0_retorna_primeiro_por_saturating_sub() {
        // saturating_sub(1) de 0u8 == 0 → PRINCIPIOS[0]
        assert!(get_principio_logic(0).contains("Soberania"));
    }

    #[test]
    fn get_principio_9_retorna_mensagem_de_erro() {
        assert!(get_principio_logic(9).contains("não encontrado"));
    }

    // ── check_compat_logic ────────────────────────────────────────────────────

    #[test]
    fn compat_sem_alertas() {
        assert!(
            check_compat_logic("fn soma(a: i32, b: i32) -> i32 { a + b }", "rust").contains("🟢")
        );
    }

    #[test]
    fn compat_detecta_process_exit() {
        assert!(check_compat_logic("process::exit(1);", "rust").contains("process::exit"));
    }

    #[test]
    fn compat_detecta_std_process_exit() {
        assert!(check_compat_logic("std::process::exit(0);", "rust").contains("process::exit"));
    }

    #[test]
    fn compat_detecta_unwrap_rust() {
        assert!(check_compat_logic("foo.unwrap()", "rust").contains("unwrap()"));
    }

    #[test]
    fn compat_unwrap_ignorado_em_outras_linguas() {
        assert!(check_compat_logic("x.unwrap()", "python").contains("🟢"));
    }

    #[test]
    fn compat_detecta_http() {
        assert!(
            check_compat_logic("fetch(\"http://api.example.com\")", "typescript")
                .contains("HTTP não cifrada")
        );
    }

    #[test]
    fn compat_detecta_log_de_senha() {
        assert!(check_compat_logic("println!(\"senha: {}\", s)", "rust").contains("vazamento"));
    }

    #[test]
    fn compat_multiplos_alertas_geram_cabecalho() {
        let r = check_compat_logic("http://x; println!(\"senha\"); unwrap()", "rust");
        assert!(r.contains("## Alertas"));
    }

    // ── gerar_historia_logic ──────────────────────────────────────────────────

    #[test]
    fn historia_contem_sujeito_acao_objetivo() {
        let r = gerar_historia_logic("pescador", "registrar produção", "ter crédito");
        assert!(
            r.contains("pescador") && r.contains("registrar produção") && r.contains("ter crédito")
        );
    }

    #[test]
    fn historia_tem_estrutura_padrao() {
        let r = gerar_historia_logic("x", "y", "z");
        assert!(r.contains("Critérios de aceitação"));
        assert!(r.contains("Cenários de uso"));
        assert!(r.contains("δ efetivo"));
    }

    // ── principios_texto ──────────────────────────────────────────────────────

    #[test]
    fn principios_texto_tem_todos_os_oito() {
        let t = principios_texto();
        for i in 1..=8 {
            assert!(t.contains(&format!("## Princípio {i}")));
        }
    }

    // ── get_prompt_texto ──────────────────────────────────────────────────────

    #[test]
    fn prompt_onboard_ok() {
        assert!(get_prompt_texto("capiba-onboard", None).is_ok());
    }

    #[test]
    fn prompt_revisar_ok() {
        assert!(get_prompt_texto("capiba-revisar", None).is_ok());
    }

    #[test]
    fn prompt_pr_ok() {
        assert!(get_prompt_texto("capiba-pr", None).is_ok());
    }

    #[test]
    fn prompt_fase_todas_as_fases() {
        for fase in [
            "preparacao",
            "desenvolvimento",
            "garantia",
            "entrega",
            "consolidacao",
        ] {
            assert!(
                get_prompt_texto("capiba-fase", Some(fase)).is_ok(),
                "falhou: {fase}"
            );
        }
    }

    #[test]
    fn prompt_fase_sem_arg_usa_preparacao() {
        assert!(get_prompt_texto("capiba-fase", None).is_ok());
    }

    #[test]
    fn prompt_fase_invalida_retorna_erro() {
        assert!(get_prompt_texto("capiba-fase", Some("inexistente")).is_err());
    }

    #[test]
    fn prompt_desconhecido_retorna_erro() {
        assert!(get_prompt_texto("capiba-xyz", None).is_err());
    }

    // ── detectar_fase ─────────────────────────────────────────────────────────

    #[test]
    fn fase_5_com_compat_md() {
        let d = tmp("fase5");
        fs::write(d.join("COMPAT.md"), "").unwrap();
        assert!(detectar_fase(&d, false).0.contains("5"));
        fs::remove_dir_all(&d).unwrap();
    }

    #[test]
    fn fase_4_com_changelog_unreleased() {
        let d = tmp("fase4");
        fs::write(d.join("CHANGELOG.md"), "## Unreleased\n").unwrap();
        assert!(detectar_fase(&d, false).0.contains("4"));
        fs::remove_dir_all(&d).unwrap();
    }

    #[test]
    fn fase_3_com_staged() {
        let d = tmp("fase3");
        assert!(detectar_fase(&d, true).0.contains("3"));
        fs::remove_dir_all(&d).unwrap();
    }

    #[test]
    fn fase_2_com_src() {
        let d = tmp("fase2src");
        fs::create_dir_all(d.join("src")).unwrap();
        assert!(detectar_fase(&d, false).0.contains("2"));
        fs::remove_dir_all(&d).unwrap();
    }

    #[test]
    fn fase_2_com_lib() {
        let d = tmp("fase2lib");
        fs::create_dir_all(d.join("lib")).unwrap();
        assert!(detectar_fase(&d, false).0.contains("2"));
        fs::remove_dir_all(&d).unwrap();
    }

    #[test]
    fn fase_1_sem_nada() {
        let d = tmp("fase1");
        assert!(detectar_fase(&d, false).0.contains("1"));
        fs::remove_dir_all(&d).unwrap();
    }

    #[test]
    fn changelog_sem_unreleased_nao_e_fase_4() {
        let d = tmp("changelog_sem_unreleased");
        fs::write(d.join("CHANGELOG.md"), "## v1.0.0\n").unwrap();
        assert!(!detectar_fase(&d, false).0.contains("4"));
        fs::remove_dir_all(&d).unwrap();
    }

    // ── capiba_resources_list ─────────────────────────────────────────────────

    #[test]
    fn resources_list_tem_quatro_itens() {
        assert_eq!(capiba_resources_list().len(), 4);
    }

    #[test]
    fn resources_list_contem_principios() {
        let uris: Vec<String> = capiba_resources_list()
            .into_iter()
            .map(|r| r.raw.uri.to_string())
            .collect();
        assert!(uris.contains(&"capiba://principios".to_string()));
        assert!(uris.contains(&"capiba://pacto".to_string()));
        assert!(uris.contains(&"capiba://contributing".to_string()));
        assert!(uris.contains(&"capiba://decisoes".to_string()));
    }

    // ── capiba_prompts_list ───────────────────────────────────────────────────

    #[test]
    fn prompts_list_tem_quatro_itens() {
        assert_eq!(capiba_prompts_list().len(), 4);
    }

    #[test]
    fn prompts_list_contem_onboard() {
        let nomes: Vec<String> = capiba_prompts_list()
            .into_iter()
            .map(|p| p.name.to_string())
            .collect();
        assert!(nomes.contains(&"capiba-onboard".to_string()));
        assert!(nomes.contains(&"capiba-fase".to_string()));
    }

    // ── make_resource ─────────────────────────────────────────────────────────

    #[test]
    fn make_resource_popula_campos() {
        let r = make_resource("capiba://test", "Nome", "Desc");
        assert_eq!(r.raw.uri, "capiba://test");
        assert_eq!(r.raw.name, "Nome");
        assert_eq!(r.raw.description.as_deref(), Some("Desc"));
    }

    // ── CapibaMcp::with_root + read_file ──────────────────────────────────────

    #[test]
    fn with_root_define_root_corretamente() {
        let d = tmp("with_root");
        let srv = CapibaMcp::with_root(d.clone());
        assert_eq!(srv.root, d);
        fs::remove_dir_all(&d).unwrap();
    }

    #[test]
    fn read_file_existente_retorna_conteudo() {
        let d = tmp("read_file");
        fs::write(d.join("arq.md"), "texto").unwrap();
        let srv = CapibaMcp::with_root(d.clone());
        assert_eq!(srv.read_file("arq.md").unwrap(), "texto");
        fs::remove_dir_all(&d).unwrap();
    }

    #[test]
    fn read_file_inexistente_retorna_erro() {
        let srv = CapibaMcp::with_root(PathBuf::from("/tmp/__capiba_nao_existe__"));
        assert!(srv.read_file("nao-existe.md").is_err());
    }

    // ── read_resource_text ────────────────────────────────────────────────────

    #[test]
    fn resource_principios_nao_precisa_de_filesystem() {
        let srv = CapibaMcp::with_root(PathBuf::from("/tmp"));
        let r = srv.read_resource_text("capiba://principios").unwrap();
        assert!(r.contains("Princípio 1"));
    }

    #[test]
    fn resource_desconhecido_retorna_erro() {
        let srv = CapibaMcp::with_root(PathBuf::from("/tmp"));
        assert!(srv.read_resource_text("capiba://inexistente").is_err());
    }

    #[test]
    fn resource_pacto_com_arquivo() {
        let d = tmp("resource_pacto");
        let estatuto = d.join(".github/estatuto");
        fs::create_dir_all(&estatuto).unwrap();
        fs::write(estatuto.join("pacto-fundante-v0.1.md"), "# Pacto").unwrap();
        let srv = CapibaMcp::with_root(d.clone());
        assert!(srv
            .read_resource_text("capiba://pacto")
            .unwrap()
            .contains("Pacto"));
        fs::remove_dir_all(&d).unwrap();
    }

    #[test]
    fn resource_contributing_com_arquivo() {
        let d = tmp("resource_contrib");
        fs::write(d.join("CONTRIBUTING.md"), "# Contribuindo").unwrap();
        let srv = CapibaMcp::with_root(d.clone());
        assert!(srv.read_resource_text("capiba://contributing").is_ok());
        fs::remove_dir_all(&d).unwrap();
    }

    #[test]
    fn resource_decisoes_com_arquivo() {
        let d = tmp("resource_decisoes");
        let decisoes = d.join(".github/decisoes");
        fs::create_dir_all(&decisoes).unwrap();
        fs::write(decisoes.join("README.md"), "# Decisões").unwrap();
        let srv = CapibaMcp::with_root(d.clone());
        assert!(srv.read_resource_text("capiba://decisoes").is_ok());
        fs::remove_dir_all(&d).unwrap();
    }

    // ── get_info ──────────────────────────────────────────────────────────────

    #[test]
    fn get_info_tem_nome_e_versao_corretos() {
        let srv = CapibaMcp::with_root(PathBuf::from("/tmp"));
        let info = srv.get_info();
        assert_eq!(info.server_info.name, "capiba-mcp");
        assert_eq!(info.server_info.version, "0.1.0");
    }

    #[test]
    fn get_info_tem_instrucoes() {
        let srv = CapibaMcp::with_root(PathBuf::from("/tmp"));
        assert!(srv.get_info().instructions.is_some());
    }

    // ── find_decisao ──────────────────────────────────────────────────────────

    #[test]
    fn find_decisao_retorna_none_em_pasta_inexistente() {
        assert!(find_decisao(Path::new("/tmp/__capiba_nao_existe__"), 1).is_none());
    }

    #[test]
    fn find_decisao_encontra_em_subdiretorio() {
        let d = tmp("find_decisao");
        let sub = d.join("2026");
        fs::create_dir_all(&sub).unwrap();
        fs::write(sub.join("0001-teste.md"), "# Decisão").unwrap();
        let r = find_decisao(&d, 1);
        assert!(r.is_some() && r.unwrap().contains("Decisão"));
        fs::remove_dir_all(&d).unwrap();
    }

    #[test]
    fn find_decisao_nao_encontra_id_inexistente() {
        let d = tmp("find_decisao_errado");
        let sub = d.join("2026");
        fs::create_dir_all(&sub).unwrap();
        fs::write(sub.join("0001-teste.md"), "conteúdo").unwrap();
        assert!(find_decisao(&d, 99).is_none());
        fs::remove_dir_all(&d).unwrap();
    }

    // ── CapibaMcp::new ────────────────────────────────────────────────────────

    #[test]
    fn new_usa_capiba_root_quando_definido() {
        let d = tmp("new_env");
        std::env::set_var("CAPIBA_ROOT", d.to_str().unwrap());
        let srv = CapibaMcp::new();
        assert_eq!(srv.root, d);
        std::env::remove_var("CAPIBA_ROOT");
        fs::remove_dir_all(&d).unwrap();
    }

    // ── handler capiba_fase_atual com .git real ───────────────────────────────

    #[tokio::test]
    async fn handler_fase_com_git_inicializado() {
        let d = tmp("handler_fase_git");
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(&d)
            .status()
            .expect("git deve estar disponível");
        let srv = CapibaMcp::with_root(PathBuf::from("/tmp"));
        let r = srv
            .capiba_fase_atual(Parameters(FaseAtualParams {
                worktree_path: d.to_str().unwrap().into(),
            }))
            .await;
        assert!(r.contains("Fase detectada"));
        fs::remove_dir_all(&d).unwrap();
    }

    // ── handlers async ────────────────────────────────────────────────────────

    #[tokio::test]
    async fn handler_check_compat_sem_alertas() {
        let srv = CapibaMcp::with_root(PathBuf::from("/tmp"));
        let r = srv
            .capiba_check_compat(Parameters(CheckCompatParams {
                code: "fn ok() {}".into(),
                language: Some("rust".into()),
            }))
            .await;
        assert!(r.contains("🟢"));
    }

    #[tokio::test]
    async fn handler_check_compat_com_alerta() {
        let srv = CapibaMcp::with_root(PathBuf::from("/tmp"));
        let r = srv
            .capiba_check_compat(Parameters(CheckCompatParams {
                code: "fetch(\"http://x\")".into(),
                language: None,
            }))
            .await;
        assert!(r.contains("HTTP"));
    }

    #[tokio::test]
    async fn handler_get_principio_1() {
        let srv = CapibaMcp::with_root(PathBuf::from("/tmp"));
        let r = srv
            .capiba_get_principio(Parameters(GetPrincipioParams { numero: 1 }))
            .await;
        assert!(r.contains("Soberania"));
    }

    #[tokio::test]
    async fn handler_get_principio_invalido() {
        let srv = CapibaMcp::with_root(PathBuf::from("/tmp"));
        let r = srv
            .capiba_get_principio(Parameters(GetPrincipioParams { numero: 99 }))
            .await;
        assert!(r.contains("não encontrado"));
    }

    #[tokio::test]
    async fn handler_gerar_historia() {
        let srv = CapibaMcp::with_root(PathBuf::from("/tmp"));
        let r = srv
            .capiba_gerar_historia(Parameters(GerarHistoriaParams {
                sujeito: "pescador".into(),
                acao: "registrar".into(),
                objetivo: "crédito".into(),
            }))
            .await;
        assert!(r.contains("pescador"));
    }

    #[tokio::test]
    async fn handler_get_decisao_nao_encontrada() {
        let srv = CapibaMcp::with_root(PathBuf::from("/tmp/__capiba_vazio__"));
        let r = srv
            .capiba_get_decisao(Parameters(GetDecisaoParams { id: 999 }))
            .await;
        assert!(r.contains("não encontrada"));
    }

    #[tokio::test]
    async fn handler_get_decisao_encontrada() {
        let d = tmp("handler_decisao");
        let sub = d.join(".github/decisoes/2026");
        fs::create_dir_all(&sub).unwrap();
        fs::write(sub.join("0042-teste.md"), "# Decisão 42").unwrap();
        let srv = CapibaMcp::with_root(d.clone());
        let r = srv
            .capiba_get_decisao(Parameters(GetDecisaoParams { id: 42 }))
            .await;
        assert!(r.contains("Decisão 42"));
        fs::remove_dir_all(&d).unwrap();
    }

    #[tokio::test]
    async fn handler_fase_atual_fase_1() {
        let d = tmp("handler_fase1");
        let srv = CapibaMcp::with_root(PathBuf::from("/tmp"));
        let r = srv
            .capiba_fase_atual(Parameters(FaseAtualParams {
                worktree_path: d.to_str().unwrap().into(),
            }))
            .await;
        assert!(r.contains("Fase detectada"));
        fs::remove_dir_all(&d).unwrap();
    }
}
