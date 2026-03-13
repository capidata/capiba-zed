// ─── Prompts ──────────────────────────────────────────────────────────────────

use capiba_prompts::*;
use rmcp::model::*;
use rmcp::ErrorData as McpError;

pub fn principios_texto() -> String {
    PRINCIPIOS
        .iter()
        .enumerate()
        .map(|(i, p)| format!("## Princípio {}\n\n{}", i + 1, p))
        .collect::<Vec<_>>()
        .join("\n\n---\n\n")
}

pub fn normalize_fase(input: &str) -> String {
    input
        .to_lowercase()
        .replace("ã", "a")
        .replace("á", "a")
        .replace("ç", "c")
        .replace("é", "e")
        .replace("í", "i")
        .replace("ó", "o")
        .replace("ú", "u")
        .trim()
        .to_string()
}

pub fn get_prompt_texto(name: &str, fase: Option<&str>) -> Result<String, McpError> {
    match name {
        "capiba-onboard" => Ok(PROMPT_ONBOARD.to_string()),
        "capiba-historia" => Ok(PROMPT_HISTORIA.to_string()),
        "capiba-ideia" => Ok(PROMPT_IDEIA.to_string()),
        "capiba-revisar" => Ok(PROMPT_REVISAR.to_string()),
        "capiba-teste" => Ok(PROMPT_TESTE.to_string()),
        "capiba-compat" => Ok(PROMPT_COMPAT.to_string()),
        "capiba-pr" => Ok(PROMPT_PR.to_string()),
        "capiba-fase" => {
            let f = normalize_fase(fase.unwrap_or("1"));
            match f.as_str() {
                "1" | "preparacao" => Ok(PROMPT_FASE_1.to_string()),
                "2" | "desenvolvimento" => Ok(PROMPT_FASE_2.to_string()),
                "3" | "garantia" => Ok(PROMPT_FASE_3.to_string()),
                "4" | "entrega" => Ok(PROMPT_FASE_4.to_string()),
                "5" | "consolidacao" => Ok(PROMPT_FASE_5.to_string()),
                f => Err(McpError::invalid_params(
                    format!("fase desconhecida: '{f}' — use 1-5 ou: preparacao | desenvolvimento | garantia | entrega | consolidacao"),
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

pub fn capiba_prompts_list() -> Vec<Prompt> {
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
            "capiba-historia",
            Some("Sessão para escrever história de contribuição: Como [quem], quero [o quê], para que [por quê]"),
            None,
        ),
        Prompt::new(
            "capiba-ideia",
            Some("Sessão de ideação de nova feature ou solução para o ecossistema"),
            None,
        ),
        Prompt::new(
            "capiba-revisar",
            Some("Revisão ética e técnica de código contra os princípios do Pacto Fundante"),
            None,
        ),
        Prompt::new(
            "capiba-teste",
            Some("Gera testes contextualizados no ecossistema Capiba para o código selecionado"),
            None,
        ),
        Prompt::new(
            "capiba-compat",
            Some("Verifica compatibilidade com os 8 princípios invioláveis do Pacto Fundante"),
            None,
        ),
        Prompt::new(
            "capiba-pr",
            Some("Geração de descrição de PR com checklists técnico e ético"),
            None,
        ),
    ]
}

// ─── Testes ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use capiba_prompts::PRINCIPIOS;

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
        use crate::tools::get_principio_logic;
        assert!(get_principio_logic(1).contains("Soberania"));
    }

    #[test]
    fn get_principio_8_retorna_autocorrecao() {
        use crate::tools::get_principio_logic;
        assert!(get_principio_logic(8).contains("Autocorreção"));
    }

    #[test]
    fn get_principio_0_retorna_primeiro_por_saturating_sub() {
        use crate::tools::get_principio_logic;
        // saturating_sub(1) de 0u8 == 0 → PRINCIPIOS[0]
        assert!(get_principio_logic(0).contains("Soberania"));
    }

    #[test]
    fn get_principio_9_retorna_mensagem_de_erro() {
        use crate::tools::get_principio_logic;
        assert!(get_principio_logic(9).contains("não encontrado"));
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
    fn prompt_historia_ok() {
        assert!(get_prompt_texto("capiba-historia", None).is_ok());
    }

    #[test]
    fn prompt_ideia_ok() {
        assert!(get_prompt_texto("capiba-ideia", None).is_ok());
    }

    #[test]
    fn prompt_revisar_ok() {
        assert!(get_prompt_texto("capiba-revisar", None).is_ok());
    }

    #[test]
    fn prompt_teste_ok() {
        assert!(get_prompt_texto("capiba-teste", None).is_ok());
    }

    #[test]
    fn prompt_compat_ok() {
        assert!(get_prompt_texto("capiba-compat", None).is_ok());
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
    fn prompt_fase_com_numeros_1_a_5() {
        for num in ["1", "2", "3", "4", "5"] {
            assert!(
                get_prompt_texto("capiba-fase", Some(num)).is_ok(),
                "falhou com número: {num}"
            );
        }
    }

    #[test]
    fn prompt_fase_com_acentos_preparacao() {
        assert!(get_prompt_texto("capiba-fase", Some("PREPARAÇÃO")).is_ok());
        assert!(get_prompt_texto("capiba-fase", Some("Preparação")).is_ok());
    }

    #[test]
    fn prompt_fase_com_acentos_consolidacao() {
        assert!(get_prompt_texto("capiba-fase", Some("CONSOLIDAÇÃO")).is_ok());
        assert!(get_prompt_texto("capiba-fase", Some("Consolidação")).is_ok());
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

    // ── capiba_prompts_list ───────────────────────────────────────────────────

    #[test]
    fn prompts_list_tem_oito_itens() {
        assert_eq!(capiba_prompts_list().len(), 8);
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
}
