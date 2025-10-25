package main

import (
	"fmt"
	"os"
	"time"
)

const totalFrames = 6572
const frameRate = 15

func main() {
	allFrames := cortadorDeUnha()
	raulSeixas(allFrames)
}

// devolver os arquivos em um array
func cortadorDeUnha() []string {
	frames := []string{}

	for i := 1; i <= totalFrames; i++ {
		content, err := os.ReadFile("./maça_do_mal/frames-ascii/" + fmt.Sprint(i) + ".txt")

		if err != nil {
			fmt.Printf("erro ao ler este frame %v", err)
		}

		frames = append(frames, string(content))
	}
	return frames
}

// controlador de frames
func raulSeixas(musicas []string) {
	for _, musica := range musicas {
		meuGato(musica)
		time.Sleep(100 / frameRate * time.Millisecond)
	}
}

// receber um arquivo e printar na tela
func meuGato(umaMusica string) {
	fmt.Print("\033[H\033[2J")
	fmt.Println(umaMusica)
}
