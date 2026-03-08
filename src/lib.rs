mod prompts;
use prompts::*;

use zed_extension_api::{
    self as zed, Result, SlashCommand, SlashCommandArgumentCompletion, SlashCommandOutput,
    SlashCommandOutputSection, Worktree,
};

struct CapibaExtension;

impl zed::Extension for CapibaExtension {
    fn new() -> Self {
        CapibaExtension
    }

    fn run_slash_command(
        &self,
        command: SlashCommand,
        args: Vec<String>,
        worktree: Option<&Worktree>,
    ) -> Result<SlashCommandOutput, String> {
        match command.name.as_str() {
            "capiba-onboard" => self.cmd_onboard(worktree),
            "capiba-fase" => self.cmd_fase(args, worktree),
            "capiba-historia" => self.cmd_simples(PROMPT_HISTORIA, worktree),
            "capiba-ideia" => self.cmd_simples(PROMPT_IDEIA, worktree),
            "capiba-revisar" => self.cmd_simples(PROMPT_REVISAR, worktree),
            "capiba-teste" => self.cmd_simples(PROMPT_TESTE, worktree),
            "capiba-compat" => self.cmd_simples(PROMPT_COMPAT, worktree),
            "capiba-pr" => self.cmd_simples(PROMPT_PR, worktree),
            cmd => Err(format!("comando desconhecido: {cmd}")),
        }
    }

    fn complete_slash_command_argument(
        &self,
        command: SlashCommand,
        _args: Vec<String>,
    ) -> Result<Vec<SlashCommandArgumentCompletion>, String> {
        match command.name.as_str() {
            "capiba-fase" => Ok(vec![
                completion("preparacao", "Fase 1 — nomear o problema e o sujeito"),
                completion("desenvolvimento", "Fase 2 — construir a história"),
                completion(
                    "garantia",
                    "Fase 3 — verificar técnico, ético e territorial",
                ),
                completion("entrega", "Fase 4 — integrar ao ecossistema"),
                completion("consolidacao", "Fase 5 — transformar em memória coletiva"),
            ]),
            _ => Ok(vec![]),
        }
    }
}

// ─── Comandos ─────────────────────────────────────────────────────────────────

impl CapibaExtension {
    fn cmd_onboard(&self, worktree: Option<&Worktree>) -> Result<SlashCommandOutput, String> {
        let context = self.read_context(worktree);
        let intro_end = PROMPT_ONBOARD_INTRO.len();
        let text = format!(
            "{PROMPT_ONBOARD_INTRO}\n\n---\n\n{context}\n\n---\n\n{PROMPT_ONBOARD_INSTRUCOES}"
        );
        Ok(SlashCommandOutput {
            sections: vec![SlashCommandOutputSection {
                range: (0..intro_end).into(),
                label: "Contexto Capiba".to_string(),
            }],
            text,
        })
    }

    fn cmd_fase(
        &self,
        args: Vec<String>,
        worktree: Option<&Worktree>,
    ) -> Result<SlashCommandOutput, String> {
        let fase = args.first().ok_or_else(|| {
            "informe a fase: preparacao | desenvolvimento | garantia | entrega | consolidacao"
                .to_string()
        })?;

        let prompt = match fase.as_str() {
            "preparacao"      => PROMPT_FASE_1,
            "desenvolvimento" => PROMPT_FASE_2,
            "garantia"        => PROMPT_FASE_3,
            "entrega"         => PROMPT_FASE_4,
            "consolidacao"    => PROMPT_FASE_5,
            f => return Err(format!("fase desconhecida: '{f}'. Use: preparacao | desenvolvimento | garantia | entrega | consolidacao")),
        };

        self.cmd_simples(prompt, worktree)
    }

    /// Monta output padrão: contexto do worktree + prompt da slash command
    fn cmd_simples(
        &self,
        prompt: &str,
        worktree: Option<&Worktree>,
    ) -> Result<SlashCommandOutput, String> {
        let context = self.read_context(worktree);
        let text = if context.is_empty() {
            prompt.to_string()
        } else {
            format!("{context}\n\n---\n\n{prompt}")
        };
        Ok(SlashCommandOutput {
            sections: vec![],
            text,
        })
    }

    /// Lê arquivos de contexto do worktree atual
    fn read_context(&self, worktree: Option<&Worktree>) -> String {
        let Some(wt) = worktree else {
            return String::new();
        };

        let mut partes = Vec::new();

        if let Ok(claude_md) = wt.read_text_file("CLAUDE.md") {
            if !claude_md.trim().is_empty() {
                partes.push(format!("# Contexto do Projeto (CLAUDE.md)\n\n{claude_md}"));
            }
        }

        if let Ok(contributing) = wt.read_text_file("CONTRIBUTING.md") {
            if !contributing.trim().is_empty() {
                partes.push(format!("# CONTRIBUTING\n\n{contributing}"));
            }
        }

        partes.join("\n\n---\n\n")
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn completion(label: &str, desc: &str) -> SlashCommandArgumentCompletion {
    SlashCommandArgumentCompletion {
        label: format!("{label} — {desc}"),
        new_text: label.to_string(),
        run_command: true,
    }
}

zed::register_extension!(CapibaExtension);
