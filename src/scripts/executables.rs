use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

use std::process::Command;

// contrato que define as características de um arquivo a ser executado
pub trait Executable {
    // nome que aparece no menu
    fn nome(&self) -> &str;

    // descricao safada sobre o que faz
    fn descricao(&self) -> &str;

    // a magica (que pode dar errado ou não), por isso é result
    fn execute(&self) -> Result<(), String>;

    // permite verificar o tipo concreto em tempo de execução
    fn as_any(&self) -> &dyn std::any::Any;

    // comando principal para execução no PTY
    fn comando(&self) -> &str;

    // argumentos para o comando
    fn args(&self) -> Vec<&str>;
}

// structs pros scripts oficiais em formato RON
#[derive(Debug, Deserialize, Serialize)]
pub struct CompileStep {
    pub compilador: String,
    pub args: Vec<String>,
    pub output: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ExecuteStep {
    pub comando: String,
    pub args: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Script {
    pub nome: String,
    pub descricao: String,
    pub arquivo: String,
    pub compilar: Option<CompileStep>,
    pub executar: ExecuteStep,
}

// implementação genérica de Executable para Script
impl Executable for Script {
    fn nome(&self) -> &str {
        &self.nome
    }

    fn descricao(&self) -> &str {
        &self.descricao
    }

    fn execute(&self) -> Result<(), String> {
        // variáveis que podem ser substituídas nos comandos
        let mut vars = HashMap::new();
        vars.insert("arquivo".to_string(), self.arquivo.clone());

        // se precisa compilar, compila primeiro
        if let Some(compile) = &self.compilar {
            println!("compilando {}...", self.arquivo);

            vars.insert("output".to_string(), compile.output.clone());

            let args: Vec<String> = compile
                .args
                .iter()
                .map(|arg| substituir_vars(arg, &vars))
                .collect();

            let status = Command::new(&compile.compilador)
                .args(&args)
                .status()
                .map_err(|e| format!("falha ao compilar: {}", e))?;

            if !status.success() {
                return Err(format!("compilação de {} falhou", self.arquivo));
            }
        }

        println!("executando {}... pressione ctrl+c para parar.", self.nome);

        // substitui variáveis no comando de execução
        let comando = substituir_vars(&self.executar.comando, &vars);
        let args: Vec<String> = self
            .executar
            .args
            .iter()
            .map(|arg| substituir_vars(arg, &vars))
            .collect();

        Command::new(&comando)
            .args(&args)
            .status()
            .map_err(|e| format!("falha ao executar: {}", e))?;

        Ok(())
    }

    fn comando(&self) -> &str {
        &self.executar.comando
    }

    fn args(&self) -> Vec<&str> {
        self.executar.args.iter().map(|s| s.as_str()).collect()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// mapa de extensão -> (interpretador, descrição)
static INTERPRETERS: &[(&str, &str, &str)] = &[
    ("js", "node", "javascript (eca)"),
    ("py", "python3", "python sssssssssssss"),
    ("rb", "ruby", "ruby da gema"),
    ("sh", "bash", "shell script clássico"),
    (
        "rs",
        "rustc",
        "rust (leia-se: melhor linguagem já inventada)",
    ),
];

pub struct ScriptNaoOficial {
    pub nome: String,
    pub arquivo: String,
    pub interpretador: String,
    pub descricao: String,
}

impl Executable for ScriptNaoOficial {
    fn nome(&self) -> &str {
        &self.nome
    }

    fn descricao(&self) -> &str {
        &self.descricao
    }

    fn execute(&self) -> Result<(), String> {
        println!("executando script não-oficial: {}", self.arquivo);
        println!("pressione ctrl+c para parar.");

        Command::new(&self.interpretador)
            .arg(&self.arquivo)
            .status()
            .map_err(|e| format!("falha ao executar: {}", e))?;

        Ok(())
    }

    fn comando(&self) -> &str {
        &self.interpretador
    }

    fn args(&self) -> Vec<&str> {
        vec![&self.arquivo]
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// função que varre o diretório e detecta scripts
pub fn detectar_scripts() -> Vec<Box<dyn Executable>> {
    let mut executaveis: Vec<Box<dyn Executable>> = Vec::new();
    let mut arquivos_referenciados = std::collections::HashSet::new();

    // 1. carrega scripts oficiais (.ron)
    if let Ok(entries) = fs::read_dir("scripts") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("ron") {
                match fs::read_to_string(&path) {
                    Ok(conteudo) => match ron::from_str::<Script>(&conteudo) {
                        Ok(script) => {
                            println!("script oficial carregado: {}", script.nome);
                            // adiciona o arquivo referenciado ao conjunto pra não duplicar
                            arquivos_referenciados.insert(script.arquivo.clone());
                            executaveis.push(Box::new(script));
                        }
                        Err(e) => eprintln!("erro ao parsear {:?}: {}", path, e),
                    },
                    Err(e) => eprintln!("erro ao ler {:?}: {}", path, e),
                }
            }
        }
    }

    // 2. auto-detecta scripts soltos na raiz
    // 2. auto-detecta scripts soltos na raiz (modo bagunça)
    if let Ok(entries) = fs::read_dir(".") {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                // verifica se tem interpretador conhecido
                if let Some((_, interpretador, desc)) =
                    INTERPRETERS.iter().find(|(e, _, _)| *e == ext)
                {
                    // pega o nome do arquivo pra verificar se já foi referenciado
                    let nome_arquivo = path
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("")
                        .to_string();

                    // só adiciona se não foi referenciado nos scripts .ron
                    if !arquivos_referenciados.contains(&nome_arquivo) {
                        let nome = path
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("desconhecido")
                            .to_string();

                        println!("script não-oficial detectado: {}", nome);

                        executaveis.push(Box::new(ScriptNaoOficial {
                            nome: format!("{} (auto)", nome),
                            arquivo: path.to_string_lossy().to_string(),
                            interpretador: interpretador.to_string(),
                            descricao: desc.to_string(),
                        }));
                    }
                }
            }
        }
    }

    executaveis
}

fn substituir_vars(template: &str, vars: &HashMap<String, String>) -> String {
    let mut resultado = template.to_string();
    for (key, value) in vars {
        resultado = resultado.replace(&format!("{{{}}}", key), value);
    }
    resultado
}
