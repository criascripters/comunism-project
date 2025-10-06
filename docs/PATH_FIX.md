# 🔧 Correção de Paths - Docker Compose

## Problema Encontrado

Após mover os arquivos `docker-compose.*.yml` para a pasta `/docker`, os paths relativos pararam de funcionar.

### Erro
```
env file C:\projetos\comunism-project\docker\backend\.env.development not found
```

### Causa
Os arquivos docker-compose estavam usando paths relativos como:
```yaml
context: ./backend
env_file: ./backend/.env.development
volumes: ./backend:/app
```

Mas agora que os arquivos estão em `/docker`, esses paths apontavam para:
- `docker/backend/` (❌ não existe)
- `docker/frontend/` (❌ não existe)

## Solução Aplicada

### Paths Corrigidos
Todos os paths relativos foram ajustados para usar `../` (voltar um nível):

```yaml
# Antes (❌)
context: ./backend
env_file: ./backend/.env.development
volumes: ./backend:/app

# Depois (✅)
context: ../backend
env_file: ../backend/.env.development
volumes: ../backend:/app
```

### Arquivos Atualizados

#### 1. `docker/docker-compose.dev.yml`
```yaml
services:
  backend:
    build:
      context: ../backend          # ✅ Corrigido
    env_file:
      - ../backend/.env.development  # ✅ Corrigido
    volumes:
      - ../backend:/app              # ✅ Corrigido

  frontend:
    build:
      context: ../frontend          # ✅ Corrigido
    env_file:
      - ../frontend/.env.development # ✅ Corrigido
    volumes:
      - ../frontend:/app             # ✅ Corrigido
```

#### 2. `docker/docker-compose.prod.yml`
```yaml
services:
  backend:
    build:
      context: ../backend           # ✅ Corrigido
    env_file:
      - ../backend/.env.production  # ✅ Corrigido

  frontend:
    build:
      context: ../frontend          # ✅ Corrigido
    env_file:
      - ../frontend/.env.production # ✅ Corrigido
```

#### 3. `docker/docker-compose.yml`
```yaml
services:
  backend:
    build:
      context: ../backend  # ✅ Corrigido

  frontend:
    build:
      context: ../frontend # ✅ Corrigido
```

### Bonus: Removida Tag Obsoleta
```yaml
# Removido (obsoleto)
version: '3.8'
```

## Estrutura de Paths

### Visão Geral
```
comunism-project/
├── backend/              ← Aqui estão os .env
│   ├── .env.development
│   └── .env.production
├── frontend/             ← Aqui estão os .env
│   ├── .env.development
│   └── .env.production
└── docker/               ← Aqui estão os docker-compose
    ├── docker-compose.yml
    ├── docker-compose.dev.yml
    └── docker-compose.prod.yml
```

### Path Relativo Explicado
Quando o Docker Compose está em `docker/docker-compose.dev.yml`:
- `./backend` = `docker/backend` (❌ não existe)
- `../backend` = `comunism-project/backend` (✅ correto!)

## Teste e Validação

### Comando Testado
```bash
make dev-up
```

### Resultado
```
✅ [+] Building 63.8s (21/21) FINISHED
✅ Container docker-db-1        Started
✅ Container docker-backend-1   Started
✅ Container docker-frontend-1  Started
```

### Status dos Containers
```bash
make status
```

```
✅ docker-frontend-1   Up 29 seconds
✅ docker-backend-1    Up 29 seconds
✅ docker-db-1         Up 30 seconds
```

## Como Funciona Agora

### 1. Executar da Raiz
```bash
cd C:\projetos\comunism-project
make dev-up
```

### 2. Docker Compose Processa
```bash
docker-compose -f docker/docker-compose.dev.yml up -d
```

### 3. Paths Resolvidos
```
docker/docker-compose.dev.yml localiza:
  ../backend/.env.development   → backend/.env.development ✅
  ../frontend/.env.development  → frontend/.env.development ✅
  ../backend (context)          → backend/ ✅
  ../frontend (context)         → frontend/ ✅
```

## Prevenção de Erros Futuros

### ✅ Sempre Execute da Raiz
```bash
# Correto ✅
cd C:\projetos\comunism-project
make dev-up

# Errado ❌
cd C:\projetos\comunism-project\docker
docker-compose -f docker-compose.dev.yml up
```

### ✅ Use os Comandos Make
Os comandos Make já estão configurados para usar os paths corretos:
```bash
make dev-up      # Já usa docker/docker-compose.dev.yml
make prod-up     # Já usa docker/docker-compose.prod.yml
make logs        # Já usa docker/docker-compose.dev.yml
```

### ✅ Se Mover Arquivos Novamente
Sempre atualize os paths relativos nos docker-compose files:
1. Identifique a nova localização
2. Calcule o path relativo
3. Atualize todos os `context:`, `env_file:` e `volumes:`

## Checklist de Verificação

Após reorganização de pastas, sempre verifique:

- [x] Paths `context:` nos builds
- [x] Paths `env_file:` para variáveis
- [x] Paths `volumes:` para hot-reload
- [x] Comandos Make atualizados
- [x] Documentação atualizada
- [x] Teste com `make dev-up`
- [x] Verifique com `make status`

## Troubleshooting

### Erro: "env file not found"
**Solução:** Verifique os paths relativos no docker-compose

### Erro: "context not found"
**Solução:** Verifique o `context:` no build

### Erro: "version is obsolete"
**Solução:** Remova a linha `version: '3.8'`

## Resultado Final

✅ **Tudo funcionando perfeitamente!**

- Containers iniciam corretamente
- .env files são carregados
- Hot-reload funciona
- Volumes montados corretamente
- Estrutura organizada mantida

---

💡 **Lição Aprendida:** Sempre ajuste paths relativos ao mover arquivos Docker Compose!
