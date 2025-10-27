"use client";
import { GameCard } from "@/components/game-card";
import { Button } from "@/components/ui/button";
import Link from "next/link";
import { useEffect, useState } from "react";
import { jogos, resumeFixture } from "./LogFetcher";

export default function GamesPage() {
  const upcomingGames = [
    {
      id: 2,
      sport: "Brasileirão Série A",
      homeTeam: "Corinthians",
      awayTeam: "São Paulo",
      homeOdds: 1.95,
      awayOdds: 2.05,
      time: "Hoje 19:00",
      isLive: false,
      round: "Rodada 15",
    },
    {
      id: 4,
      sport: "Brasileirão Série A",
      homeTeam: "Botafogo",
      awayTeam: "Vasco",
      homeOdds: 1.7,
      awayOdds: 2.25,
      time: "Hoje 21:30",
      isLive: false,
      round: "Rodada 15",
    },
    {
      id: 6,
      sport: "Brasileirão Série A",
      homeTeam: "Santos",
      awayTeam: "Cruzeiro",
      homeOdds: 1.8,
      awayOdds: 2.1,
      time: "Amanhã 16:00",
      isLive: false,
      round: "Rodada 15",
    },
    {
      id: 7,
      sport: "Brasileirão Série A",
      homeTeam: "Athletico-PR",
      awayTeam: "Bahia",
      homeOdds: 2.0,
      awayOdds: 1.9,
      time: "Amanhã 18:30",
      isLive: false,
      round: "Rodada 15",
    },
    {
      id: 8,
      sport: "Brasileirão Série A",
      homeTeam: "Flamengo",
      awayTeam: "Palmeiras",
      homeOdds: 2.1,
      awayOdds: 1.75,
      time: "Sábado 20:00",
      isLive: false,
      round: "Rodada 16",
    },
    {
      id: 9,
      sport: "Brasileirão Série A",
      homeTeam: "Atlético-MG",
      awayTeam: "Fluminense",
      homeOdds: 1.85,
      awayOdds: 2.15,
      time: "Domingo 16:00",
      isLive: false,
      round: "Rodada 16",
    },
  ];

  const [games, setGames] = useState([] as resumeFixture[]);

  useEffect(() => {
    jogos()
      .then((item) => setGames(item.sort((a, b) => new Date(a.date).getTime() - new Date(b.date).getTime())))
      .catch((err) => {
        alert("olha o erro");
        console.error(err);
      });
  }, []);

  return (
    <div className="min-h-screen bg-background">
      {/* fetch & log fixtures for integration / debugging */}
      {/* <LogFetcher /> */}
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
              <Link href="/games" className="text-sm font-medium text-primary">
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

      <div className="container mx-auto px-4 py-8">
        <div className="mb-8">
          <h1 className="text-4xl md:text-5xl font-bold mb-2">PRÓXIMAS RODADAS</h1>
          <p className="text-muted-foreground">Faça suas apostas nos próximos jogos do Brasileirão</p>
        </div>

        <div className="space-y-8">
          {games.map((item, i) => (
            <div key={i}>
              {/* <h2 className="text-2xl font-bold mb-4">{item.round}</h2> */}
              <div className="space-y-4">
                {/* {upcomingGames
                  .filter((g) => g.round === "Rodada 15")
                  .map((game) => (
                    <GameCard key={game.id} game={game} />
                  ))} */}
                <GameCard game={item} />
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
