use std::collections::HashMap;
use std::thread;
use std::time::Duration;

struct Panela {
    miojo: String,
    agua_quente: bool,
    tempero: String,
    cozido: bool,
}

impl Panela {
    fn new() -> Self {
        Self {
            miojo: "Miojo sabor galinha caipira".into(),
            agua_quente: false,
            tempero: "sachê misterioso".into(),
            cozido: false,
        }
    }

    fn ferver_agua(&mut self) {
        println!("🔥 Esquentando a água...");
        thread::sleep(Duration::from_secs(2));
        self.agua_quente = true;
        println!("💧 Água fervendo!");
    }

    fn adicionar_miojo(&mut self) {
        if !self.agua_quente {
            println!("Erro: a água ainda não ferveu!");
            return;
        }
        println!("🍜 Adicionando {} na água...", self.miojo);
        thread::sleep(Duration::from_secs(3));
        self.cozido = true;
        println!("⏳ Miojo cozido!");
    }

    fn misturar_tempero(&self) {
        if !self.cozido {
            println!("Erro: o miojo ainda está cru!");
            return;
        }
        println!("🧂 Misturando {}...", self.tempero);
        thread::sleep(Duration::from_secs(1));
        println!("✅ Miojo pronto!");
    }
}

fn main() {
    println!("=== Receita de Miojo ===");

    let mut panela = Panela::new();
    panela.ferver_agua();
    panela.adicionar_miojo();
    panela.misturar_tempero();

    println!("🍽️ Sirva-se com moderação.");
}
