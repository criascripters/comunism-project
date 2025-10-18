use crate::{app::App, ui::renderer::ui, utils::IGNORAR_CTRL_C};

use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyCode, KeyModifiers},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, prelude::CrosstermBackend};
use std::{
    io::{self, stdout},
    time::{Duration, Instant},
};

// encapsula o ciclo de vida do TUI e o loop principal
pub fn run(mut app: App) -> io::Result<()> {
    // configuração do terminal
    enable_raw_mode()?; // deixa ler cada tecla apertada
    stdout().execute(EnterAlternateScreen)?; // abre uma tela nova
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut need_redraw = true;
    let mut last_frame = Instant::now();

    loop {
        if app.sair || !IGNORAR_CTRL_C.load(std::sync::atomic::Ordering::SeqCst) {
            break;
        }

        // se overlay ta ativo queremos ~60 FPS; senao, 4 FPS ta ok
        let tick = if app.overlay.is_some() {
            Duration::from_millis(16)
        } else {
            Duration::from_millis(250)
        };

        // redesenha se:
        // tem overlay e chegou dado novo (dirty)
        // passou o tick (pra manter FPS/min)
        // houve interação de usuário (mais abaixo)
        let has_dirty = app
            .overlay
            .as_ref()
            .map(|o| o.take_dirty())
            .unwrap_or(false);
        if has_dirty || need_redraw || last_frame.elapsed() >= tick {
            terminal.draw(|frame| ui(frame, &mut app))?;
            need_redraw = false;
            last_frame = Instant::now();
        }

        // eventos de input (timeout pequeno para não bloquear a taxa de frames)
        if event::poll(Duration::from_millis(5))? {
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
                                need_redraw = true;
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
                            KeyCode::Down => {
                                app.proximo();
                                need_redraw = true;
                            }
                            KeyCode::Up => {
                                app.anterior();
                                need_redraw = true;
                            }
                            KeyCode::Enter => {
                                if let Err(e) = app.executar_selecionado() {
                                    // não derrube a TUI; logue e continue
                                    eprintln!("erro ao executar: {e}");
                                }
                                // overlay acabou de abrir → redesenhar
                                need_redraw = true;
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
