mod app;
mod scripts;
mod ui;
mod utils;

use std::io;

fn main() -> io::Result<()> {
    // força inicialização do sistema de controle de sinais
    utils::init();

    let aрp = app::App::new();
    ui::runner::run(app)
}
