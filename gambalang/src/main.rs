// src/main.rs
use gamba::{eval_in_runtime, Runtime, Value};
use std::env;
use std::fs;
use std::io::{self, IsTerminal, Read};
use std::path::Path;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    let args: Vec<String> = env::args().collect();
    let bin = args.get(0).map(|s| s.as_str()).unwrap_or("gamba");

    // parse arg simples (sem deps)
    let mut eval_code: Option<String> = None;
    let mut force_repl = false;
    let mut script_path: Option<String> = None;
    let mut script_args: Vec<String> = Vec::new();
    let mut after_dd = false;

    // args[0] é o bin
    let mut i = 1;
    while i < args.len() {
        let a = &args[i];
        if after_dd {
            script_args.push(a.clone());
            i += 1;
            continue;
        }
        match a.as_str() {
            "-h" | "--help" => {
                print_usage(bin);
                std::process::exit(0);
            }
            "-V" | "--version" => {
                println!("gamba {}", VERSION);
                std::process::exit(0);
            }
            "--repl" => {
                force_repl = true;
                i += 1;
            }
            "-e" | "--eval" => {
                if i + 1 >= args.len() {
                    eprintln!("{}: faltou o código após {}", bin, a);
                    std::process::exit(2);
                }
                eval_code = Some(args[i + 1].clone());
                i += 2;
            }
            "--" => {
                after_dd = true;
                i += 1;
            }
            "-" => {
                // "-" tratado como script do stdin
                if script_path.is_none() {
                    script_path = Some("-".to_string());
                } else {
                    script_args.push("-".to_string());
                }
                i += 1;
            }
            _ if a.starts_with('-') => {
                eprintln!("{}: opção desconhecida: {}", bin, a);
                eprintln!("use --help para ajuda.");
                std::process::exit(2);
            }
            _ => {
                if script_path.is_none() {
                    script_path = Some(a.clone());
                } else {
                    script_args.push(a.clone());
                }
                i += 1;
            }
        }
    }

    let stdin_is_tty = io::stdin().is_terminal();

    // cria runtime com builtins
    let mut runtime = Runtime::with_builtins();

    // prioridades de modo:
    // 1) -e/--eval
    // 2) script path (ou "-" => stdin)
    // 3) se stdin não é TTY, ler script de stdin
    // 4) repl (se --repl ou default sem args)
    if let Some(code) = eval_code {
        inject_meta(&mut runtime, "eval", None, &[]).ok();
        let exit = run_source(&mut runtime, &code, "<eval>", PrintMode::Silent);
        std::process::exit(exit);
    }

    if let Some(path) = script_path {
        if path == "-" {
            // script do stdin explicitamente
            match read_all_stdin() {
                Ok(mut src) => {
                    src = strip_shebang(&src);
                    inject_meta(&mut runtime, "stdin", Some("stdin"), &script_args).ok();
                    let exit = run_source(&mut runtime, &src, "stdin", PrintMode::Silent);
                    std::process::exit(exit);
                }
                Err(e) => {
                    eprintln!("erro ao ler stdin: {}", e);
                    std::process::exit(1);
                }
            }
        } else {
            // arquivo
            match fs::read_to_string(&path) {
                Ok(mut source) => {
                    source = strip_shebang(&source);
                    inject_meta(&mut runtime, "file", Some(&path), &script_args).ok();
                    let exit = run_source(&mut runtime, &source, &path, PrintMode::Silent);
                    std::process::exit(exit);
                }
                Err(e) => {
                    eprintln!("erro ao ler '{}': {}", path, e);
                    std::process::exit(1);
                }
            }
        }
    }

    if !stdin_is_tty && !force_repl {
        // sem args e com pipe: roda stdin como script
        match read_all_stdin() {
            Ok(mut src) => {
                src = strip_shebang(&src);
                inject_meta(&mut runtime, "stdin", Some("stdin"), &[]).ok();
                let exit = run_source(&mut runtime, &src, "stdin", PrintMode::Silent);
                std::process::exit(exit);
            }
            Err(e) => {
                eprintln!("erro ao ler stdin: {}", e);
                std::process::exit(1);
            }
        }
    }

    // REPL
    inject_meta(&mut runtime, "repl", None, &[]).ok();
    let exit = run_repl(&mut runtime);
    std::process::exit(exit);
}

