# 🚀 Guia Rápido - Comunism Project

## Primeira Vez?

```powershell
# 1. Configure o ambiente
make setup-dev

# 2. Inicie tudo
make dev-up

# 3. Veja os logs
make logs
```

**Pronto!** Acesse http://localhost:3000

---

## Comandos do Dia a Dia

### Iniciar/Parar
```powershell
make dev-up       # Inicia desenvolvimento
make dev-down     # Para desenvolvimento
make dev-restart  # Reinicia serviços
```

### Logs
```powershell
make logs              # Todos os logs
make logs-backend      # Só backend
make logs-frontend     # Só frontend
make logs-db           # Só database
```

### Problemas?
```powershell
make rebuild-dev  # Reconstrói tudo do zero
make clean        # Para e limpa volumes
```

### Produção
```powershell
make setup-prod   # Configura produção
make prod-up      # Inicia produção
make prod-down    # Para produção
```

---

## Estrutura do Projeto

```
comunism-project/
├── backend/              # NestJS API (porta 5122)
│   ├── .env.development.example
│   └── .env.production.example
├── frontend/             # Next.js App (porta 3000)
│   ├── .env.development.example
│   └── .env.production.example
├── docker-compose.dev.yml    # Ambiente dev
├── docker-compose.prod.yml   # Ambiente prod
└── Makefile                  # Comandos simplificados
```

---

## Portas Padrão

| Serviço    | Porta | URL                      |
|------------|-------|--------------------------|
| Frontend   | 3000  | http://localhost:3000    |
| Backend    | 5122  | http://localhost:5122    |
| PostgreSQL | 5432  | localhost:5432           |

---

## Variáveis de Ambiente

Após o `make setup-dev`, edite:
- `backend/.env.development`
- `frontend/.env.development`

Para produção:
- `backend/.env.production`
- `frontend/.env.production`

---

## Problemas Comuns

### Porta em uso?
```powershell
# Para todos os containers primeiro
make dev-down

# Ou mude as portas nos arquivos docker-compose
```

### Mudou o Dockerfile?
```powershell
make rebuild-dev
```

### Quer começar do zero?
```powershell
make clean
make dev-up
```

### Container não inicia?
```powershell
# Veja os logs
make logs

# Se necessário, limpe tudo
make clean-all
make dev-up
```

---

## Ajuda

```powershell
make help  # Lista todos os comandos
```

📖 **Documentação completa:** [DOCKER_SETUP.md](DOCKER_SETUP.md)
