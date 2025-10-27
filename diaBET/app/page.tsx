import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Shield, TrendingUp, Trophy, Zap } from "lucide-react";
import Link from "next/link";

export default function HomePage() {
  const featuredGames = [
    {
      id: 2,
      sport: "Brasileirão Série A",
      homeTeam: "Corinthians",
      awayTeam: "São Paulo",
      homeOdds: 1.95,
      awayOdds: 2.05,
      time: "Hoje 19:00",
      isLive: false,
      image: "/soccer-match-action.jpg",
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
      image: "/soccer-match-action.jpg",
    },
    {
      id: 8,
      sport: "Brasileirão Série A",
      homeTeam: "Flamengo",
      awayTeam: "Palmeiras",
      homeOdds: 2.1,
      awayOdds: 1.85,
      time: "Sábado 20:00",
      isLive: false,
      image: "/soccer-match-action.jpg",
    },
  ];

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

      {/* Hero Section */}
      <section className="relative overflow-hidden bg-secondary text-secondary-foreground">
        <div className="absolute inset-0 bg-[url('/sports-stadium-crowd-atmosphere.jpg')] bg-cover bg-center opacity-20" />
        <div className="relative container mx-auto px-4 py-20 md:py-32">
          <div className="max-w-3xl">
            <Badge className="mb-4 bg-accent text-accent-foreground">Aposte no Placar Exato</Badge>
            <h1 className="text-5xl md:text-7xl font-bold mb-6 text-balance">APOSTE NO BRASILEIRÃO</h1>
            <p className="text-lg md:text-xl mb-8 text-secondary-foreground/80 leading-relaxed">
              Preveja o placar exato dos jogos do Brasileirão e ganhe! Apostas simples, pagamentos rápidos e uma
              experiência única.
            </p>
            <div className="flex flex-col sm:flex-row gap-4">
              <Link href="/auth">
                <Button size="lg" className="bg-primary hover:bg-primary/90 text-lg h-12">
                  Começar a Apostar
                </Button>
              </Link>
              <Link href="/games">
                <Button
                  size="lg"
                  variant="outline"
                  className="text-lg h-12 border-secondary-foreground/20 hover:bg-secondary-foreground/10 bg-transparent"
                >
                  Ver Próximos Jogos
                </Button>
              </Link>
            </div>
          </div>
        </div>
      </section>

      {/* Featured Games */}
      <section className="container mx-auto px-4 py-16">
        <div className="flex items-center justify-between mb-8">
          <div>
            <h2 className="text-3xl md:text-4xl font-bold mb-2">JOGOS EM DESTAQUE</h2>
            <p className="text-muted-foreground">Aposte nos próximos jogos do Brasileirão</p>
          </div>
          <Link href="/games">
            <Button variant="outline">Ver Todos os Jogos</Button>
          </Link>
        </div>

        <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-6">
          {featuredGames.map((game) => (
            <Card key={game.id} className="overflow-hidden group hover:shadow-lg transition-shadow">
              <div className="relative h-48 overflow-hidden">
                <img
                  src={game.image || "/placeholder.svg"}
                  alt={`${game.homeTeam} vs ${game.awayTeam}`}
                  className="w-full h-full object-cover group-hover:scale-105 transition-transform duration-300"
                />
                <div className="absolute top-3 left-3">
                  <Badge className="bg-secondary text-secondary-foreground">{game.sport}</Badge>
                </div>
              </div>

              <div className="p-5">
                <div className="text-sm text-muted-foreground mb-3">{game.time}</div>

                <div className="space-y-2 mb-4">
                  <div className="flex items-center justify-between">
                    <span className="font-semibold text-lg">{game.homeTeam}</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="font-semibold text-lg">{game.awayTeam}</span>
                  </div>
                </div>

                <Link href={`/games/${game.id}`}>
                  <Button className="w-full bg-primary hover:bg-primary/90">Apostar no Placar</Button>
                </Link>
              </div>
            </Card>
          ))}
        </div>
      </section>

      {/* Features */}
      <section className="bg-muted py-16">
        <div className="container mx-auto px-4">
          <h2 className="text-3xl md:text-4xl font-bold text-center mb-12">POR QUE ESCOLHER A MONKEY BET</h2>

          <div className="grid md:grid-cols-2 lg:grid-cols-4 gap-8">
            <div className="text-center">
              <div className="inline-flex items-center justify-center w-16 h-16 rounded-full bg-primary text-primary-foreground mb-4">
                <Zap className="h-8 w-8" />
              </div>
              <h3 className="text-xl font-bold mb-2">Apostas Simples</h3>
              <p className="text-muted-foreground leading-relaxed">
                Preveja o placar exato e faça sua aposta de forma rápida e fácil
              </p>
            </div>

            <div className="text-center">
              <div className="inline-flex items-center justify-center w-16 h-16 rounded-full bg-accent text-accent-foreground mb-4">
                <TrendingUp className="h-8 w-8" />
              </div>
              <h3 className="text-xl font-bold mb-2">Grandes Prêmios</h3>
              <p className="text-muted-foreground leading-relaxed">Acerte o placar exato e ganhe prêmios incríveis</p>
            </div>

            <div className="text-center">
              <div className="inline-flex items-center justify-center w-16 h-16 rounded-full bg-primary text-primary-foreground mb-4">
                <Shield className="h-8 w-8" />
              </div>
              <h3 className="text-xl font-bold mb-2">Seguro e Confiável</h3>
              <p className="text-muted-foreground leading-relaxed">
                Seus fundos e dados protegidos com segurança de nível bancário
              </p>
            </div>

            <div className="text-center">
              <div className="inline-flex items-center justify-center w-16 h-16 rounded-full bg-accent text-accent-foreground mb-4">
                <Trophy className="h-8 w-8" />
              </div>
              <h3 className="text-xl font-bold mb-2">Pagamentos Instantâneos</h3>
              <p className="text-muted-foreground leading-relaxed">
                Ganhe e retire seus ganhos instantaneamente, sem espera
              </p>
            </div>
          </div>
        </div>
      </section>

      {/* CTA Section */}
      <section className="bg-secondary text-secondary-foreground py-16">
        <div className="container mx-auto px-4 text-center">
          <h2 className="text-4xl md:text-5xl font-bold mb-4">PRONTO PARA COMEÇAR A GANHAR?</h2>
          <p className="text-xl mb-8 text-secondary-foreground/80">
            Junte-se a milhares de apostadores e experimente a emoção hoje
          </p>
          <Link href="/auth">
            <Button size="lg" className="bg-primary hover:bg-primary/90 text-lg h-14 px-8">
              Criar Conta Grátis
            </Button>
          </Link>
        </div>
      </section>

      {/* Footer */}
      <footer className="border-t border-border bg-card py-12">
        <div className="container mx-auto px-4">
          <div className="grid md:grid-cols-4 gap-8 mb-8">
            <div>
              <div className="flex items-center gap-2 mb-4">
                <div className="h-8 w-8 rounded-lg bg-primary flex items-center justify-center">
                  <Zap className="h-5 w-5 text-primary-foreground" />
                </div>
                <span className="text-xl font-bold">MONKEY BET</span>
              </div>
              <p className="text-sm text-muted-foreground leading-relaxed">
                O destino principal para apostas no Brasileirão com apostas simples de placar e pagamentos instantâneos.
              </p>
            </div>

            <div>
              <h4 className="font-bold mb-4">Esportes</h4>
              <ul className="space-y-2 text-sm text-muted-foreground">
                <li>
                  <Link href="#" className="hover:text-foreground transition-colors">
                    Brasileirão Série A
                  </Link>
                </li>
                <li>
                  <Link href="#" className="hover:text-foreground transition-colors">
                    Copa do Brasil
                  </Link>
                </li>
                <li>
                  <Link href="#" className="hover:text-foreground transition-colors">
                    Libertadores
                  </Link>
                </li>
                <li>
                  <Link href="#" className="hover:text-foreground transition-colors">
                    Champions League
                  </Link>
                </li>
              </ul>
            </div>

            <div>
              <h4 className="font-bold mb-4">Empresa</h4>
              <ul className="space-y-2 text-sm text-muted-foreground">
                <li>
                  <Link href="#" className="hover:text-foreground transition-colors">
                    Sobre Nós
                  </Link>
                </li>
                <li>
                  <Link href="#" className="hover:text-foreground transition-colors">
                    Jogo Responsável
                  </Link>
                </li>
                <li>
                  <Link href="#" className="hover:text-foreground transition-colors">
                    Termos de Serviço
                  </Link>
                </li>
                <li>
                  <Link href="#" className="hover:text-foreground transition-colors">
                    Política de Privacidade
                  </Link>
                </li>
              </ul>
            </div>

            <div>
              <h4 className="font-bold mb-4">Suporte</h4>
              <ul className="space-y-2 text-sm text-muted-foreground">
                <li>
                  <Link href="#" className="hover:text-foreground transition-colors">
                    Central de Ajuda
                  </Link>
                </li>
                <li>
                  <Link href="#" className="hover:text-foreground transition-colors">
                    Fale Conosco
                  </Link>
                </li>
                <li>
                  <Link href="#" className="hover:text-foreground transition-colors">
                    Como Apostar
                  </Link>
                </li>
                <li>
                  <Link href="#" className="hover:text-foreground transition-colors">
                    Perguntas Frequentes
                  </Link>
                </li>
              </ul>
            </div>
          </div>

          <div className="border-t border-border pt-8 text-center text-sm text-muted-foreground">
            <p>© 2025 Monkey Bet. Todos os direitos reservados. Jogue com responsabilidade. +18</p>
          </div>
        </div>
      </footer>
    </div>
  );
}