fn print_usage(bin: &str) {
    println!(
        "gamba {} - interpretador de scripts gamba

uso:
  {bin} [opções] [script] [-- args...]
  {bin} -e \"código\"
  {bin} --repl
  echo 'println(1+2)' | {bin}

opções:
  -e, --eval CODE     avalia um trecho de código
  --repl              força o modo repl
  -                   lê o script do stdin
  -h, --help          mostra esta ajuda
  -V, --version       mostra a versão

exemplos:
  {bin} testes/tutor.ga
  {bin} testes/fireworks.ga
  {bin} script.ga -- foo bar
  {bin} -e \"println(1+2)\"
  echo 'println(\"oi\")' | {bin}
",
        VERSION,
        bin = bin
    );
}

enum PrintMode {
    //sSilent: não imprime o valor final do programa (estilo python)
    Silent,
    // repl: imprime o valor da expressão quando não for Unit
    Repl,
}

fn read_all_stdin() -> io::Result<String> {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;
    Ok(buffer)
}

// suporta shebang no topo (#!...): remove a primeira linha se começar com "#!"
fn strip_shebang(src: &str) -> String {
    let mut lines = src.lines();
    if let Some(first) = lines.next() {
        if first.starts_with("#!") {
            // pula a primeira linha e junta o resto
            return lines.collect::<Vec<_>>().join(
                "
",
            );
        }
    }
    src.to_string()
}

// injeta metadados no ambiente: args, __file__, __dir__, __mode__
fn inject_meta(
    rt: &mut Runtime,
    mode: &str,
    script_path: Option<&str>,
    script_args: &[String],
) -> Result<(), gamba::GambaError> {
    use gamba::Value;

    // args: lista de strings com os argumentos do script após --
    let args_list = Value::List(script_args.iter().cloned().map(Value::String).collect());
    rt.env.set("args".to_string(), args_list)?;

    // __mode__: "file" | "stdin" | "eval" | "repl"
    rt.env
        .set("__mode__".to_string(), Value::String(mode.to_string()))?;

    // __file__ e __dir__
    match script_path {
        Some(p) => {
            rt.env
                .set("__file__".to_string(), Value::String(p.to_string()))?;
            let dir = Path::new(p)
                .parent()
                .map(|d| d.to_string_lossy().to_string())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| ".".to_string());
            rt.env.set("__dir__".to_string(), Value::String(dir))?;
        }
        None => {
            let tag = match mode {
                "stdin" => "stdin",
                "eval" => "<eval>",
                "repl" => "<repl>",
                _ => "<unknown>",
            };
            rt.env
                .set("__file__".to_string(), Value::String(tag.to_string()))?;
            rt.env
                .set("__dir__".to_string(), Value::String(".".to_string()))?;
        }
    }

    Ok(())
}

fn run_source(rt: &mut Runtime, source: &str, name: &str, mode: PrintMode) -> i32 {
    match eval_in_runtime(rt, source) {
        Ok(value) => {
            match mode {
                PrintMode::Silent => {
                    // estilo linguagem de script: nao imprimir o valor final automaticamente
                }
                PrintMode::Repl => {
                    if !matches!(value, Value::Unit) {
                        println!("{}", value);
                    }
                }
            }
            0
        }
        Err(e) => {
            eprintln!("erro em {}: {}", name, e);
            // se tem linha/col, mostra um trecho com caret
            if e.line > 0 {
                let line_idx = e.line.saturating_sub(1);
                if let Some(src_line) = source.lines().nth(line_idx) {
                    eprintln!("--> {}:{}:{}", name, e.line, e.col);
                    eprintln!("{}", src_line);
                    if e.col > 0 {
                        let mut caret = String::new();
                        for _ in 1..e.col {
                            caret.push(' ');
                        }
                        caret.push('^');
                        eprintln!("{}", caret);
                    }
                }
            }
            1
        }
    }
}

fn run_repl(rt: &mut Runtime) -> i32 {
    use std::io::{self, Write};
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut buffer = String::new();

    println!("gamba {} — repl", VERSION);
    println!("dicas: use ctrl+c para sair; avalie uma expressão por linha.");

    loop {
        buffer.clear();
        print!("gamba> ");
        stdout.flush().ok();
        if stdin.read_line(&mut buffer).is_err() {
            println!();
            break;
        }
        if buffer.trim().is_empty() {
            continue;
        }
        let src = buffer.trim_end();
        // no repl, imprime o valor (quando não for Unit)
        let _ = run_source(rt, src, "<repl>", PrintMode::Repl);
    }
    0
}
