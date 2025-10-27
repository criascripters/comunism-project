"use client"

import { useState } from "react"
import { Button } from "@/components/ui/button"
import { Card } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { X } from "lucide-react"

interface Bet {
  id: string
  market: string
  selection: string
  odds: number
  team?: string
}

interface BetSlipProps {
  bets: Bet[]
  onRemoveBet: (betId: string) => void
}

export function BetSlip({ bets, onRemoveBet }: BetSlipProps) {
  const [stake, setStake] = useState<number>(10)

  const totalOdds = bets.reduce((acc, bet) => acc * bet.odds, 1)
  const potentialWin = stake * totalOdds

  return (
    <Card className="p-6">
      <div className="flex items-center justify-between mb-6">
        <h3 className="text-xl font-bold">CUPOM DE APOSTAS</h3>
        {bets.length > 0 && (
          <span className="text-sm text-muted-foreground">
            {bets.length} {bets.length === 1 ? "seleção" : "seleções"}
          </span>
        )}
      </div>

      {bets.length === 0 ? (
        <div className="text-center py-12">
          <div className="text-muted-foreground mb-2">Seu cupom está vazio</div>
          <div className="text-sm text-muted-foreground">Clique nas odds para adicionar seleções</div>
        </div>
      ) : (
        <div className="space-y-6">
          <div className="space-y-3">
            {bets.map((bet) => (
              <div key={bet.id} className="p-4 rounded-lg bg-muted relative">
                <button
                  onClick={() => onRemoveBet(bet.id)}
                  className="absolute top-2 right-2 p-1 hover:bg-background rounded-full transition-colors"
                >
                  <X className="h-4 w-4" />
                </button>

                <div className="pr-6">
                  <div className="text-sm text-muted-foreground mb-1">{bet.market}</div>
                  <div className="font-semibold mb-2">{bet.selection}</div>
                  <div className="text-lg font-bold text-primary">{bet.odds.toFixed(2)}</div>
                </div>
              </div>
            ))}
          </div>

          <div className="space-y-4">
            <div>
              <label className="text-sm font-medium mb-2 block">Valor da Aposta</label>
              <div className="relative">
                <span className="absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground">R$</span>
                <Input
                  type="number"
                  value={stake}
                  onChange={(e) => setStake(Number(e.target.value))}
                  className="pl-9 h-12 text-lg"
                  min="1"
                />
              </div>
            </div>

            <div className="flex gap-2">
              {[10, 25, 50, 100].map((amount) => (
                <Button key={amount} variant="outline" size="sm" onClick={() => setStake(amount)} className="flex-1">
                  R${amount}
                </Button>
              ))}
            </div>

            <div className="space-y-2 pt-4 border-t border-border">
              <div className="flex justify-between text-sm">
                <span className="text-muted-foreground">Odds Totais</span>
                <span className="font-bold">{totalOdds.toFixed(2)}</span>
              </div>
              <div className="flex justify-between text-sm">
                <span className="text-muted-foreground">Aposta</span>
                <span className="font-bold">R${stake.toFixed(2)}</span>
              </div>
              <div className="flex justify-between text-lg font-bold pt-2 border-t border-border">
                <span>Ganho Potencial</span>
                <span className="text-primary">R${potentialWin.toFixed(2)}</span>
              </div>
            </div>

            <Button className="w-full h-12 text-lg bg-primary hover:bg-primary/90">Fazer Aposta</Button>
          </div>
        </div>
      )}
    </Card>
  )
}
