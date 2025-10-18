use crate::scripts::{Executable, detectar_scripts};
use crate::ui::term_overlay::TermOverlay;
use std::collections::HashMap;
use std::process::Command;

use ratatui::widgets::ListState;

// uma struct pra guardar o estado da aplicaçao
pub struct App {
    pub codigos: Vec<Box<dyn Executable>>,
    pub script_selecionado: usize, // índice do script real (não da UI)
    pub ui_state: ListState,
    pub sair: bool,
    pub overlay: Option<TermOverlay>,
}

impl App {
    // construtor pra criar o app
    pub fn new() -> App {
        let codigos = detectar_scripts();
        let mut ui_state = ListState::default();
        ui_state.select(Some(0));
        App {
            codigos,
            script_selecionado: 0,
            ui_state,
            sair: false,
            overlay: None,
        }
    }

    // navegação pra quando tiver mais itens
    pub fn proximo(&mut self) {
        if self.codigos.is_empty() {
            return;
        }

        self.script_selecionado = if self.script_selecionado >= self.codigos.len() - 1 {
            0
        } else {
            self.script_selecionado + 1
        };

        // reseta a seleção visual - vai ser recalculada na UI
        self.ui_state.select(None);
    }

    pub fn anterior(&mut self) {
        if self.codigos.is_empty() {
            return;
        }

        self.script_selecionado = if self.script_selecionado == 0 {
            self.codigos.len() - 1
        } else {
            self.script_selecionado - 1
        };

        // reseta a seleção visual - vai ser recalculada na UI
        self.ui_state.select(None);
    }

    // o truque de mágica: executar o item selecionado
    pub fn executar_selecionado(&mut self) -> Result<(), String> {
        if let Some(atracao) = self.codigos.get(self.script_selecionado) {
            // 1) Script oficial (.ron): compila (se precisar) e substitui {arquivo}/{output}
            if let Some(script) = atracao
                .as_any()
                .downcast_ref::<crate::scripts::executables::Script>()
            {
                let mut vars = HashMap::new();
                vars.insert("arquivo".to_string(), script.arquivo.clone());

                // compile step
                if let Some(compile) = &script.compilar {
                    vars.insert("output".to_string(), compile.output.clone());

                    // abre overlay instantaneamente com mensagem de compilação
                    let overlay_temp = crate::ui::term_overlay::TermOverlay::spawn(
                        "echo",
                        &["compilando...", "aguarde um momento"],
                        80,
                        22,
                        format!("{} (compilando...)", script.nome()),
                    )
                    .map_err(|e| format!("falha ao criar terminal: {e}"))?;
                    self.overlay = Some(overlay_temp);

                    let cargs: Vec<String> = compile
                        .args
                        .iter()
                        .map(|a| substituir_vars(a, &vars))
                        .collect();

                    let status = Command::new(&compile.compilador)
                        .args(&cargs)
                        .status()
                        .map_err(|e| format!("falha ao compilar {}: {}", script.arquivo, e))?;

                    if !status.success() {
                        return Err(format!("compilação de {} falhou", script.arquivo));
                    }
                }

                fn substituir_vars(template: &str, vars: &HashMap<String, String>) -> String {
                    let mut resultado = template.to_string();
                    for (k, v) in vars {
                        resultado = resultado.replace(&format!("{{{}}}", k), v);
                    }
                    resultado
                }

                // substitui em comando e args (usa {output} se tiver)
                let cmd_res = substituir_vars(&script.executar.comando, &vars);
                let mut args_res: Vec<String> = script
                    .executar
                    .args
                    .iter()
                    .map(|a| substituir_vars(a, &vars))
                    .collect();

                // se algum arg é o arquivo original, torne absoluto (ajuda node/python)
                if let Ok(abs) = std::fs::canonicalize(&script.arquivo) {
                    for a in &mut args_res {
                        if a == &script.arquivo {
                            *a = abs.to_string_lossy().to_string();
                        }
                    }
                }

                // tamanhos (rosquinha precisa de 80x22, o resto pode ser 80x24)
                let (cols, rows) = if script.nome.to_lowercase().contains("rosquinha") {
                    (80, 22)
                } else {
                    (80, 24)
                };

                let arg_refs: Vec<&str> = args_res.iter().map(|s| s.as_str()).collect();
                // substitui o overlay temporário pelo processo real
                let overlay = crate::ui::term_overlay::TermOverlay::spawn(
                    &cmd_res,
                    &arg_refs,
                    cols,
                    rows,
                    script.nome(),
                )
                .map_err(|e| format!("falha ao criar terminal: {e}"))?;
                self.overlay = Some(overlay);
                return Ok(());
            }

            // script não-oficial (auto)
            if let Some(auto) = atracao
                .as_any()
                .downcast_ref::<crate::scripts::executables::ScriptNaoOficial>()
            {
                // arquivo absoluto
                let mut path = std::path::PathBuf::from(&auto.arquivo);
                if path.is_relative() {
                    if let Ok(abs) = std::fs::canonicalize(&path) {
                        path = abs;
                    }
                }
                let args = vec![path.to_string_lossy().to_string()];
                let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
                let overlay = crate::ui::term_overlay::TermOverlay::spawn(
                    &auto.interpretador,
                    &arg_refs,
                    80,
                    24,
                    auto.nome(),
                )
                .map_err(|e| format!("falha ao criar terminal: {e}"))?;
                self.overlay = Some(overlay);
                return Ok(());
            }
        }

        Ok(()) // se nada estiver selecionado, não faz nada
    }
}
