// ─── Resources ────────────────────────────────────────────────────────────────

use rmcp::model::*;
use std::path::Path;

pub fn capiba_resources_list() -> Vec<Resource> {
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

pub fn make_resource(uri: &str, name: &str, description: &str) -> Resource {
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

pub fn detectar_fase(path: &Path, has_staged: bool) -> (&'static str, &'static str) {
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

// ─── Testes ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn tmp(suffix: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("capiba_{suffix}"));
        fs::create_dir_all(&d).unwrap();
        d
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

    // ── make_resource ─────────────────────────────────────────────────────────

    #[test]
    fn make_resource_popula_campos() {
        let r = make_resource("capiba://test", "Nome", "Desc");
        assert_eq!(r.raw.uri, "capiba://test");
        assert_eq!(r.raw.name, "Nome");
        assert_eq!(r.raw.description.as_deref(), Some("Desc"));
    }
}
