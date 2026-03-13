// ─── Flags ────────────────────────────────────────────────────────────────────

/// Verifica se há flags de help ou version e executa a ação correspondente.
/// Retorna `Some(())` se uma flag foi encontrada e processada, `None` caso contrário.
pub fn check_help_version_flags(args: &[String]) -> Option<()> {
    for arg in &args[1..] {
        match arg.as_str() {
            "--help" | "-h" => {
                eprintln!("capiba-mcp — Servidor MCP do protocolo Capiba");
                eprintln!();
                eprintln!("Uso: capiba-mcp [OPÇÕES]");
                eprintln!();
                eprintln!("Opções:");
                eprintln!("  --help, -h       Mostrar esta mensagem de ajuda");
                eprintln!("  --version, -v    Mostrar versão");
                eprintln!();
                eprintln!("Variáveis de ambiente:");
                eprintln!("  CAPIBA_ROOT      Raiz do repositório (padrão: diretório atual)");
                eprintln!();
                eprintln!("O servidor executa via stdio usando o protocolo MCP v2024-11-05");
                eprintln!("Normalmente usado pela extensão Zed capiba-zed.");
                return Some(());
            }
            "--version" | "-v" => {
                eprintln!("capiba-mcp 0.1.1");
                return Some(());
            }
            _ => {}
        }
    }
    None
}

// ─── Testes ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags_help_longo_retorna_some() {
        let args = vec!["capiba-mcp".into(), "--help".into()];
        assert!(check_help_version_flags(&args).is_some());
    }

    #[test]
    fn flags_help_curto_retorna_some() {
        let args = vec!["capiba-mcp".into(), "-h".into()];
        assert!(check_help_version_flags(&args).is_some());
    }

    #[test]
    fn flags_version_longo_retorna_some() {
        let args = vec!["capiba-mcp".into(), "--version".into()];
        assert!(check_help_version_flags(&args).is_some());
    }

    #[test]
    fn flags_version_curto_retorna_some() {
        let args = vec!["capiba-mcp".into(), "-v".into()];
        assert!(check_help_version_flags(&args).is_some());
    }

    #[test]
    fn flags_sem_help_ou_version_retorna_none() {
        let args = vec!["capiba-mcp".into()];
        assert!(check_help_version_flags(&args).is_none());
    }

    #[test]
    fn flags_argumento_desconhecido_retorna_none() {
        let args = vec!["capiba-mcp".into(), "--unknown".into()];
        assert!(check_help_version_flags(&args).is_none());
    }

    #[test]
    fn flags_help_ignorado_com_outros_argumentos() {
        let args = vec!["capiba-mcp".into(), "--unknown".into(), "--help".into()];
        assert!(check_help_version_flags(&args).is_some());
    }

    #[test]
    fn flags_version_primeiro_argumento_retorna_some() {
        let args = vec!["capiba-mcp".into(), "-v".into(), "extra".into()];
        assert!(check_help_version_flags(&args).is_some());
    }
}
