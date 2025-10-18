use crate::{app::App, ui::renderer::ui, utils::IGNORAR_CTRL_C};

use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyCode, KeyModifiers},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, prelude::CrosstermBackend};
use std::{
    io::{self, stdout},
    time::Duration,
};

// encapsula o ciclo de vida do TUI e o loop principal
pub fn run(mut app: App) -> io::Result<()> {
    // configuração do terminal
    enable_raw_mode()?; // deixa ler cada tecla apertada
    stdout().execute(EnterAlternateScreen)?; // abre uma tela nova
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    // o loop principal da aplicaçao
    while !app.sair && IGNORAR_CTRL_C.load(std::sync::atomic::Ordering::SeqCst) {
        // 1. desenha a interface
        terminal.draw(|frame| ui(frame, &mut app))?;

        // 2. checa se você apertou alguma tecla
        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Press {
                    // se tiver overlay ativo, redireciona input pro terminal
                    if let Some(term) = &mut app.overlay {
                        // envia Ctrl+C / Ctrl+D para o processo
                        if key.modifiers.contains(KeyModifiers::CONTROL) {
                            match key.code {
                                KeyCode::Char('c') => {
                                    term.send_str("\x03");
                                    continue;
                                } // SIGINT
                                KeyCode::Char('d') => {
                                    term.send_str("\x04");
                                    continue;
                                } // EOT
                                _ => {}
                            }
                        }
                        match key.code {
                            KeyCode::Esc | KeyCode::Char('q') => {
                                app.overlay = None; // fecha o overlay
                            }
                            _ => {
                                term.send_key(key.code); // envia tecla pro processo
                            }
                        }
                    } else {
                        // navegação normal quando não tem overlay
                        match key.code {
                            KeyCode::Char('q') => app.sair = true,
                            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                app.sair = true
                            }
                            KeyCode::Down => app.proximo(),
                            KeyCode::Up => app.anterior(),
                            KeyCode::Enter => {
                                if let Err(e) = app.executar_selecionado() {
                                    // não derrube a TUI; logue e continue
                                    eprintln!("erro ao executar: {e}");
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    // restauração do terminal
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
