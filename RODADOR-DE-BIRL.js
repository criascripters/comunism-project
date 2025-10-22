import magico from "./birlProjects/compiler/magico.js";

const fileName = process.argv[2];

if (!fileName) {
  console.error("Uso: node index.js <nome_do_arquivo>");
  console.error("Exemplo: node index.js hello_world");
  process.exit(1);
}

magico(fileName);
