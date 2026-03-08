use std::{env, path::Path, process};

fn main() {
    match env::args().nth(1).as_deref() {
        Some("install-tools") => install_tools(),
        _ => {
            eprintln!("Uso: cargo xtask <tarefa>");
            eprintln!();
            eprintln!("Tarefas disponíveis:");
            eprintln!("  install-tools   Instala ferramentas de dev de [workspace.metadata.tools]");
            process::exit(1);
        }
    }
}

fn install_tools() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let workspace_root = Path::new(manifest_dir).parent().unwrap();
    let cargo_toml_path = workspace_root.join("Cargo.toml");

    let content =
        std::fs::read_to_string(&cargo_toml_path).expect("Não foi possível ler Cargo.toml");

    let doc: toml::Value = content.parse().expect("Cargo.toml inválido");

    let tools = doc
        .get("workspace")
        .and_then(|w| w.get("metadata"))
        .and_then(|m| m.get("tools"))
        .and_then(|t| t.as_table())
        .expect("[workspace.metadata.tools] não encontrado em Cargo.toml");

    let has_binstall = process::Command::new("cargo")
        .args(["binstall", "--version"])
        .stdout(process::Stdio::null())
        .stderr(process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    println!(
        "→ instalando {} ferramentas via {}...",
        tools.len(),
        if has_binstall {
            "cargo binstall"
        } else {
            "cargo install"
        }
    );

    for (name, version) in tools {
        let ver = version.as_str().expect("versão deve ser string");
        print!("  {name}@{ver} ... ");

        let status = if has_binstall {
            process::Command::new("cargo")
                .args(["binstall", "--no-confirm", &format!("{name}@{ver}")])
                .status()
        } else {
            // cargo install requer semver completo ou qualificador (^0.6, não 0.6)
            let ver_qualified = if ver.chars().filter(|&c| c == '.').count() < 2 {
                format!("^{ver}")
            } else {
                ver.to_string()
            };
            process::Command::new("cargo")
                .args(["install", name, "--version", &ver_qualified, "--locked"])
                .status()
        };

        match status {
            Ok(s) if s.success() => println!("✓"),
            _ => {
                println!("✗");
                eprintln!("Erro ao instalar {name}. Abortando.");
                process::exit(1);
            }
        }
    }

    println!("✓ todas as ferramentas instaladas");
}
