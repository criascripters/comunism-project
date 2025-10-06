# 📂 Estrutura do Projeto - Comunism Project# 📁 Estrutura do Projeto - Comunism Project



## Estrutura de Diretórios## Arquivos de Configuração



```### Docker & Compose

comunism-project/```

├── 📁 backend/                      # API NestJSdocker-compose.yml          # Base configuration

│   ├── Dockerfile                   # Multi-stage builddocker-compose.dev.yml      # Development environment

│   ├── .env.development.example     # Template devdocker-compose.prod.yml     # Production environment

│   ├── .env.production.example      # Template prod```

│   ├── prisma/                      # Database schema

│   └── src/                         # Source code### Backend (NestJS)

│```

├── 📁 frontend/                     # App Next.jsbackend/

│   ├── Dockerfile                   # Multi-stage build├── Dockerfile                    # Multi-stage build (dev/prod)

│   ├── .env.development.example     # Template dev├── .env.development.example      # Template para desenvolvimento

│   ├── .env.production.example      # Template prod├── .env.production.example       # Template para produção

│   ├── public/                      # Assets estáticos├── .env.development             # Suas configs de dev (git ignored)

│   └── src/                         # Source code└── .env.production              # Suas configs de prod (git ignored)

│```

├── 📁 docker/                       # ⭐ Configurações Docker

│   ├── docker-compose.yml           # Base configuration### Frontend (Next.js)

│   ├── docker-compose.dev.yml       # Development```

│   └── docker-compose.prod.yml      # Productionfrontend/

│├── Dockerfile                    # Multi-stage build (dev/prod)

├── 📁 docs/                         # ⭐ Documentação├── .env.development.example      # Template para desenvolvimento

│   ├── DOCKER_SETUP.md              # Setup Docker detalhado├── .env.production.example       # Template para produção

│   ├── QUICK_START.md               # Guia rápido├── .env.development             # Suas configs de dev (git ignored)

│   ├── SETUP_CHECKLIST.md           # Checklist de setup└── .env.production              # Suas configs de prod (git ignored)

│   └── PROJECT_STRUCTURE.md         # Estrutura (este arquivo)```

│

├── 📁 scripts/                      # ⭐ Scripts auxiliares## Documentação

│   └── start.bat                    # Script Windows interativo

│| Arquivo | Propósito |

├── 📄 Makefile                      # Comandos simplificados|---------|-----------|

├── 📄 README.md                     # Documentação principal| `README.md` | Overview do projeto e quick start |

└── 📄 .gitignore                    # Arquivos ignorados| `QUICK_START.md` | Guia rápido de comandos |

```| `DOCKER_SETUP.md` | Documentação completa do Docker |

| `PROJECT_STRUCTURE.md` | Este arquivo - estrutura do projeto |

## 🎯 Por Que Esta Estrutura?

## Comandos & Scripts

### Antes (Desorganizado)

```| Arquivo | Propósito |

comunism-project/|---------|-----------|

├── README.md| `Makefile` | Comandos simplificados (make dev-up, etc) |

├── DOCKER_SETUP.md| `start.bat` | Script interativo para Windows |

├── QUICK_START.md

├── PROJECT_STRUCTURE.md## Ambientes

├── SETUP_CHECKLIST.md

├── docker-compose.yml### Development

├── docker-compose.dev.yml- **Comando:** `make dev-up`

├── docker-compose.prod.yml- **Features:** 

├── start.bat  - Hot-reload ativado

├── Makefile  - Volumes montados

├── backend/  - Debug mode

└── frontend/- **Database:** `comunism_dev`

```- **Variáveis:** `.env.development`

❌ Raiz poluída  

❌ Difícil encontrar documentação  ### Production

❌ Configs Docker espalhadas  - **Comando:** `make prod-up`

- **Features:**

### Depois (Organizado)  - Build otimizado

```  - Sem volumes de código

comunism-project/  - Production mode

├── backend/- **Database:** `comunism_prod`

├── frontend/- **Variáveis:** `.env.production`

├── docker/          ⭐ Tudo sobre Docker aqui

├── docs/            ⭐ Toda documentação aqui## Workflow de Desenvolvimento

├── scripts/         ⭐ Todos os scripts aqui

├── Makefile```

└── README.md1. Clone do projeto

```   ↓

✅ Raiz limpa  2. make setup-dev (primeira vez)

✅ Documentação centralizada     ↓

✅ Fácil navegação  3. Edite backend/.env.development e frontend/.env.development

   ↓

## Descrição das Pastas4. make dev-up

   ↓

### 📁 `/docker`5. Desenvolva! (hot-reload automático)

**Propósito:** Centralizar todas as configurações Docker   ↓

6. make logs (para debugar)

```   ↓

docker/7. make dev-down (quando terminar)

├── docker-compose.yml           # Configuração base```

├── docker-compose.dev.yml       # Ambiente desenvolvimento

└── docker-compose.prod.yml      # Ambiente produção## Workflow de Produção

