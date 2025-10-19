use crate::scripts::{Executable, detectar_scripts};
use crate::ui::term_overlay::{MessageOverlay, Overlay};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use crossbeam_channel::{Receiver, Sender, unbounded};
use ratatui::widgets::ListState;

fn substituir_vars(template: &str, vars: &HashMap<String, String>) -> String {
    let mut resultado = template.to_string();
    for (k, v) in vars {
        resultado = resultado.replace(&format!("{{{}}}", k), v);
    }
    resultado
}

// uma struct pra guardar o estado da aplicaçao
pub struct App {
    pub codigos: Vec<Box<dyn Executable>>,
    pub script_selecionado: usize, // índice do script real (não da UI)
    pub ui_state: ListState,
    pub sair: bool,
    pub overlay: Option<Overlay>,
    tx: Sender<UiMsg>,
    rx: Receiver<UiMsg>,
}

enum UiMsg {
    SpawnOverlay {
        cmd: String,
        args: Vec<String>,
        cols: u16,
        rows: u16,
        title: String,
    },
    ShowMessage {
        title: String,
        text: String,
    },
}

impl App {
    // construtor pra criar o app
    pub fn new() -> App {
        let codigos = detectar_scripts();
        let mut ui_state = ListState::default();
        ui_state.select(Some(0));
        let (tx, rx) = unbounded();
        Apр {
            codigos,
            script_selecionado: 0,
            ui_state,
            sair: false,
            overlay: None,
            tx,
            rx,
        }
    }

    fn gamba_bin_candidates(&self) -> Vec<PathBuf> {
        let root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let bin_name = if cfg!(windows) { "gamba.exe" } else { "gamba" };
        // primeiro tenta o perfil atual (debug na maioria das vezes), depois o outro
        let profiles = if cfg!(debug_assertions) {
            vec!["debug", "release"]
        } else {
            vec!["release", "debug"]
        };
        profiles
            .into_iter()
            .map(|p| {
                let mut pb = root.clone();
                pb.push("target");
                pb.push(p);
                pb.push(bin_name);
                pb
            })
            .collect()
    }

