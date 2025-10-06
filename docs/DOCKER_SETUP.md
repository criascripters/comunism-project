# Comunism Project - Docker Environment Setup

Este projeto está configurado para rodar em dois ambientes distintos: **Desenvolvimento** e **Produção**.

## 🚀 Quick Start (Recomendado)

### Primeira vez? Use o Make!

1. **Setup do ambiente de desenvolvimento:**
```powershell
make setup-dev
```

2. **Inicie o ambiente:**
```powershell
make dev-up
```

3. **Veja os logs:**
```powershell
make logs
```

Pronto! Seu ambiente está rodando em:
- Frontend: http://localhost:3000
- Backend: http://localhost:5122
- Database: localhost:5432

### Lista completa de comandos Make:
```powershell
make help
```

## 📋 Comandos Make Disponíveis

### Setup
- `make setup-dev` - Configura o ambiente de desenvolvimento
- `make setup-prod` - Configura o ambiente de produção

### Desenvolvimento
- `make dev-up` - Inicia o ambiente de desenvolvimento
- `make dev-down` - Para o ambiente de desenvolvimento
- `make dev-restart` - Reinicia o ambiente de desenvolvimento
- `make rebuild-dev` - Reconstrói e inicia o ambiente (use após mudanças no Dockerfile)

### Produção
- `make prod-up` - Inicia o ambiente de produção
- `make prod-down` - Para o ambiente de produção
- `make prod-restart` - Reinicia o ambiente de produção
- `make rebuild-prod` - Reconstrói e inicia o ambiente (use após mudanças no Dockerfile)

### Logs
- `make logs` - Mostra todos os logs (desenvolvimento)
- `make logs-backend` - Mostra logs do backend
- `make logs-frontend` - Mostra logs do frontend
- `make logs-db` - Mostra logs do banco de dados
- `make logs-prod` - Mostra todos os logs (produção)

### Manutenção
- `make clean` - Para todos os containers e remove volumes
- `make clean-all` - Limpa tudo incluindo imagens Docker

## 🎯 Ambientes Disponíveis

### Desenvolvimento (Development)
- Hot-reload ativado
- Volumes montados para desenvolvimento em tempo real
- Logs verbosos
- Banco de dados: `comunism_dev`

### Produção (Production)
- Build otimizado
- Sem volumes de desenvolvimento
- Restart automático
- Banco de dados: `comunism_prod`

## 📋 Pré-requisitos

- Docker
- Docker Compose

## 🔧 Como Usar (Método Manual)

Se preferir não usar o Makefile, você pode usar os comandos docker-compose diretamente:

### Ambiente de Desenvolvimento

Para iniciar o ambiente de desenvolvimento:

```powershell
docker-compose -f docker-compose.dev.yml up --build
```

Ou com o comando curto:

```powershell
docker-compose -f docker-compose.dev.yml up
```

Para rodar em background:

```powershell
docker-compose -f docker-compose.dev.yml up -d
```

### Ambiente de Produção

Para iniciar o ambiente de produção:

```powershell
docker-compose -f docker-compose.prod.yml up --build
```

Ou com o comando curto:

```powershell
docker-compose -f docker-compose.prod.yml up
```

Para rodar em background:

```powershell
docker-compose -f docker-compose.prod.yml up -d
```

## 🛑 Parar os Serviços

### Desenvolvimento
```powershell
docker-compose -f docker-compose.dev.yml down
```

### Produção
```powershell
docker-compose -f docker-compose.prod.yml down
```

Para remover também os volumes:
```powershell
docker-compose -f docker-compose.dev.yml down -v
```

## 📁 Arquivos de Configuração

### Frontend
- `.env.development` - Variáveis de ambiente para desenvolvimento
- `.env.production` - Variáveis de ambiente para produção

### Backend
- `.env.development` - Variáveis de ambiente para desenvolvimento
- `.env.production` - Variáveis de ambiente para produção

## 🔒 Variáveis de Ambiente

Certifique-se de configurar as variáveis de ambiente adequadas em cada arquivo `.env`:

### Backend
- `NODE_ENV` - Ambiente (development/production)
- `DATABASE_URL` - URL de conexão com o banco de dados
- `PORT` - Porta do servidor backend

### Frontend
- `NODE_ENV` - Ambiente (development/production)
- `NEXT_PUBLIC_API_URL` - URL da API backend

## 📦 Serviços

### Backend
- **Porta:** 5122
- **Framework:** NestJS
- **Desenvolvimento:** Hot-reload com `npm run start:dev`
- **Produção:** Build otimizado com `npm run start:prod`

### Frontend
- **Porta:** 3000
- **Framework:** Next.js
- **Desenvolvimento:** Hot-reload com `npm run dev`
- **Produção:** Build otimizado com `npm run start`

### Database
- **Porta:** 5432
- **Sistema:** PostgreSQL 15
- **Desenvolvimento:** Volume `db-data-dev`
- **Produção:** Volume `db-data-prod`

## 🔄 Rebuild dos Containers

Se você fizer mudanças nos Dockerfiles ou nas dependências:

```powershell
# Desenvolvimento
docker-compose -f docker-compose.dev.yml up --build

# Produção
docker-compose -f docker-compose.prod.yml up --build
```

## 📝 Logs

Para ver os logs dos containers:

```powershell
# Desenvolvimento
docker-compose -f docker-compose.dev.yml logs -f

# Produção
docker-compose -f docker-compose.prod.yml logs -f
```

Para ver logs de um serviço específico:

```powershell
docker-compose -f docker-compose.dev.yml logs -f backend
docker-compose -f docker-compose.dev.yml logs -f frontend
docker-compose -f docker-compose.dev.yml logs -f db
```

## 🐛 Troubleshooting

### Porta já em uso
Se você receber um erro de porta já em uso, pare os containers existentes ou mude as portas nos arquivos `docker-compose.*.yml`.

### Volumes corrompidos
Para limpar completamente e começar do zero:

```powershell
docker-compose -f docker-compose.dev.yml down -v
docker system prune -a
```

### Rebuild completo
```powershell
docker-compose -f docker-compose.dev.yml down
docker-compose -f docker-compose.dev.yml build --no-cache
docker-compose -f docker-compose.dev.yml up
```