```

```

**Benefícios:**1. Configure variáveis sensíveis

- ✅ Todas as configs Docker em um lugar   ↓

- ✅ Fácil de manter e versionar2. make setup-prod

- ✅ Não polui a raiz do projeto   ↓

3. Edite .env.production com valores seguros

### 📁 `/docs`   ↓

**Propósito:** Centralizar toda a documentação4. make prod-up

   ↓

```5. Teste a aplicação

docs/   ↓

├── DOCKER_SETUP.md             # Guia completo Docker6. make logs-prod (monitorar)

├── QUICK_START.md              # Comandos rápidos```

├── SETUP_CHECKLIST.md          # Checklist para novos devs

└── PROJECT_STRUCTURE.md        # Este arquivo## Comandos Make Principais

```

### Setup

**Benefícios:**```makefile

- ✅ Documentação organizadamake setup-dev      # Primeira configuração (dev)

- ✅ README.md mais limpomake setup-prod     # Primeira configuração (prod)

- ✅ Fácil de expandir```



### 📁 `/scripts`### Operação

**Propósito:** Centralizar scripts auxiliares```makefile

make dev-up         # ▶️  Start development

```make dev-down       # ⏸️  Stop development

scripts/make dev-restart    # 🔄 Restart development

└── start.bat                   # Menu interativo Windowsmake rebuild-dev    # 🔨 Rebuild development

```

make prod-up        # ▶️  Start production

**Benefícios:**make prod-down      # ⏸️  Stop production

- ✅ Scripts não poluem a raizmake prod-restart   # 🔄 Restart production

- ✅ Expansível para novos scriptsmake rebuild-prod   # 🔨 Rebuild production

- ✅ Fácil de encontrar utilitários```



## Comandos Make Atualizados### Monitoramento

```makefile

Todos os comandos Make foram atualizados para usar os novos caminhos:make logs           # 📋 All logs (dev)

make logs-backend   # 📋 Backend logs

```makefilemake logs-frontend  # 📋 Frontend logs

# Antesmake logs-db        # 📋 Database logs

docker-compose -f docker-compose.dev.yml upmake logs-prod      # 📋 All logs (prod)

```

# Depois

docker-compose -f docker/docker-compose.dev.yml up### Manutenção

``````makefile

make clean          # 🧹 Clean volumes

### Comandos Disponíveismake clean-all      # 🧹 Clean everything

```

```bash

# Setup## Portas

make setup-dev      # Configura desenvolvimento

make setup-prod     # Configura produção| Serviço | Porta | Ambiente |

|---------|-------|----------|

# Desenvolvimento| Frontend | 3000 | Dev & Prod |

make dev-up         # Inicia desenvolvimento| Backend | 5122 | Dev & Prod |

make dev-down       # Para desenvolvimento| PostgreSQL | 5432 | Dev & Prod |

make dev-restart    # Reinicia

make rebuild-dev    # Rebuild completo## Volumes Docker



# Produção| Volume | Ambiente | Propósito |

make prod-up        # Inicia produção|--------|----------|-----------|

make prod-down      # Para produção| `db-data-dev` | Development | PostgreSQL data |

make prod-restart   # Reinicia| `db-data-prod` | Production | PostgreSQL data |

make rebuild-prod   # Rebuild completo| `./backend:/app` | Development | Code hot-reload |

| `./frontend:/app` | Development | Code hot-reload |

# Monitoramento

make status         # Status dos containers## GitIgnore Strategy

make logs           # Logs desenvolvimento

make logs-backend   # Logs backend### Versionado (committed):

make logs-frontend  # Logs frontend- ✅ `.env.development.example`

make logs-db        # Logs database- ✅ `.env.production.example`

make logs-prod      # Logs produção- ✅ Dockerfiles

- ✅ docker-compose.*.yml

# Manutenção- ✅ Makefile

make clean          # Limpa volumes

make clean-all      # Limpa tudo### Não Versionado (git ignored):

```- ❌ `.env`

- ❌ `.env.development`

## Scripts Atualizados- ❌ `.env.production`

- ❌ `node_modules/`

### `scripts/start.bat`- ❌ `.next/`

Script interativo para Windows atualizado com novos caminhos.- ❌ `dist/`



```powershell## Resolução de Problemas

# Execute da raiz do projeto

.\scripts\start.bat### Container não inicia?

``````bash

make logs            # Veja os erros

O script automaticamente navega para a raiz antes de executar os comandos Make.make rebuild-dev     # Rebuilde do zero

```

## Documentação Atualizada

### Porta em uso?

Todos os arquivos de documentação foram atualizados com os novos caminhos:```bash

make dev-down        # Para os containers

| Arquivo | Localização | Propósito |# Ou edite as portas nos docker-compose.*.yml

|---------|-------------|-----------|```

| README.md | `/` | Overview e quick start |

| DOCKER_SETUP.md | `/docs/` | Setup Docker detalhado |### Mudou dependências?

| QUICK_START.md | `/docs/` | Guia rápido de comandos |```bash

