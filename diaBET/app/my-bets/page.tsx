"use client";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { format } from "date-fns";
import { Calendar, Clock, TrendingUp } from "lucide-react";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { useEffect, useState } from "react";

interface Bet {
  id: string;
  gameId: string;
  homeTeam: string;
  awayTeam: string;
  predictedScore: string;
  amount: number;
  date: string;
  gameDate: string;
  gameTime: string;
  status: "pending" | "won" | "lost";
}

export default function MyBetsPage() {
  const [bets, setBets] = useState<Bet[]>([]);
  const router = useRouter();
  const [checkingAuth, setCheckingAuth] = useState(true);

  useEffect(() => {
    // verifica se usuário está logado (salvo no localStorage por /auth)
    const user = typeof window !== "undefined" ? localStorage.getItem("user") : null;
    if (!user) {
      router.push("/auth");
      return;
    }

    const storedBets = JSON.parse(localStorage.getItem("bets") || "[]");
    setBets(storedBets);
    setCheckingAuth(false);
  }, [router]);

  const totalBetAmount = bets.reduce((sum, bet) => sum + bet.amount, 0);
  const pendingBets = bets.filter((bet) => bet.status === "pending").length;

  return (
    <div className="min-h-screen bg-background">
      {/* Header */}
      <header className="border-b border-border bg-card sticky top-0 z-50">
        <div className="container mx-auto px-4 py-4">
          <div className="flex items-center justify-between">
            <Link href="/" className="flex items-center gap-2">
              <div className="h-10 w-10 rounded-lg bg-primary flex items-center justify-center">
                {/* <Zap className="h-6 w-6 text-primary-foreground" /> */}
                <img src="/mamaco.png" alt="Logo" className="h-6 w-6 text-primary-foreground" />
              </div>
              <span className="text-2xl font-bold tracking-tight">MONKEY BET</span>
            </Link>

            <nav className="hidden md:flex items-center gap-6">
              <Link href="/games" className="text-sm font-medium hover:text-primary transition-colors">
                Próximos Jogos
              </Link>
              <Link href="/my-bets" className="text-sm font-medium text-primary">
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

      <div className="container mx-auto px-4 py-8">
        <div className="mb-8">
          <h1 className="text-4xl md:text-5xl font-bold mb-2">MINHAS APOSTAS</h1>
          <p className="text-muted-foreground">Acompanhe todas as suas apostas realizadas</p>
        </div>

        {/* Stats Cards */}
        <div className="grid md:grid-cols-3 gap-4 mb-8">
          <Card className="p-6">
            <div className="flex items-center justify-between">
              <div>
                <div className="text-sm text-muted-foreground mb-1">Total Apostado</div>
                <div className="text-3xl font-bold">R${totalBetAmount.toFixed(2)}</div>
              </div>
              <TrendingUp className="h-8 w-8 text-primary" />
            </div>
          </Card>

          <Card className="p-6">
            <div className="flex items-center justify-between">
              <div>
                <div className="text-sm text-muted-foreground mb-1">Apostas Pendentes</div>
                <div className="text-3xl font-bold">{pendingBets}</div>
              </div>
              <Clock className="h-8 w-8 text-yellow-500" />
            </div>
          </Card>

          <Card className="p-6">
            <div className="flex items-center justify-between">
              <div>
                <div className="text-sm text-muted-foreground mb-1">Total de Apostas</div>
                <div className="text-3xl font-bold">{bets.length}</div>
              </div>
              <Calendar className="h-8 w-8 text-blue-500" />
            </div>
          </Card>
        </div>

        {/* Bets List */}
        {bets.length === 0 ? (
          <Card className="p-12">
            <div className="text-center">
              <div className="text-muted-foreground mb-4 text-lg">Você ainda não fez nenhuma aposta</div>
              <Link href="/games">
                <Button className="bg-primary hover:bg-primary/90">Ver Próximos Jogos</Button>
              </Link>
            </div>
          </Card>
        ) : (
          <div className="space-y-4">
            {bets.map((bet) => (
              <Card key={bet.id} className="p-6">
                <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
                  <div className="flex-1">
                    <div className="flex items-center gap-3 mb-3">
                      <Badge
                        variant={
                          bet.status === "pending" ? "secondary" : bet.status === "won" ? "default" : "destructive"
                        }
                      >
                        {bet.status === "pending" ? "Pendente" : bet.status === "won" ? "Ganhou" : "Perdeu"}
                      </Badge>
                      <span className="text-sm text-muted-foreground">
                        {/* {new Date(bet.date).toLocaleDateString("pt-BR")} às{" "}
                        {new Date(bet.date).toLocaleTimeString("pt-BR", {
                          hour: "2-digit",
                          minute: "2-digit",
                        })} */}
                        {format(new Date(bet.date), "dd/MM/yyyy 'às' HH:mm")}
                      </span>
                    </div>

                    <div className="mb-2">
                      <div className="text-xl font-bold mb-1">
                        {bet.homeTeam} vs {bet.awayTeam}
                      </div>
                      <div className="text-sm text-muted-foreground">
                        {bet.gameDate} - {bet.gameTime} <small>vou arrumar não</small>
                      </div>
                    </div>

                    <div className="flex items-center gap-4">
                      <div>
                        <span className="text-sm text-muted-foreground">Placar Previsto: </span>
                        <span className="font-bold text-lg">{bet.predictedScore}</span>
                      </div>
                    </div>
                  </div>

                  <div className="text-right">
                    <div className="text-sm text-muted-foreground mb-1">Valor Apostado</div>
                    <div className="text-2xl font-bold text-primary">R${bet.amount.toFixed(2)}</div>
                  </div>
                </div>
              </Card>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
