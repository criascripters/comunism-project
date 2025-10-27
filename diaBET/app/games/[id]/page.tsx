"use client";

import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { format } from "date-fns";
import { ArrowLeft } from "lucide-react";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { useEffect, useState } from "react";
import { jogo, resumeOnluFixture } from "../LogFetcher";
export default function GameDetailPage({ params }: { params: Promise<{ id: string }> | { id: string } }) {
  const router = useRouter();
  const [homeScore, setHomeScore] = useState<string>("");
  const [awayScore, setAwayScore] = useState<string>("");
  const [betAmount, setBetAmount] = useState<string>("10");

  const [game, setGame] = useState({} as resumeOnluFixture);
  const [id, setId] = useState<string>("");

  useEffect(() => {
    let mounted = true;
    const run = async () => {
      try {
        const p: any = await (params as any);
        const resolvedId: string = p?.id;
        if (!resolvedId) return;
        if (mounted) setId(resolvedId);
        const item = await jogo(resolvedId);
        if (mounted) setGame(item);
      } catch (err) {
        alert("olha o erro");
        console.error(err);
      }
    };
    run();
    return () => {
      mounted = false;
    };
  }, [params]);

  // const game = {
  //   id: params.id,
  //   homeTeam: "Flamengo",
  //   awayTeam: "Palmeiras",
  //   time: "Sábado 20:00",
  //   date: "25/01/2025",
  //   stadium: "Maracanã",
  //   round: "Rodada 16",
  // }

  const handlePlaceBet = () => {
    if (!homeScore || !awayScore || !betAmount) {
      alert("Por favor, preencha todos os campos");
      return;
    }

    const bet = {
      id: Date.now().toString(),
      gameId: id,
      homeTeam: game.teams.home.name,
      awayTeam: game.teams.away.name,
      predictedScore: `${homeScore} x ${awayScore}`,
      amount: Number.parseFloat(betAmount),
      date: new Date().toISOString(),
      gameDate: game.date,
      gameTime: game.date,
      status: "pending",
    };

    const existingBets = JSON.parse(localStorage.getItem("bets") || "[]");
    localStorage.setItem("bets", JSON.stringify([...existingBets, bet]));

    alert("Aposta realizada com sucesso!");
    router.push("/my-bets");
  };

  return (
    <div className="min-h-screen bg-background">
      {/* Header */}
      <header className="border-b border-border bg-card sticky top-0 z-50">
        <div className="container mx-auto px-4 py-4">
          <div className="flex items-center justify-between">
            <Link href="/" className="flex items-center gap-2">
              <div className="h-10 w-10 rounded-lg bg-primary flex items-center justify-center">
                <img src="/mamaco.png" alt="Logo" className="h-6 w-6 text-primary-foreground" />
              </div>
              <span className="text-2xl font-bold tracking-tight">MONKEY BET</span>
            </Link>

            <nav className="hidden md:flex items-center gap-6">
              <Link href="/games" className="text-sm font-medium hover:text-primary transition-colors">
                Próximos Jogos
              </Link>
              <Link href="/my-bets" className="text-sm font-medium hover:text-primary transition-colors">
                Minhas Apostas
              </Link>
              <Link href="#" className="text-sm font-medium hover:text-primary transition-colors">
                Esportes
              </Link>
              <Link href="#" className="text-sm font-medium hover:text-primary transition-colors">
                Promoções
              </Link>
            </nav>

            <div className="flex items-center gap-3">
              <Link href="/auth">
                <Button variant="ghost" size="sm">
                  Entrar
                </Button>
              </Link>
              <Link href="/auth">
                <Button size="sm" className="bg-primary hover:bg-primary/90">
                  Começar
                </Button>
              </Link>
            </div>
          </div>
        </div>
      </header>

      <div className="container mx-auto px-4 py-6">
        <Link href="/games">
          <Button variant="ghost" className="mb-6">
            <ArrowLeft className="h-4 w-4 mr-2" />
            Voltar aos Jogos
          </Button>
        </Link>
        {game.date && (
          <div className="max-w-2xl mx-auto space-y-6">
            <Card className="p-6 bg-secondary text-secondary-foreground">
              <div className="text-center mb-6">
                <div className="text-sm text-secondary-foreground/70 mb-2">{game.round}</div>
                <div className="text-sm text-secondary-foreground/70 mb-1">
                  {format(new Date(game.date), "yyyy-MM-dd HH:mm")}
                </div>
                <div className="text-xs text-secondary-foreground/60">{game.venue}</div>
              </div>

              <div className="space-y-4">
                <div className="flex items-center justify-center gap-4">
                  <div className="text-center flex-1">
                    <div className="w-16 h-16 rounded-full bg-accent flex items-center justify-center text-accent-foreground font-bold mx-auto mb-2">
                      <img
                        src={game.teams.home.logo}
                        style={{
                          maxHeight: 45,
                        }}
                      />
                    </div>
                    <div className="text-xl font-bold">{game.teams.home.name}</div>
                  </div>

                  <div className="text-3xl font-bold text-secondary-foreground/50">VS</div>

                  <div className="text-center flex-1">
                    <div className="w-16 h-16 rounded-full bg-primary flex items-center justify-center text-primary-foreground font-bold mx-auto mb-2">
                      <img
                        src={game.teams.away.logo}
                        style={{
                          maxHeight: 45,
                        }}
                      />
                    </div>
                    <div className="text-xl font-bold">{game.teams.away.name}</div>
                  </div>
                </div>
              </div>
            </Card>

            <Card className="p-6">
              <h2 className="text-2xl font-bold mb-6">FAÇA SUA APOSTA</h2>

              <div className="space-y-6">
                <div>
                  <Label className="text-base font-semibold mb-4 block">Qual será o placar final?</Label>
                  <div className="flex items-center justify-center gap-4">
                    <div className="flex-1">
                      <Label htmlFor="homeScore" className="text-sm mb-2 block text-center">
                        {game.teams.home.name}
                      </Label>
                      <Input
                        id="homeScore"
                        type="number"
                        min="0"
                        max="20"
                        value={homeScore}
                        onChange={(e) => setHomeScore(e.target.value)}
                        className="h-16 text-3xl text-center font-bold"
                        placeholder="0"
                      />
                    </div>

                    <div className="text-3xl font-bold text-muted-foreground pt-8">X</div>

                    <div className="flex-1">
                      <Label htmlFor="awayScore" className="text-sm mb-2 block text-center">
                        {game.teams.away.name}
                      </Label>
                      <Input
                        id="awayScore"
                        type="number"
                        min="0"
                        max="20"
                        value={awayScore}
                        onChange={(e) => setAwayScore(e.target.value)}
                        className="h-16 text-3xl text-center font-bold"
                        placeholder="0"
                      />
                    </div>
                  </div>
                </div>

                <div>
                  <Label htmlFor="betAmount" className="text-base font-semibold mb-2 block">
                    Valor da Aposta
                  </Label>
                  <div className="relative">
                    <span className="absolute left-4 top-1/2 -translate-y-1/2 text-muted-foreground text-lg">R$</span>
                    <Input
                      id="betAmount"
                      type="number"
                      min="1"
                      step="0.01"
                      value={betAmount}
                      onChange={(e) => setBetAmount(e.target.value)}
                      className="pl-12 h-14 text-xl"
                    />
                  </div>
                  <div className="flex gap-2 mt-3">
                    {[10, 25, 50, 100].map((amount) => (
                      <Button
                        key={amount}
                        variant="outline"
                        size="sm"
                        onClick={() => setBetAmount(amount.toString())}
                        className="flex-1"
                      >
                        R${amount}
                      </Button>
                    ))}
                  </div>
                </div>

                <Button onClick={handlePlaceBet} className="w-full h-14 text-lg bg-primary hover:bg-primary/90">
                  Confirmar Aposta
                </Button>
              </div>
            </Card>
          </div>
        )}
      </div>
    </div>
  );
}
