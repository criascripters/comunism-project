<p align="center">
<img src="https://i.postimg.cc/YCMHpTVH/aura.gif">
</p>

<h1 align="center">
  o painel do caos
</h1>

<p align="center">
  um lugar onde qualquer código é bem-vindo. se é que dá pra chamar algumas coisas aqui de código.
  <br />
  um r/place, só que com código. todo pull request é aceito automaticamente.
</p>

---

## o que é isso?

é um hub glorificado, feito em rust com uma interface de terminal bonitinha (tui), que detecta e roda qualquer script jogado neste repositório. a gente tem c, javascript, python, rust, e amanhã talvez a gente tenha cobol. quem sabe?

## como rodar

você vai precisar de algumas coisinhas pra ver o circo pegar fogo:

- **rust:** pra rodar o painel em si. (`cargo run`)
- **um compilador c:** (`gcc` ou `clang`) pro nosso amigo comunista.
- **node.js:** pra fofoca sobre o brunisco. (ugh, javascript.)
- **python3 & pip:** pra ver cobras falantes e uma ia local que vai roubar meu emprego.

com tudo isso instalado, é só fazer:

```bash
cargo run
```

e pronto! navegue com as setinhas, aperte `enter` pra executar e `q` pra sair.

## como adicionar sua própria bagunça

existem duas maneiras de contribuir para a anarquia:

### 1. o jeito oficial (oficializado por: eu mesmo)

se você tem um script que precisa de um passo de compilação ou comandos mais complexos, o jeito é criar um arquivo `.ron` na pasta `scripts/`.

olha o `scripts/rosquinha.ron` como exemplo. você define nome, descrição, como compilar e como executar. nosso painel lê isso e adiciona na lista de "oficiais".

### 2. o jeito "perdidos no root"

só joga o seu arquivo `.js`, `.py`, `.sh` ou `.rs` na raiz do projeto.

o painel vai farejar ele, dar um nome automático e adicionar na lista de "perdidos no root". simples assim. sem burocracia.

### 2. o jeito foda-se

teoricamente você pode tacar qualquer coisa nesse repositório. se quiser fazer uma bet vai fundo


---

> feito com amor, caos e uma quantidade questionável de linguagens de programação.
