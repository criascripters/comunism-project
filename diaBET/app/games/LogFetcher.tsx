"use client";

// export default function LogFetcher() {
//   useEffect(() => {
//     const qs = new URLSearchParams({
//       league: "71",
//       season: "2025",
//       from: "2025-10-25",
//       to: "2025-10-31",
//     });

//     fetch(`/api/bet?${qs.toString()}`)
//       .then((r) => r.json())
//       .then((json) => {
//         // log para integração futura
//         console.log("[api/bet] fixtures response:", json);
//       })
//       .catch((err) => console.error("[api/bet] fetch error:", err));
//   }, []);

//   return null;
// }

export const jogos = (): Promise<resumeFixture[]> => {
  const data = new Date();
  // usa ISO YYYY-MM-DD; adiciona 10 dias para o intervalo
  const hoje = data.toISOString().slice(0, 10);
  const dezDiasDate = new Date(data);
  dezDiasDate.setDate(data.getDate() + 10);
  const dezDias = dezDiasDate.toISOString().slice(0, 10);

  const qs = new URLSearchParams({
    league: "71",
    season: "2025",
    from: hoje,
    to: dezDias,
  });

  return fetch(`/api/bet?${qs.toString()}`)
    .then((r) => r.json())
    .then((r) => r.data)
    .then((json: Ifixture) => {
      console.log(json);
      return json.response.map((item) => {
        return {
          id: String(item.fixture.id),
          status: item.fixture.status.long,
          teams: {
            away: {
              logo: item.teams.away.logo,
              name: item.teams.away.name,
            },
            home: {
              logo: item.teams.home.logo,
              name: item.teams.home.name,
            },
          },
          goals: {
            home: item.goals.home ?? 0,
            away: item.goals.away ?? 0,
          },
          round: item.league.round,
          date: item.fixture.date,
        };
      });
    })
    .catch((err) => {
      console.error("[api/bet] fetch error:", err);
      return [];
    });
};

export const jogo = (id: string): Promise<resumeOnluFixture> => {
  const EMPTY_FIXTURE: resumeOnluFixture = {
    date: "",
    venue: "",
    round: "",
    teams: {
      home: {
        name: "",
        logo: "",
        winner: false,
      },
      away: {
        name: "",
        logo: "",
        winner: false,
      },
    },
  };

  return fetch(`/api/bet?id=${id}`)
    .then((r) => r.json())
    .then((r) => r.data)
    .then((json: IOnlyFix) => {
      const a = json.response.map((item) => {
        return {
          date: item.fixture.date,
          venue: item.fixture.venue.name,
          round: item.league.round,
          teams: {
            away: {
              logo: item.teams.away.logo,
              name: item.teams.away.name,
              winner: item.teams.away.winner ?? false,
            },
            home: {
              logo: item.teams.home.logo,
              name: item.teams.home.name,
              winner: item.teams.home.winner ?? false,
            },
          },
        } as resumeOnluFixture;
      });
      if (a.length > 0) {
        return a[0];
      }
      return EMPTY_FIXTURE;
    })
    .catch((err) => {
      console.error("[api/bet] fetch error:", err);
      return EMPTY_FIXTURE;
    });
};

export type resumeFixture = {
  id: string;
  status: string;
  teams: {
    home: {
      name: string;
      logo: string;
    };
    away: {
      name: string;
      logo: string;
    };
  };
  goals: {
    home: number;
    away: number;
  };
  round: string;
  date: string;
};

export type resumeOnluFixture = {
  date: string;
  venue: string; // estádio
  round: string;
  teams: {
    home: {
      name: string;
      logo: string;
      winner: boolean;
    };
    away: {
      name: string;
      logo: string;
      winner: boolean;
    };
  };
};