    fn spawn_gamba_with_build_if_needed(
        &mut self,
        script_abs: String,
        title: String,
    ) -> Result<(), String> {
        // tenta achar um binário existente
        if let Some(bin) = self.gamba_bin_candidates().into_iter().find(|p| p.exists()) {
            let cmd = bin.to_string_lossy().to_string();
            let args = vec![script_abs];
            let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
            match crate::ui::term_overlay::TermOverlay::spawn(&cmd, &arg_refs, 80, 24, title) {
                Ok(overlay) => {
                    self.overlay = Some(Overlay::Terminal(overlay));
                    return Ok(());
                }
                Err(e) => {
                    self.overlay = Some(Overlay::Message(MessageOverlay::new(
                        "erro ao criar terminal",
                        format!("{e}"),
                    )));
                    return Err(format!("{e}"));
                }
            }
        }

        // não achou binário -> compila uma vez (debug por padrão)
        let overlay_temp = crate::ui::term_overlay::TermOverlay::spawn(
            "echo",
            &["compilando gamba...", "aguarde um momento"],
            80,
            22,
            format!("{} (compilando...)", title),
        )
        .map_err(|e| format!("falha ao criar terminal: {e}"))?;
        self.overlay = Some(Overlay::Terminal(overlay_temp));

        let tx = self.tx.clone();

        // precisamos desses valores dentro da thread
        let title_cl = title.clone();
        // após compilar, vamos usar target/debug/gamba como caminho padrão
        let mut debug_bin = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        debug_bin.push("target");
        debug_bin.push("debug");
        debug_bin.push(if cfg!(windows) { "gamba.exe" } else { "gamba" });
        let debug_bin_str = debug_bin.to_string_lossy().to_string();
        let script_abs_cl = script_abs.clone();

        std::thread::spawn(move || {
            match std::process::Command::new("cargo")
                .args(&["build", "-q", "-p", "gamba"])
                .output()
            {
                Ok(out) => {
                    if out.status.success() && Path::new(&debug_bin_str).exists() {
                        let _ = tx.send(UiMsg::SpawnOverlay {
                            cmd: debug_bin_str,
                            args: vec![script_abs_cl],
                            cols: 80,
                            rows: 24,
                            title: title_cl,
                        });
                    } else {
                        let err_text = format!(
                            "falha ao compilar gamba (status: {:?})
stdout:
{}
stderr:
{}",
                            out.status.code(),
                            String::from_utf8_lossy(&out.stdout),
                            String::from_utf8_lossy(&out.stderr),
                        );
                        let _ = tx.send(UiMsg::ShowMessage {
                            title: "compilação do gamba falhou".to_string(),
                            text: err_text,
                        });
                    }
                }
                Err(e) => {
                    let err_text = format!("não consegui invocar cargo build -p gamba: {}", e);
                    let _ = tx.send(UiMsg::ShowMessage {
                        title: "erro ao invocar cargo".to_string(),
                        text: err_text,
                    });
                }
            }
        });

        Ok(())
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
            if let Some(script) = atracao
                .as_any()
                .downcast_ref::<crate::scripts::executables::Script>()
            {
                // se o arquivo oficial termina com .ga, usa o binário gamba direto
                if std::path::Path::new(&script.arquivo)
                    .extension()
                    .and_then(|s| s.to_str())
                    .map(|ext| ext.eq_ignore_ascii_case("ga"))
                    .unwrap_or(false)
                {
                    let abs = std::fs::canonicalize(&script.arquivo)
                        .unwrap_or_else(|_| std::path::PathBuf::from(&script.arquivo));
                    return self.spawn_gamba_with_build_if_needed(
                        abs.to_string_lossy().to_string(),
                        script.nome().to_string(),
                    );
                }

                let mut vars = HashMap::new();
                vars.insert("arquivo".to_string(), script.arquivo.clone());

                if let Some(compile) = &script.compilar {
                    vars.insert("output".to_string(), compile.output.clone());

                    let overlay_temp = crate::ui::term_overlay::TermOverlay::spawn(
                        "echo",
                        &["compilando...", "aguarde um momento"],
                        80,
                        22,
                        format!("{} (compilando...)", script.nome()),
                    )
                    .map_err(|e| format!("falha ao criar terminal: {e}"))?;
                    self.overlay = Some(Overlay::Terminal(overlay_temp));

                    let cargs: Vec<String> = compile
                        .args
                        .iter()
                        .map(|a| substituir_vars(a, &vars))
                        .collect();
                    let compilador = compile.compilador.clone();

                    let cmd_res = substituir_vars(&script.executar.comando, &vars);
                    let mut args_res: Vec<String> = script
                        .executar
                        .args
                        .iter()
                        .map(|a| substituir_vars(a, &vars))
                        .collect();

                    if let Ok(abs) = std::fs::canonicalize(&script.arquivo) {
                        for a in &mut args_res {
                            if a == &script.arquivo {
                                *a = abs.to_string_lossy().to_string();
                            }
                        }
                    }

                    let (cols, rows) = if script.nome.to_lowercase().contains("rosquinha") {
                        (80, 22)
                    } else {
                        (80, 24)
                    };

                    let tx = self.tx.clone();
                    let title = script.nome().to_string();
                    let compilador_cl = compilador.clone();
                    let cargs_cl = cargs.clone();

                    std::thread::spawn(move || {
                        match Command::new(&compilador_cl).args(&cargs_cl).output() {
                            Ok(out) => {
                                if out.status.success() {
                                    let _ = tx.send(UiMsg::SpawnOverlay {
                                        cmd: cmd_res,
                                        args: args_res,
                                        cols,
                                        rows,
                                        title,
                                    });
                                } else {
                                    let err_text = format!(
                                        "falha ao compilar (status: {:?})
stdout:
{}
stderr:
{}",
                                        out.status.code(),
                                        String::from_utf8_lossy(&out.stdout),
                                        String::from_utf8_lossy(&out.stderr),
                                    );
                                    let _ = tx.send(UiMsg::ShowMessage {
                                        title: "compilação falhou".to_string(),
                                        text: err_text,
                                    });
                                }
                            }
                            Err(e) => {
                                let err_text = format!(
                                    "não consegui invocar o compilador
comando: {} {:?}
erro: {}",
                                    compilador_cl, cargs_cl, e
                                );
                                let _ = tx.send(UiMsg::ShowMessage {
                                    title: "erro ao invocar compilador".to_string(),
                                    text: err_text,
                                });
                            }
                        }
                    });

                    return Ok(());
                }

                let cmd_res = substituir_vars(&script.executar.comando, &vars);
                let mut args_res: Vec<String> = script
                    .executar
                    .args
                    .iter()
                    .map(|a| substituir_vars(a, &vars))
                    .collect();

                if let Ok(abs) = std::fs::canonicalize(&script.arquivo) {
                    for a in &mut args_res {
                        if a == &script.arquivo {
                            *a = abs.to_string_lossy().to_string();
                        }
                    }
                }

                let (cols, rows) = if script.nome.to_lowercase().contains("rosquinha") {
                    (80, 22)
                } else {
                    (80, 24)
                };

                let arg_refs: Vec<&str> = args_res.iter().map(|s| s.as_str()).collect();
                match crate::ui::term_overlay::TermOverlay::spawn(
                    &cmd_res,
                    &arg_refs,
                    cols,
                    rows,
                    script.nome(),
                ) {
                    Ok(overlay) => {
                        self.overlay = Some(Overlay::Terminal(overlay));
                    }
                    Err(e) => {
                        self.overlay = Some(Overlay::Message(MessageOverlay::new(
                            "erro ao criar terminal",
                            format!("{e}"),
                        )));
                    }
                }
            }

            if let Some(auto) = atracao
                .as_any()
                .downcast_ref::<crate::scripts::executables::ScriptNaoOficial>()
            {
                let mut path = std::path::PathBuf::from(&auto.arquivo);
                if path.is_relative() {
                    if let Ok(abs) = std::fs::canonicalize(&path) {
                        path = abs;
                    }
                }

                // trata .ga de forma especial: usa binário em target/ e compila se necessário (uma vez)
                let is_ga = path
                    .extension()
                    .and_then(|s| s.to_str())
                    .map(|s| s.eq_ignore_ascii_case("ga"))
                    .unwrap_or(false);

                if is_ga {
                    let title = auto.nome().to_string();
                    let script_abs = path.to_string_lossy().to_string();
                    return self.spawn_gamba_with_build_if_needed(script_abs, title);
                }

                // comportamento padrão para outras linguagens
                let parts: Vec<&str> = auto.interpretador.split_whitespace().collect();
                let cmd = parts.get(0).map(|s| s.to_string()).unwrap_or_default();
                let mut args: Vec<String> =
                    parts.iter().skip(1).map(|s| (*s).to_string()).collect();
                args.push(path.to_string_lossy().to_string());

                let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
                match crate::ui::term_overlay::TermOverlay::spawn(
                    &cmd,
                    &arg_refs,
                    80,
                    24,
                    auto.nome(),
                ) {
                    Ok(overlay) => {
                        self.overlay = Some(Overlay::Terminal(overlay));
                    }
                    Err(e) => {
                        self.overlay = Some(Overlay::Message(MessageOverlay::new(
                            "erro ao executar",
                            format!("{e}"),
                        )));
                    }
                }
            }
        }

        Ok(())
    }

    pub fn process_messages(&mut self) -> bool {
        let mut changed = false;
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                UiMsg::SpawnOverlay {
                    cmd,
                    args,
                    cols,
                    rows,
                    title,
                } => {
                    let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
                    if let Ok(overlay) = crate::ui::term_overlay::TermOverlay::spawn(
                        &cmd, &arg_refs, cols, rows, title,
                    ) {
                        self.overlay = Some(Overlay::Terminal(overlay));
                        changed = true;
                    } else {
                        self.overlay = Some(Overlay::Message(MessageOverlay::new(
                            "erro ao abrir terminal",
                            format!("comando: {} {:?}", cmd, args),
                        )));
                        changed = true;
                    }
                }
                UiMsg::ShowMessage { title, text } => {
                    self.overlay = Some(Overlay::Message(MessageOverlay::new(title, text)));
                    changed = true;
                }
            }
        }
        changed
    }
}
