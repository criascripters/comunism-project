use crate::executables::{Executable, detectar_scripts};
use crate::signals::EXECUTANDO_SCRIPT;

use crossterm::{
    ExecutableCommand,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::widgets::ListState;
use std::io::{self, stdout};

// uma struct pra guardar o estado da aplicaçao
pub struct App {
    pub codigos: Vec<Box<dyn Executable>>,
    pub script_selecionado: usize, // índice do script real (não da UI)
    pub ui_state: ListState,
    pub sair: bool,
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
            // "desliga" o tui temporariamente
            // pra não bugar tudo quando o script c tentar limpar a tela
            disable_raw_mode().unwrap();
            stdout().execute(LeaveAlternateScreen).unwrap();

            // avisa que ta executando um script
            EXECUTANDO_SCRIPT.store(true, std::sync::atomic::Ordering::SeqCst);

            // executa a atração (filho recebe SIGINT automaticamente se pressionado)
            let resultado = atracao.execute();

            // avisa que terminou de executar
            EXECUTANDO_SCRIPT.store(false, std::sync::atomic::Ordering::SeqCst);

            // nao volta pro tui automaticamente, deixa o usuario ver o output
            println!("\npressione enter para voltar ao menu");
            // desativa raw mode temporariamente pra ler input normal
            disable_raw_mode().unwrap();

            // espera o usuario apertar enter antes de voltar ao menu
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();

            // reativa raw mode antes de voltar pro tui
            enable_raw_mode().unwrap();

            // limpa a tela e restaura o cursor antes de voltar pro tui
            print!("\x1B[2J\x1B[H");
            io::Write::flush(&mut stdout()).unwrap();

            // força uma atualizacao completa do terminal
            std::thread::sleep(std::time::Duration::from_millis(50));
            print!("\x1Bc"); // reset completo do terminal
            io::Write::flush(&mut stdout()).unwrap();

            // e agora religa o tui pra voltar ao nosso menu
            enable_raw_mode().unwrap();
            stdout().execute(EnterAlternateScreen).unwrap();

            return resultado;
        }

        Ok(()) // se nada estiver selecionado, não faz nada
    }
}