export type Ifixture = {
  get: string;
  parameters: {
    league: string;
    season: string;
    from: string;
    to: string;
  };
  errors: Array<any>;
  results: number;
  paging: {
    current: number;
    total: number;
  };
  response: Array<{
    fixture: {
      id: number;
      referee?: string;
      timezone: string;
      date: string;
      timestamp: number;
      periods: {
        first?: number;
        second?: number;
      };
      venue: {
        id?: number;
        name: string;
        city: string;
      };
      status: {
        long: string;
        short: string;
        elapsed?: number;
        extra?: number;
      };
    };
    league: {
      id: number;
      name: string;
      country: string;
      logo: string;
      flag: string;
      season: number;
      round: string;
      standings: boolean;
    };
    teams: {
      home: {
        id: number;
        name: string;
        logo: string;
        winner?: boolean;
      };
      away: {
        id: number;
        name: string;
        logo: string;
        winner?: boolean;
      };
    };
    goals: {
      home?: number;
      away?: number;
    };
    score: {
      halftime: {
        home?: number;
        away?: number;
      };
      fulltime: {
        home?: number;
        away?: number;
      };
      extratime: {
        home: any;
        away: any;
      };
      penalty: {
        home: any;
        away: any;
      };
    };
  }>;
};

export type IOnlyFix = {
  get: string;
  parameters: {
    id: string;
  };
  errors: Array<any>;
  results: number;
  paging: {
    current: number;
    total: number;
  };
  response: Array<{
    fixture: {
      id: number;
      referee: string;
      timezone: string;
      date: string;
      timestamp: number;
      periods: {
        first: number;
        second: number;
      };
      venue: {
        id: any;
        name: string;
        city: string;
      };
      status: {
        long: string;
        short: string;
        elapsed: number;
        extra: number;
      };
    };
    league: {
      id: number;
      name: string;
      country: string;
      logo: string;
      flag: string;
      season: number;
      round: string;
      standings: boolean;
    };
    teams: {
      home: {
        id: number;
        name: string;
        logo: string;
        winner: boolean;
      };
      away: {
        id: number;
        name: string;
        logo: string;
        winner: boolean;
      };
    };
    goals: {
      home: number;
      away: number;
    };
    score: {
      halftime: {
        home: number;
        away: number;
      };
      fulltime: {
        home: number;
        away: number;
      };
      extratime: {
        home: any;
        away: any;
      };
      penalty: {
        home: any;
        away: any;
      };
    };
    events: Array<{
      time: {
        elapsed: number;
        extra?: number;
      };
      team: {
        id: number;
        name: string;
        logo: string;
      };
      player: {
        id: number;
        name: string;
      };
      assist: {
        id?: number;
        name?: string;
      };
      type: string;
      detail: string;
      comments?: string;
    }>;
    lineups: Array<{
      team: {
        id: number;
        name: string;
        logo: string;
        colors: {
          player: {
            primary: string;
            number: string;
            border: string;
          };
          goalkeeper: {
            primary: string;
            number: string;
            border: string;
          };
        };
      };
      formation: string;
      startXI: Array<{
        player: {
          id: number;
          name: string;
          number: number;
          pos: string;
          grid: string;
        };
      }>;
      substitutes: Array<{
        player: {
          id: number;
          name: string;
          number: number;
          pos: string;
          grid: any;
        };
      }>;
      coach: {
        id: number;
        name: string;
        photo: string;
      };
    }>;
    statistics: Array<{
      team: {
        id: number;
        name: string;
        logo: string;
      };
      statistics: Array<{
        type: string;
        value: any;
      }>;
    }>;
    players: Array<{
      team: {
        id: number;
        name: string;
        logo: string;
        update: string;
      };
      players: Array<{
        player: {
          id: number;
          name: string;
          photo: string;
        };
        statistics: Array<{
          games: {
            minutes?: number;
            number: number;
            position: string;
            rating?: string;
            captain: boolean;
            substitute: boolean;
          };
          offsides?: number;
          shots: {
            total?: number;
            on?: number;
          };
          goals: {
            total?: number;
            conceded: number;
            assists?: number;
            saves?: number;
          };
          passes: {
            total?: number;
            key?: number;
            accuracy?: string;
          };
          tackles: {
            total?: number;
            blocks?: number;
            interceptions?: number;
          };
          duels: {
            total?: number;
            won?: number;
          };
          dribbles: {
            attempts?: number;
            success?: number;
            past?: number;
          };
          fouls: {
            drawn?: number;
            committed?: number;
          };
          cards: {
            yellow: number;
            red: number;
          };
          penalty: {
            won: any;
            commited: any;
            scored: number;
            missed: number;
            saved?: number;
          };
        }>;
      }>;
    }>;
  }>;
};
