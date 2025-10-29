#!/bin/bash
cols=3
rows=14
pos=2
vel=0.5
score=0

declare -a pista
for ((i=0; i<$rows; i++)); do
  pista[$i]="   "
done

gerar_obstaculo() {
  col=$((RANDOM % cols))
  linha="   "
  linha="${linha:0:$col}X${linha:$((col+1))}"
  echo "$linha"
}

while true; do
  clear
  echo "Use A/D para mover. Pontos: $score"
  echo "▄▀▄▀▄▀▄▀▄▀▄▀"

  for ((i=0; i<$rows-1; i++)); do
    linha="${pista[$i]}"
    echo "| ${linha:0:1} ${linha:1:1} ${linha:2:1} |"
  done

  player="   "
  player="${player:0:$((pos-1))}▲${player:$pos}"
  echo "| ${player:0:1} ${player:1:1} ${player:2:1} |"

  if (( RANDOM % 4 == 0 )); then
    novo=$(gerar_obstaculo)
  else
    novo="   "
  fi
  pista=("$novo" "${pista[@]:0:$(($rows-1))}")

  echo "=================="

  ((score++))

  if (( score % 50 == 0 )); then
    vel=$(awk -v v="$vel" 'BEGIN {v -= 0.1; if (v < 0.01) v = 0.01; printf "%.1f", v}')
  fi

  read -n1 -t "$vel" tecla
  if [[ $tecla == "a" && $pos -gt 1 ]]; then
    ((pos--))
  elif [[ $tecla == "d" && $pos -lt $cols ]]; then
    ((pos++))
  fi

  if [ "${pista[$(($rows-2))]:$((pos-1)):1}" == "X" ]; then
    echo "⛐ Você bateu! Sua pontuação: $score."
    echo "Visite: fernando-online.web.app"
    break
  fi
done