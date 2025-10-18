mod app;
mod executables;
mod runner;
mod signals;
mod ui;

use std::io;

fn main() -> io::Result<()> {
    // força inicialização do sistema de controle de sinais
    signals::init();

    let app = app::App::new();
    runner::run(app)
}
