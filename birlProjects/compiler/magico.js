import { exec, spawn } from "child_process";
import fs from "fs";

export default async (fileName) => {
  const data = (await fs.promises.readFile(`./birlProjects/codes_birl/${fileName}.birl`)).toString();
  if (!data) throw new Error("Deu merda");

  const code = birl2C(data); // transpila
  await fs.promises.writeFile(`./birlProjects/transpiled/${fileName}.c`, code); // salva
  await compiler(fileName); // compila

  console.log(`\x1b[42m\x1b[31mrodando ${fileName}.birl\x1b[0m`);

  console.log(`\x1b[31m
⣿⣞⣿⣽⢾⣟⣯⡷⣿⣻⣽⢾⣟⣯⢿⣯⣍⡙⠺⣿⢯⣷⣟⡿⣽⣾⣻⢷⣻⣿
⣿⣞⡿⣞⣯⡿⣞⣿⣳⡿⣽⣻⣾⣯⢿⣞⣿⣽⣦⡀⠉⠛⣾⢿⣽⣞⣿⣻⣽⣾
⣿⣞⣿⣻⣽⣻⢯⣷⡿⠙⠁⠀⠀⢨⣽⣟⣾⣽⢾⡿⣦⡀⠀⠙⣿⣾⣳⢿⣳⣿
⣿⣾⣹⣷⢿⣹⣿⠏⠀⠀⠀⢀⣾⣿⢿⣾⣹⣾⣿⣹⣿⣷⠀ ⠈⣷⣿⢿⣹⣾
⣿⣞⣷⣟⡿⣝⠁⠀⠀⢀⣄⠀⠙⠯⣿⣞⣯⣷⢯⣷⣟⣾⣷⠀⠀⠘⣯⣿⢯⣿
⣿⣞⡿⣞⣿⣻⣷⣦⣶⣿⡿⣷⣆⡀⠈⠻⣷⣯⢿⣳⣯⡷⣿⠀⠀ ⣹⣯⣿⢾
⣿⣞⡿⣯⣷⣟⣷⣻⣽⣾⣻⣽⣻⣷⣄⠀⠈⠙⢿⣽⣳⣿⣻⠀⠀⠀⢼⣟⣾⢿
⣿⣞⣿⣽⡾⣽⡾⣯⢷⣯⡷⣟⣷⢯⣿⣷⣤⡀⠈⠙⢷⣯⠏⠀⠀ ⣾⡿⣽⣻
⣿⢾⣽⣞⣿⣝⠛⠃⣄⠉⠙⠻⢽⢿⣞⣷⣻⢿⣦⡀⠀⠁⠀⠀⠀⣼⣿⣻⣽⢿
⣿⢯⣷⡿⠚⠉⢰⣤⡿⣷⣤⣀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢺⡿⣷⣻⡽⣿
⣿⢯⡇⠁⠀⣸⣿⢯⣿⡽⣯⡿⣿⢷⣶⣤⣤⣤⣤⣤⣶⣾⣦⣀⠀⠀⢽⣿⡽⣿
⣿⣻⢷⣶⣾⣟⣯⣿⣞⣿⣳⡿⣯⡿⣽⢯⡿⣽⣻⣽⣻⢾⡽⣿⣳⣶⡿⣯⢿⣽
\x1b[0m\n\n`);

  await runFile(fileName); // executa
};

