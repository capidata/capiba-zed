// ─── Servidor ─────────────────────────────────────────────────────────────────

use crate::prompts::{capiba_prompts_list, get_prompt_texto, principios_texto};
use crate::resources::{capiba_resources_list, detectar_fase};
use crate::tools::{
    check_compat_logic, find_decisao, gerar_historia_logic, get_principio_logic, CheckCompatParams,
    FaseAtualParams, GerarHistoriaParams, GetDecisaoParams, GetPrincipioParams,
};
use rmcp::serde_json::Value;
use rmcp::{
    handler::server::router::tool::ToolRouter, handler::server::wrapper::Parameters, model::*,
    service::RequestContext, tool, tool_handler, tool_router, ErrorData as McpError, ServerHandler,
};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct CapibaMcp {
    pub root: PathBuf,
    pub tool_router: ToolRouter<Self>,
}

impl CapibaMcp {
    pub fn new() -> Self {
        let root = std::env::var("CAPIBA_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());
        Self::with_root(root)
    }

    pub fn with_root(root: PathBuf) -> Self {
        Self {
            root,
            tool_router: Self::tool_router(),
        }
    }

    pub fn read_file(&self, rel: &str) -> Result<String, McpError> {
        let path = self.root.join(rel);
        std::fs::read_to_string(&path).map_err(|_| {
            McpError::internal_error(format!("arquivo não encontrado: {}", path.display()), None)
        })
    }

    pub fn read_worktree_context(&self) -> String {
        let mut partes = Vec::new();
        if let Ok(text) = std::fs::read_to_string(self.root.join("CLAUDE.md")) {
            if !text.trim().is_empty() {
                partes.push(format!("# Contexto do Projeto (CLAUDE.md)\n\n{text}"));
            }
        }
        if let Ok(text) = std::fs::read_to_string(self.root.join("CONTRIBUTING.md")) {
            if !text.trim().is_empty() {
                partes.push(format!("# CONTRIBUTING\n\n{text}"));
            }
        }
        partes.join("\n\n---\n\n")
    }

    pub fn read_resource_text(&self, uri: &str) -> Result<String, McpError> {
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

// ─── Tools ────────────────────────────────────────────────────────────────────

#[tool_router]
impl CapibaMcp {
    #[tool(
        description = "Verifica se um trecho de código respeita os princípios invioláveis do Pacto Fundante"
    )]
    pub async fn capiba_check_compat(
        &self,
        Parameters(p): Parameters<CheckCompatParams>,
    ) -> String {
        check_compat_logic(&p.code, p.language.as_deref().unwrap_or("desconhecida"))
    }

    #[tool(description = "Retorna um princípio inviolável do Pacto Fundante com explicação")]
    pub async fn capiba_get_principio(
        &self,
        Parameters(p): Parameters<GetPrincipioParams>,
    ) -> String {
        get_principio_logic(p.numero)
    }

    #[tool(description = "Recupera uma decisão formal do ledger público da CAPIDATA")]
    pub async fn capiba_get_decisao(&self, Parameters(p): Parameters<GetDecisaoParams>) -> String {
        find_decisao(&self.root.join(".github/decisoes"), p.id)
            .unwrap_or_else(|| format!("Decisão #{} não encontrada no ledger.", p.id))
    }

    #[tool(description = "Estrutura uma história de contribuição no formato padrão da CAPIDATA")]
    pub async fn capiba_gerar_historia(
        &self,
        Parameters(p): Parameters<GerarHistoriaParams>,
    ) -> String {
        gerar_historia_logic(&p.sujeito, &p.acao, &p.objetivo)
    }

    #[tool(
        description = "Detecta a fase atual do processo de contribuição baseado nos arquivos do worktree"
    )]
    pub async fn capiba_fase_atual(&self, Parameters(p): Parameters<FaseAtualParams>) -> String {
        let path = PathBuf::from(&p.worktree_path);
        let has_staged = if path.join(".git").exists() {
            // Use timeout-safe check: existência de staged files via git
            tokio::task::spawn_blocking({
                let worktree = p.worktree_path.clone();
                move || {
                    std::process::Command::new("git")
                        .args(["-C", &worktree, "diff", "--cached", "--name-only"])
                        .output()
                        .map(|o| !o.stdout.is_empty())
                        .unwrap_or(false)
                }
            })
            .await
            .unwrap_or(false)
        } else {
            false
        };
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
                version: "0.1.1".into(),
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

        // Contexto do worktree (CLAUDE.md + CONTRIBUTING.md), se disponível
        let contexto = self.read_worktree_context();
        let texto_final = if contexto.is_empty() {
            texto
        } else {
            format!("{contexto}\n\n---\n\n{texto}")
        };

        Ok(GetPromptResult {
            description: None,
            messages: vec![PromptMessage::new_text(
                PromptMessageRole::User,
                texto_final,
            )],
        })
    }
}

// ─── Testes ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::{
        CheckCompatParams, FaseAtualParams, GerarHistoriaParams, GetDecisaoParams,
        GetPrincipioParams,
    };
    use rmcp::handler::server::wrapper::Parameters;
    use std::fs;

    fn tmp(suffix: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("capiba_{suffix}"));
        fs::create_dir_all(&d).unwrap();
        d
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
        assert_eq!(info.server_info.version, "0.1.1");
    }

    #[test]
    fn get_info_tem_instrucoes() {
        let srv = CapibaMcp::with_root(PathBuf::from("/tmp"));
        assert!(srv.get_info().instructions.is_some());
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
