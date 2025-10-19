package main

import (
	"fmt"
	"math/rand"
	"strings"
	"syscall"
	"time"
	"unsafe"
)

type Point struct {
	x, y int
}

type Game struct {
	snake     []Point
	food      Point
	direction Point
	nextDir   Point
	width     int
	height    int
	score     int
	gameOver  bool
}

type termios struct {
	Iflag  uint32
	Oflag  uint32
	Cflag  uint32
	Lflag  uint32
	Cc     [20]byte
	Ispeed uint32
	Ospeed uint32
}

func enableRawMode() (*termios, error) {
	var oldState termios
	_, _, err := syscall.Syscall(syscall.SYS_IOCTL, uintptr(syscall.Stdin), syscall.TCGETS, uintptr(unsafe.Pointer(&oldState)))
	if err != 0 {
		return nil, err
	}

	newState := oldState
	newState.Lflag &^= syscall.ICANON | syscall.ECHO
	newState.Cc[syscall.VMIN] = 0
	newState.Cc[syscall.VTIME] = 0

	_, _, err = syscall.Syscall(syscall.SYS_IOCTL, uintptr(syscall.Stdin), syscall.TCSETS, uintptr(unsafe.Pointer(&newState)))
	if err != 0 {
		return nil, err
	}

	return &oldState, nil
}

func restoreTerminal(oldState *termios) {
	syscall.Syscall(syscall.SYS_IOCTL, uintptr(syscall.Stdin), syscall.TCSETS, uintptr(unsafe.Pointer(oldState)))
}

func main() {
	rand.Seed(time.Now().UnixNano())

	game := &Game{
		width:     35,
		height:    15,
		snake:     []Point{{17, 7}, {16, 7}, {15, 7}},
		food:      Point{25, 7},
		direction: Point{1, 0},
		nextDir:   Point{1, 0},
		score:     0,
		gameOver:  false,
	}

	oldState, err := enableRawMode()
	if err != nil {
		fmt.Println("erro ao configurar terminal:", err)
		return
	}
	defer restoreTerminal(oldState)

	fmt.Print("\033[?25l") // esconde cursor
	defer fmt.Print("\033[?25h\033[0m") // mostra cursor e reseta cor

	buffer := make([]byte, 3)
	
	game.render()
	
	go func() {
		for {
			n, _ := syscall.Read(syscall.Stdin, buffer)
			if n > 0 {
				switch {
				case buffer[0] == 'w' || buffer[0] == 'W':
					if game.direction.y == 0 {
						game.nextDir = Point{0, -1}
					}
				case buffer[0] == 's' || buffer[0] == 'S':
					if game.direction.y == 0 {
						game.nextDir = Point{0, 1}
					}
				case buffer[0] == 'a' || buffer[0] == 'A':
					if game.direction.x == 0 {
						game.nextDir = Point{-1, 0}
					}
				case buffer[0] == 'd' || buffer[0] == 'D':
					if game.direction.x == 0 {
						game.nextDir = Point{1, 0}
					}
				case buffer[0] == 27 && n == 3:
					if buffer[1] == 91 {
						switch buffer[2] {
						case 65: // up
							if game.direction.y == 0 {
								game.nextDir = Point{0, -1}
							}
						case 66: // down
							if game.direction.y == 0 {
								game.nextDir = Point{0, 1}
							}
						case 68: // left
							if game.direction.x == 0 {
								game.nextDir = Point{-1, 0}
							}
						case 67: // right
							if game.direction.x == 0 {
								game.nextDir = Point{1, 0}
							}
						}
					}
				case buffer[0] == 'q' || buffer[0] == 'Q' || buffer[0] == 27:
					game.gameOver = true
					return
				}
			}
			time.Sleep(10 * time.Millisecond)
		}
	}()

	for !game.gameOver {
		time.Sleep(120 * time.Millisecond)
		game.update()
		game.render()
	}

	game.gameOverScreen()
	time.Sleep(2 * time.Second)
}

func (g *Game) render() {
	// volta pro início sem limpar toda a tela
	fmt.Print("\033[H")
	
	var screen strings.Builder
	
	// borda superior
	screen.WriteString("┌" + strings.Repeat("─", g.width*2) + "┐\n")
	
	for y := 0; y < g.height; y++ {
		screen.WriteString("│")
		for x := 0; x < g.width; x++ {
			pos := Point{x, y}
			
			if pos == g.snake[0] {
				screen.WriteString("\033[31m★\033[0m ")
			} else if contains(g.snake, pos) {
				screen.WriteString("\033[31m█\033[0m ")
			} else if pos == g.food {
				screen.WriteString("\033[33m$\033[0m ")
			} else {
				screen.WriteString("  ")
			}
		}
		screen.WriteString("│\n")
	}
	
	// borda inferior
	screen.WriteString("└" + strings.Repeat("─", g.width*2) + "┘\n")
	screen.WriteString(fmt.Sprintf("\033[31m★ COBRA COMUNISTA ★\033[0m pts:%d | WASD/setas | Q sair\n", g.score))
	
	fmt.Print(screen.String())
}

func (g *Game) update() {
	g.direction = g.nextDir
	
	head := g.snake[0]
	newHead := Point{head.x + g.direction.x, head.y + g.direction.y}

	if newHead.x < 0 || newHead.x >= g.width ||
		newHead.y < 0 || newHead.y >= g.height ||
		contains(g.snake[1:], newHead) {
		g.gameOver = true
		return
	}

	g.snake = append([]Point{newHead}, g.snake...)

	if newHead == g.food {
		g.score++
		g.spawnFood()
	} else {
		g.snake = g.snake[:len(g.snake)-1]
	}
}

func (g *Game) spawnFood() {
	for {
		g.food = Point{
			rand.Intn(g.width),
			rand.Intn(g.height),
		}
		if !contains(g.snake, g.food) {
			break
		}
	}
}

func (g *Game) gameOverScreen() {
	fmt.Print("\033[H")
	
	width := g.width*2 + 2
	fmt.Println("\n" + strings.Repeat("█", width))
	fmt.Println(centerText("GAME OVER COMUNISTA", width))
	fmt.Println(centerText("A LUTA CONTINUA...", width))
	fmt.Printf("%s\n", centerText(fmt.Sprintf("PONTOS: %d", g.score), width))
	fmt.Println(strings.Repeat("█", width) + "\n")
}

func centerText(text string, width int) string {
	padding := (width - len(text)) / 2
	if padding < 0 {
		padding = 0
	}
	return strings.Repeat(" ", padding) + text
}

func contains(slice []Point, item Point) bool {
	for _, a := range slice {
		if a == item {
			return true
		}
	}
	return false
}