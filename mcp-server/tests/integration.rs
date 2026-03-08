//! Testes de integração do capiba-mcp via protocolo MCP
//!
//! Cobrem os handlers ServerHandler (list_resources, read_resource,
//! list_prompts, get_prompt, call_tool) que requerem RequestContext<rmcp::RoleServer>
//! e não podem ser testados em unit tests.

use rmcp::{
    model::{CallToolRequestParams, GetPromptRequestParams, ReadResourceRequestParams},
    service::RunningService,
    transport::child_process::TokioChildProcess,
    RoleClient, ServiceExt,
};
use std::{fs, path::Path};

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Cria um diretório temporário com a estrutura mínima que o servidor precisa.
fn setup_root(suffix: &str) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!("capiba_integ_{suffix}"));
    let estatuto = root.join(".github/estatuto");
    let decisoes = root.join(".github/decisoes/2026");
    fs::create_dir_all(&estatuto).unwrap();
    fs::create_dir_all(&decisoes).unwrap();
    fs::write(estatuto.join("pacto-fundante-v0.1.md"), "# Pacto Fundante").unwrap();
    fs::write(root.join("CONTRIBUTING.md"), "# Contribuindo").unwrap();
    fs::write(root.join(".github/decisoes/README.md"), "# Decisões").unwrap();
    fs::write(
        decisoes.join("0001-pacto-fundante.md"),
        "# Decisão 0001 — Pacto Fundante",
    )
    .unwrap();
    root
}

async fn start_server(root: &Path) -> RunningService<RoleClient, ()> {
    let binary = env!("CARGO_BIN_EXE_capiba-mcp");
    let mut cmd = tokio::process::Command::new(binary);
    cmd.env("CAPIBA_ROOT", root);
    let transport = TokioChildProcess::new(cmd).unwrap();
    ().serve(transport).await.unwrap()
}

fn text_of(result: &rmcp::model::CallToolResult) -> &str {
    result.content[0].raw.as_text().unwrap().text.as_str()
}

// ── list_resources ────────────────────────────────────────────────────────────

#[tokio::test]
async fn integ_list_resources_retorna_quatro() {
    let root = setup_root("list_resources");
    let svc = start_server(&root).await;
    let result = svc.peer().list_resources(None).await.unwrap();
    assert_eq!(result.resources.len(), 4);
    fs::remove_dir_all(&root).unwrap();
}

#[tokio::test]
async fn integ_list_resources_uris_corretas() {
    let root = setup_root("list_res_uris");
    let svc = start_server(&root).await;
    let result = svc.peer().list_resources(None).await.unwrap();
    let uris: Vec<&str> = result
        .resources
        .iter()
        .map(|r| r.raw.uri.as_str())
        .collect();
    assert!(uris.contains(&"capiba://principios"));
    assert!(uris.contains(&"capiba://pacto"));
    assert!(uris.contains(&"capiba://contributing"));
    assert!(uris.contains(&"capiba://decisoes"));
    fs::remove_dir_all(&root).unwrap();
}

// ── read_resource ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn integ_read_resource_principios_embutido() {
    let root = setup_root("read_principios");
    let svc = start_server(&root).await;
    let result = svc
        .peer()
        .read_resource(ReadResourceRequestParams {
            meta: None,
            uri: "capiba://principios".into(),
        })
        .await
        .unwrap();
    let text = match &result.contents[0] {
        rmcp::model::ResourceContents::TextResourceContents { text, .. } => text.as_str(),
        _ => panic!("esperado TextResourceContents"),
    };
    assert!(text.contains("Princípio 1"));
    fs::remove_dir_all(&root).unwrap();
}

#[tokio::test]
async fn integ_read_resource_pacto_do_filesystem() {
    let root = setup_root("read_pacto");
    let svc = start_server(&root).await;
    let result = svc
        .peer()
        .read_resource(ReadResourceRequestParams {
            meta: None,
            uri: "capiba://pacto".into(),
        })
        .await
        .unwrap();
    let text = match &result.contents[0] {
        rmcp::model::ResourceContents::TextResourceContents { text, .. } => text.as_str(),
        _ => panic!("esperado TextResourceContents"),
    };
    assert!(text.contains("Pacto Fundante"));
    fs::remove_dir_all(&root).unwrap();
}

#[tokio::test]
async fn integ_read_resource_uri_desconhecida_retorna_erro() {
    let root = setup_root("read_unkn");
    let svc = start_server(&root).await;
    let result = svc
        .peer()
        .read_resource(ReadResourceRequestParams {
            meta: None,
            uri: "capiba://nao-existe".into(),
        })
        .await;
    assert!(result.is_err());
    fs::remove_dir_all(&root).unwrap();
}

// ── list_prompts ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn integ_list_prompts_retorna_quatro() {
    let root = setup_root("list_prompts");
    let svc = start_server(&root).await;
    let result = svc.peer().list_prompts(None).await.unwrap();
    assert_eq!(result.prompts.len(), 4);
    fs::remove_dir_all(&root).unwrap();
}

#[tokio::test]
async fn integ_list_prompts_contem_onboard_e_fase() {
    let root = setup_root("list_prompts_nomes");
    let svc = start_server(&root).await;
    let result = svc.peer().list_prompts(None).await.unwrap();
    let names: Vec<&str> = result.prompts.iter().map(|p| p.name.as_str()).collect();
    assert!(names.contains(&"capiba-onboard"));
    assert!(names.contains(&"capiba-fase"));
    fs::remove_dir_all(&root).unwrap();
}

