// ─── Tools ────────────────────────────────────────────────────────────────────

use capiba_prompts::PRINCIPIOS;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ─── Parâmetros das tools ─────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct CheckCompatParams {
    pub code: String,
    pub language: Option<String>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct GetPrincipioParams {
    pub numero: u8,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct GetDecisaoParams {
    pub id: u32,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct GerarHistoriaParams {
    pub sujeito: String,
    pub acao: String,
    pub objetivo: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct FaseAtualParams {
    pub worktree_path: String,
}

// ─── Lógica pura ──────────────────────────────────────────────────────────────

pub fn check_compat_logic(code: &str, lang: &str) -> String {
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

pub fn get_principio_logic(numero: u8) -> String {
    PRINCIPIOS
        .get((numero as usize).saturating_sub(1))
        .copied()
        .unwrap_or("Princípio não encontrado. Informe um número entre 1 e 8.")
        .to_string()
}

pub fn gerar_historia_logic(sujeito: &str, acao: &str, objetivo: &str) -> String {
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

/// Busca recursivamente uma decisão pelo número em qualquer subdiretório de `dir`.
pub fn find_decisao(dir: &std::path::Path, id: u32) -> Option<String> {
    find_decisao_recurse(dir, id, 0)
}

fn find_decisao_recurse(dir: &std::path::Path, id: u32, depth: u32) -> Option<String> {
    const MAX_DEPTH: u32 = 10;
    if depth > MAX_DEPTH {
        return None; // Proteção contra recursão infinita (symlinks, etc)
    }

    let entries = std::fs::read_dir(dir).ok()?;
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if path.is_dir() {
            if let Some(found) = find_decisao_recurse(&path, id, depth + 1) {
                return Some(found);
            }
        } else if name.starts_with(&format!("{:04}-", id)) {
            return std::fs::read_to_string(&path).ok();
        }
    }
    None
}

// ─── Testes ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::{Path, PathBuf};

    fn tmp(suffix: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("capiba_{suffix}"));
        fs::create_dir_all(&d).unwrap();
        d
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
}