| SETUP_CHECKLIST.md | `/docs/` | Checklist para setup |make rebuild-dev     # Reinstala node_modules

| PROJECT_STRUCTURE.md | `/docs/` | Este arquivo |```



## Variáveis de Ambiente### Banco de dados corrompido?

```bash

As variáveis de ambiente permanecem nos respectivos diretórios:make clean           # Remove volumes

make dev-up          # Recria tudo

``````

backend/

├── .env.development.example### Quer começar do zero?

├── .env.production.example```bash

├── .env.development         (git ignored)make clean-all       # Remove TUDO

└── .env.production          (git ignored)make setup-dev       # Reconfigura

make dev-up          # Inicia novamente

frontend/```

├── .env.development.example

├── .env.production.example## Próximos Passos

├── .env.development         (git ignored)

└── .env.production          (git ignored)1. ✅ Configure seu ambiente: `make setup-dev`

```2. ✅ Inicie os serviços: `make dev-up`

3. ✅ Verifique os logs: `make logs`

## Portas Padrão4. ✅ Acesse: http://localhost:3000



| Serviço    | Porta | URL                      |---

|------------|-------|--------------------------|

| Frontend   | 3000  | http://localhost:3000    |💡 **Dica:** Execute `make help` a qualquer momento para ver todos os comandos disponíveis!

| Backend    | 5122  | http://localhost:5122    |
| PostgreSQL | 5432  | localhost:5432           |

## GitIgnore

O `.gitignore` permanece na raiz e cobre todo o projeto:

```
# Versionado (✅)
- docker/docker-compose.*.yml
- docs/*.md
- scripts/*.bat
- Dockerfiles
- .env.*.example

# Não versionado (❌)
- .env (valores reais)
- .env.development
- .env.production
- node_modules/
- .next/ e dist/
```

## Migração Completa

### O que foi movido?

```
✅ docker-compose.yml          → docker/docker-compose.yml
✅ docker-compose.dev.yml      → docker/docker-compose.dev.yml
✅ docker-compose.prod.yml     → docker/docker-compose.prod.yml
✅ DOCKER_SETUP.md             → docs/DOCKER_SETUP.md
✅ QUICK_START.md              → docs/QUICK_START.md
✅ SETUP_CHECKLIST.md          → docs/SETUP_CHECKLIST.md
✅ PROJECT_STRUCTURE.md        → docs/PROJECT_STRUCTURE.md
✅ start.bat                   → scripts/start.bat
```

### O que foi atualizado?

```
✅ Makefile                    → Paths atualizados
✅ README.md                   → Links atualizados
✅ scripts/start.bat           → Paths atualizados
```

## Quick Start com Nova Estrutura

### 1. Clone e Setup
```bash
git clone https://github.com/ricardo11t/comunism-project.git
cd comunism-project
make setup-dev
```

### 2. Configure Variáveis
Edite os arquivos:
- `backend/.env.development`
- `frontend/.env.development`

### 3. Inicie o Projeto
```bash
make dev-up
```

### 4. Acesse
- Frontend: http://localhost:3000
- Backend: http://localhost:5122

### 5. Monitore
```bash
make logs
```

## Troubleshooting

### ❌ Comando não encontra docker-compose files
**Problema:** `docker-compose.dev.yml: No such file`

**Solução:**
```bash
# Verifique se está usando o Makefile atualizado
make help

# Ou use o caminho completo manualmente
docker-compose -f docker/docker-compose.dev.yml up
```

### ❌ Script start.bat não funciona
**Problema:** Script não encontra comandos

**Solução:**
```bash
# Execute sempre da raiz do projeto
cd C:\projetos\comunism-project
.\scripts\start.bat
```

### ❌ Links quebrados na documentação
**Problema:** Links antigos não funcionam

**Solução:**
Todos os links foram atualizados. Use os novos caminhos:
- `docs/DOCKER_SETUP.md`
- `docs/QUICK_START.md`
- etc.

## Benefícios da Nova Estrutura

### 🎯 Organização
- Raiz do projeto limpa
- Pastas com propósitos claros
- Fácil localização de arquivos

### 📚 Manutenção
- Documentação centralizada
- Configs Docker isoladas
- Scripts organizados

### 🚀 Escalabilidade
- Fácil adicionar nova documentação
- Simples adicionar novos scripts
- Estrutura preparada para crescer

### 👥 Colaboração
- Novos devs encontram tudo facilmente
- Estrutura intuitiva
- Bem documentado

## Próximos Passos

1. ✅ Execute `make help` para ver todos os comandos
2. ✅ Leia [Quick Start](QUICK_START.md) para começar
3. ✅ Consulte [Docker Setup](DOCKER_SETUP.md) para detalhes
4. ✅ Use [Setup Checklist](SETUP_CHECKLIST.md) na primeira vez

---

💡 **A estrutura agora está profissional, organizada e pronta para escalar!**