// ── get_prompt ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn integ_get_prompt_onboard() {
    let root = setup_root("get_onboard");
    let svc = start_server(&root).await;
    let result = svc
        .peer()
        .get_prompt(GetPromptRequestParams {
            meta: None,
            name: "capiba-onboard".into(),
            arguments: None,
        })
        .await
        .unwrap();
    assert!(!result.messages.is_empty());
    match &result.messages[0].content {
        rmcp::model::PromptMessageContent::Text { text } => {
            assert!(!text.is_empty());
        }
        _ => panic!("esperado PromptMessageContent::Text"),
    }
    fs::remove_dir_all(&root).unwrap();
}

#[tokio::test]
async fn integ_get_prompt_fase_preparacao() {
    let root = setup_root("get_fase_prep");
    let svc = start_server(&root).await;
    let result = svc
        .peer()
        .get_prompt(GetPromptRequestParams {
            meta: None,
            name: "capiba-fase".into(),
            arguments: Some(
                rmcp::serde_json::json!({"fase": "preparacao"})
                    .as_object()
                    .cloned()
                    .unwrap(),
            ),
        })
        .await
        .unwrap();
    assert!(!result.messages.is_empty());
    fs::remove_dir_all(&root).unwrap();
}

#[tokio::test]
async fn integ_get_prompt_desconhecido_retorna_erro() {
    let root = setup_root("get_prompt_err");
    let svc = start_server(&root).await;
    let result = svc
        .peer()
        .get_prompt(GetPromptRequestParams {
            meta: None,
            name: "capiba-inexistente".into(),
            arguments: None,
        })
        .await;
    assert!(result.is_err());
    fs::remove_dir_all(&root).unwrap();
}

// ── call_tool ─────────────────────────────────────────────────────────────────

#[tokio::test]
async fn integ_tool_check_compat_sem_alertas() {
    let root = setup_root("tool_compat_ok");
    let svc = start_server(&root).await;
    let result = svc
        .peer()
        .call_tool(CallToolRequestParams {
            meta: None,
            name: "capiba_check_compat".into(),
            arguments: Some(
                rmcp::serde_json::json!({"code": "fn ok() {}", "language": "rust"})
                    .as_object()
                    .cloned()
                    .unwrap(),
            ),
            task: None,
        })
        .await
        .unwrap();
    assert!(text_of(&result).contains("🟢"));
    fs::remove_dir_all(&root).unwrap();
}

#[tokio::test]
async fn integ_tool_check_compat_com_alerta_http() {
    let root = setup_root("tool_compat_http");
    let svc = start_server(&root).await;
    let result = svc
        .peer()
        .call_tool(CallToolRequestParams {
            meta: None,
            name: "capiba_check_compat".into(),
            arguments: Some(
                rmcp::serde_json::json!({"code": "fetch(\"http://api.example.com\")"})
                    .as_object()
                    .cloned()
                    .unwrap(),
            ),
            task: None,
        })
        .await
        .unwrap();
    assert!(text_of(&result).contains("HTTP"));
    fs::remove_dir_all(&root).unwrap();
}

#[tokio::test]
async fn integ_tool_get_principio_i() {
    let root = setup_root("tool_principio");
    let svc = start_server(&root).await;
    let result = svc
        .peer()
        .call_tool(CallToolRequestParams {
            meta: None,
            name: "capiba_get_principio".into(),
            arguments: Some(
                rmcp::serde_json::json!({"numero": 1})
                    .as_object()
                    .cloned()
                    .unwrap(),
            ),
            task: None,
        })
        .await
        .unwrap();
    assert!(text_of(&result).contains("Soberania"));
    fs::remove_dir_all(&root).unwrap();
}

#[tokio::test]
async fn integ_tool_get_decisao_encontrada() {
    let root = setup_root("tool_decisao");
    let svc = start_server(&root).await;
    let result = svc
        .peer()
        .call_tool(CallToolRequestParams {
            meta: None,
            name: "capiba_get_decisao".into(),
            arguments: Some(
                rmcp::serde_json::json!({"id": 1})
                    .as_object()
                    .cloned()
                    .unwrap(),
            ),
            task: None,
        })
        .await
        .unwrap();
    assert!(text_of(&result).contains("Decisão 0001"));
    fs::remove_dir_all(&root).unwrap();
}

#[tokio::test]
async fn integ_tool_gerar_historia() {
    let root = setup_root("tool_historia");
    let svc = start_server(&root).await;
    let result = svc
        .peer()
        .call_tool(CallToolRequestParams {
            meta: None,
            name: "capiba_gerar_historia".into(),
            arguments: Some(
                rmcp::serde_json::json!({
                    "sujeito": "pescador",
                    "acao": "registrar captura",
                    "objetivo": "obter crédito"
                })
                .as_object()
                .cloned()
                .unwrap(),
            ),
            task: None,
        })
        .await
        .unwrap();
    assert!(text_of(&result).contains("pescador"));
    fs::remove_dir_all(&root).unwrap();
}

#[tokio::test]
async fn integ_tool_fase_atual() {
    let root = setup_root("tool_fase");
    let root_str = root.to_string_lossy().to_string();
    let svc = start_server(&root).await;
    let result = svc
        .peer()
        .call_tool(CallToolRequestParams {
            meta: None,
            name: "capiba_fase_atual".into(),
            arguments: Some(
                rmcp::serde_json::json!({"worktree_path": &root_str})
                    .as_object()
                    .cloned()
                    .unwrap(),
            ),
            task: None,
        })
        .await
        .unwrap();
    assert!(text_of(&result).contains("Fase detectada"));
    fs::remove_dir_all(&root).unwrap();
}
