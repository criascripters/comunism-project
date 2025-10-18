use ctrlc;
use once_cell::sync::Lazy;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

// flag global pra saber se ta executando um script
pub static EXECUTANDO_SCRIPT: Lazy<Arc<AtomicBool>> =
    Lazy::new(|| Arc::new(AtomicBool::new(false)));

// o filho ja recebe o sinal automaticamente no mesmo grupo de processos
pub static IGNORAR_CTRL_C: Lazy<Arc<AtomicBool>> = Lazy::new(|| {
    let flag = Arc::new(AtomicBool::new(true));

    // registra o handler uma vez durante toda a vida do programa
    ctrlc::set_handler({
        let flag = flag.clone();
        let executando = EXECUTANDO_SCRIPT.clone();
        move || {
            if executando.load(Ordering::SeqCst) {
                // se ta executando script, faz nada (o filho trata)
                return;
            }
            // se não ta executando, permite sair
            flag.store(false, Ordering::SeqCst);
        }
    })
    .expect("falha ao registrar handler de sinais - o caos venceu");

    flag
});

// função utilitária só pra inicializar os sinais quando o programa começa
pub fn init() {
    // força inicialização do sistema de controle de sinais
    let _ = &*IGNORAR_CTRL_C;
}