const birl2C = (birlCode) => {
  // A tradução é feita com um simples replace no código birl com o seu respectivo valor
  //em C, a regex (?=(?:[^"]|"[^"]*")*$) evita que sejam substituido os valores dentro
  //de aspas.
  var code = birlCode;

  if (code == null) return "";

  //Traduzindo a MAIN
  code = code.replace(/(HORA DO SHOW)(?=(?:[^"]|"[^"]*")*$)/g, "int main (void) {");
  //Traduzindo o BIRL
  code = code.replace(/(BIRL)(?=(?:[^"]|"[^"]*")*$)/g, "}");
  //Traduzindo printf
  code = code.replace(/(CE QUER VER ESSA PORRA[\?]?)(?=(?:[^"]|"[^"]*")*$)/g, "printf");
  //Traduzindo scanf
  code = code.replace(/(QUE QUE CE QUER MONSTR[AÃ]O[\?]?)(?=(?:[^"]|"[^"]*")*$)/g, "scanf");
  //Traduzindo if
  code = code.replace(/(ELE QUE A GENTE QUER[\?]?)(?=(?:[^"]|"[^"]*")*$)(.*)/g, "if $2 {");
  //Traduzindo else
  code = code.replace(/(N[AÃ]O VAI DAR N[AÃ]O)(?=(?:[^"]|"[^"]*")*$)/g, "} else {");
  //Traduzindo else if
  code = code.replace(/(QUE NUM VAI DAR O QUE[\?]?)(?=(?:[^"]|"[^"]*")*$)(.*)/g, "} else if $2 {");
  code = code.replace(/(QUE N[AÃ]O VAI DAR O QUE[\?]?)(?=(?:[^"]|"[^"]*")*$)(.*)/g, "} else if $2 {");
  //Traduzindo while
  code = code.replace(/(NEGATIVA BAMBAM)(?=(?:[^"]|"[^"]*")*$)(.*)/g, "while $2 {");
  //Traduzindo for
  code = code.replace(/(MAIS QUERO MAIS)(?=(?:[^"]|"[^"]*")*$)(.*)/g, "for $2 {");
  //Traduzindo declaração de função
  code = code.replace(/(O[H]? O HOM[EI][M]? A[IÍ] PO[ \t]*\()(?=(?:[^"]|"[^"]*")*$)(.*)(\))/g, "$2 {");
  //Traduzindo retorno da função
  code = code.replace(/(BORA CUMPAD[EI])(?=(?:[^"]|"[^"]*")*$)/g, "return");
  //Traduzindo chamada de função
  code = code.replace(/(AJUDA O MALUCO TA DOENTE)(?=(?:[^"]|"[^"]*")*$)/g, " ");
  code = code.replace(/(AJUDA O MALUCO QUE TA DOENTE)(?=(?:[^"]|"[^"]*")*$)/g, " ");
  //Traduzindo parada no código
  code = code.replace(/(SAI FILH[OA] DA PUTA)(?=(?:[^"]|"[^"]*")*$)/g, "break");
  //Traduzindo continuar o código
  code = code.replace(/(VAMO MONSTRO)(?=(?:[^"]|"[^"]*")*$)/g, "continue");

  //Traduzindo os tipos de dados
  code = code.replace(/(FRANGO)(?=(?:[^"]|"[^"]*")*$)/g, "char");
  code = code.replace(/(MONSTRO)(?=(?:[^"]|"[^"]*")*$)/g, "int");
  code = code.replace(/(MONSTRINHO)(?=(?:[^"]|"[^"]*")*$)/g, "short");
  code = code.replace(/(MONSTR[ÃA]O)(?=(?:[^"]|"[^"]*")*$)/g, "long");
  code = code.replace(/(TRAP[EÉ]ZIO DESCENDENTE)(?=(?:[^"]|"[^"]*")*$)/g, "double");
  code = code.replace(/(TRAP[EÉ]ZIO)(?=(?:[^"]|"[^"]*")*$)/g, "float");
  code = code.replace(/(B[IÍ]CEPS)(?=(?:[^"]|"[^"]*")*$)/g, "unsigned");

  //Colocando as bibliotecas
  code = "#include <stdio.h>\n#include <math.h>\n\n" + code;

  return code;
};

const compiler = (fileName) => {
  return new Promise((resolve, reject) => {
    exec(
      "gcc ./birlProjects/transpiled/" + fileName + ".c -o ./birlProjects/transpiled/" + fileName + " -lm",
      (error) => {
        if (error) {
          reject(new Error("quebrou na hora de compilar: " + error));
        } else {
          resolve();
        }
      }
    );
  });
};

const runFile = (fileName) => {
  return new Promise((resolve, reject) => {
    const child = spawn(`./${fileName}`, { cwd: "./birlProjects/transpiled", stdio: "inherit" });
    // stdio: 'inherit' => conecta ao terminal atual

    child.on("close", (code) => {
      exec(`rm ${fileName} ${fileName}.c`, { cwd: "./birlProjects/transpiled" });
      resolve();
    });

    child.on("error", reject);
  });
};
