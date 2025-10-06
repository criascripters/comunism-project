# ✅ Checklist de Setup - Comunism Project

Use este checklist na primeira vez que configurar o projeto.

## Pré-requisitos

- [ ] Git instalado
- [ ] Docker instalado e rodando
- [ ] Docker Compose instalado
- [ ] Make instalado (opcional, mas recomendado)
  - Windows: `choco install make` ou `scoop install make`
  - Linux/Mac: Já vem instalado

## Setup Inicial

### 1. Clone do Projeto
```bash
git clone https://github.com/ricardo11t/comunism-project.git
cd comunism-project
```

- [ ] Repositório clonado
- [ ] Navegado para a pasta do projeto

### 2. Verificação do Ambiente

Execute:
```powershell
make help
```

- [ ] Comando `make` funcionando
- [ ] Lista de comandos exibida

Se o `make` não estiver disponível, você pode usar `.\start.bat` no Windows.

# RECOMENDADO QUE INSTALE O MAKE

### 3. Configuração do Ambiente de Desenvolvimento

Execute:
```powershell
make setup-dev
```

Isso irá criar:
- `backend/.env.development`
- `frontend/.env.development`

- [ ] Comando executado com sucesso
- [ ] Arquivos `.env.development` criados

### 4. Configurar Variáveis de Ambiente

Edite os arquivos criados:

**backend/.env.development:**
```env
NODE_ENV=development
DATABASE_URL="postgresql://postgres:postgres@db:5432/comunism_dev"
PORT=5122

# Adicione suas variáveis aqui
```

**frontend/.env.development:**
```env
NODE_ENV=development
NEXT_PUBLIC_API_URL=http://localhost:5122

# Adicione suas variáveis aqui
```

- [ ] `backend/.env.development` configurado
- [ ] `frontend/.env.development` configurado

### 5. Iniciar o Ambiente

Execute:
```powershell
make dev-up
```

Aguarde enquanto:
- 🔨 Docker builda as imagens
- 📦 Instala as dependências
- 🚀 Inicia os containers

- [ ] Comando executado sem erros
- [ ] Containers iniciados

### 6. Verificar Status

Execute:
```powershell
make status
```

Você deve ver 3 containers rodando:
- `comunism-project-frontend-1`
- `comunism-project-backend-1`
- `comunism-project-db-1`

- [ ] 3 containers com status "Up"

### 7. Testar a Aplicação

Abra no navegador:
- [ ] Frontend: http://localhost:3000
- [ ] Backend: http://localhost:5122

### 8. Verificar Logs

Execute:
```powershell
make logs
```

Pressione `Ctrl+C` para sair.

- [ ] Logs sendo exibidos corretamente
- [ ] Sem erros críticos nos logs

## Comandos Úteis para o Dia a Dia

```powershell
# Ver containers rodando
make status

# Ver logs em tempo real
make logs

# Ver logs de um serviço específico
make logs-backend
make logs-frontend
make logs-db

# Parar o ambiente
make dev-down

# Reiniciar após mudanças
make dev-restart

# Rebuild completo (após mudanças no Dockerfile ou package.json)
make rebuild-dev
```

## Resolução de Problemas

### ❌ Make não encontrado
**Solução:**
```powershell
# Windows com Chocolatey
choco install make

# Windows com Scoop
scoop install make

# Ou use o script alternativo
.\start.bat
```

### ❌ Docker não está rodando
**Solução:**
- Inicie o Docker Desktop
- Aguarde alguns segundos
- Tente novamente

### ❌ Porta 3000 ou 5122 já está em uso
**Solução 1:** Parar o processo usando a porta
```powershell
# Descobrir o processo
netstat -ano | findstr :3000
netstat -ano | findstr :5122

# Matar o processo (substitua PID)
taskkill /PID <PID> /F
```

**Solução 2:** Mudar as portas
- Edite `docker-compose.dev.yml`
- Mude as portas na seção `ports:`

### ❌ Erro ao buildar imagens
**Solução:**
```powershell
# Limpar tudo e começar do zero
make clean-all
make dev-up
```

### ❌ Banco de dados não conecta
**Solução:**
```powershell
# Verificar logs do banco
make logs-db

# Se necessário, remover volumes e recriar
make clean
make dev-up
```

### ❌ Hot-reload não funciona
**Solução:**
- Verifique se os volumes estão montados corretamente
- Execute: `make rebuild-dev`

## Próximos Passos

Agora que tudo está funcionando:

1. 🎨 **Explore a aplicação**
   - Frontend em http://localhost:3000
   - API em http://localhost:5122

2. 📝 **Leia a documentação**
   - [README.md](README.md) - Overview
   - [QUICK_START.md](QUICK_START.md) - Guia rápido
   - [DOCKER_SETUP.md](DOCKER_SETUP.md) - Detalhes do Docker
   - [PROJECT_STRUCTURE.md](PROJECT_STRUCTURE.md) - Estrutura

3. 💻 **Comece a desenvolver**
   - Faça alterações no código
   - Veja as mudanças em tempo real (hot-reload)
   - Use `make logs` para debugar

4. 🚀 **Contribua**
   - Crie uma branch
   - Faça suas modificações
   - Abra um Pull Request

## Checklist Final

Antes de começar a desenvolver, certifique-se:

- [ ] ✅ Todos os pré-requisitos instalados
- [ ] ✅ Repositório clonado
- [ ] ✅ Ambiente configurado (`make setup-dev`)
- [ ] ✅ Variáveis de ambiente ajustadas
- [ ] ✅ Containers rodando (`make dev-up`)
- [ ] ✅ Frontend acessível (http://localhost:3000)
- [ ] ✅ Backend acessível (http://localhost:5122)
- [ ] ✅ Logs sem erros críticos
- [ ] ✅ Hot-reload funcionando

## 🎉 Tudo pronto!

Agora você está pronto para começar a desenvolver!

Se tiver problemas, consulte:
- `make help` - Lista de comandos
- [DOCKER_SETUP.md](DOCKER_SETUP.md) - Troubleshooting detalhado
- Issues do GitHub - Reporte problemas

---

💡 **Dica:** Salve este arquivo e use-o como referência sempre que configurar o projeto em uma nova máquina!
