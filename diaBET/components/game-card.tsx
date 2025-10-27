"use client";

import { resumeFixture } from "@/app/games/LogFetcher";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { format } from "date-fns";
import Link from "next/link";

interface Game {
  id: number;
  sport: string;
  homeTeam: string;
  awayTeam: string;
  homeScore?: number;
  awayScore?: number;
  homeOdds: number;
  awayOdds: number;
  time: string;
  isLive: boolean;
}

export function GameCard({ game }: { game: resumeFixture }) {
  return (
    <Card className="p-6 hover:shadow-lg transition-shadow">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-6">
        <div className="flex-1">
          <div className="flex items-center gap-3 mb-4">
            {/* <Badge variant="secondary">{game.sport}</Badge> */}
            <Badge variant="secondary">Brasileirão Série A</Badge>
            {/* {game.isLive && (
              <Badge className="bg-destructive text-destructive-foreground animate-pulse">
                <span className="inline-block w-2 h-2 rounded-full bg-white mr-1.5" />
                AO VIVO
              </Badge>
            )} */}
            <span className="text-sm text-muted-foreground">{format(game.date, "yyyy-MM-dd HH:mm:ss")}</span>
          </div>

          <div className="space-y-3">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-3">
                <div className="w-10 h-10 rounded-full bg-muted flex items-center justify-center font-bold text-sm">
                  <img src={game.teams.home.logo} />
                </div>
                <span className="font-semibold text-lg">{game.teams.home.name}</span>
              </div>
              {/* {game.isLive && game.homeScore !== undefined && (
                <span className="text-3xl font-bold">{game.homeScore}</span>
              )} */}
            </div>

            <div className="flex items-center justify-between">
              <div className="flex items-center gap-3">
                <div className="w-10 h-10 rounded-full bg-muted flex items-center justify-center font-bold text-sm">
                  <img src={game.teams.away.logo} />
                </div>
                <span className="font-semibold text-lg">{game.teams.away.name}</span>
              </div>
              {/* {game.isLive && game.awayScore !== undefined && (
                <span className="text-3xl font-bold">{game.awayScore}</span>
              )} */}
            </div>
          </div>
        </div>

        <div className="flex md:flex-col gap-3 md:min-w-[140px]">
          <Button
            className="flex-1 bg-primary hover:bg-primary/90 h-12"
            onClick={(e) => {
              e.preventDefault();
              // Add to bet slip logic
            }}
          >
            <div className="text-left w-full">
              <div className="text-xs opacity-80">
                <img
                  style={{
                    maxHeight: 40,
                  }}
                  src={game.teams.home.logo}
                />
              </div>
              {/* <div className="text-lg font-bold">{game.homeOdds.toFixed(2)}</div> */}
              <div className="text-lg font-bold">{game.teams.home.name}</div>
            </div>
          </Button>
          <Button
            className="flex-1 bg-primary hover:bg-primary/90 h-12"
            onClick={(e) => {
              e.preventDefault();
              // Add to bet slip logic
            }}
          >
            <div className="text-left w-full">
              <div className="text-xs opacity-80">
                <img
                  style={{
                    maxHeight: 40,
                  }}
                  src={game.teams.away.logo}
                />
              </div>
              <div className="text-lg font-bold">{game.teams.away.name}</div>
            </div>
          </Button>
        </div>

        <Link href={`/games/${game.id}`}>
          <Button variant="outline" className="w-full md:w-auto bg-transparent">
            Ver Mercados
          </Button>
        </Link>
      </div>
    </Card>
  );
}
