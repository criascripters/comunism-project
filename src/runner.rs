use crate::{app::App, signals::IGNORAR_CTRL_C, ui::ui};

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
                    match key.code {
                        KeyCode::Char('q') => app.sair = true,
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            app.sair = true
                        }
                        KeyCode::Down => app.proximo(),
                        KeyCode::Up => app.anterior(),
                        KeyCode::Enter => {
                            if let Err(e) = app.executar_selecionado() {
                                // se der erro, pode mostrar na tela no futuro
                                // por enquanto, só encerra pra não travar
                                app.sair = true;
                                println!("deu erro: {}", e);
                            }
                        }
                        _ => {}
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
