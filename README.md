# comunism-project

A proposta é simples: **vocês decidem tudo.**  
Conversem, debatam e construam juntos — **stack**, **linguagem**, **tema**, **arquitetura**, **objetivos**… tudo é aberto.  
O único propósito é ver **o que acontece** quando a comunidade cria algo de forma completamente livre.  
Daqui a **30 dias**, o resultado deste experimento vai virar um **vídeo**.

---

## 📂 Estrutura do Projeto

```
comunism-project/
├── 📁 backend/          # API NestJS (porta 5122)
├── 📁 frontend/         # App Next.js (porta 3000)
├── 📁 docker/           # Configurações Docker
├── 📁 docs/             # Documentação completa
├── 📁 scripts/          # Scripts auxiliares
├── 📄 Makefile          # Comandos simplificados
└── 📄 README.md         # Este arquivo
```

---

## Como participar

1. **Faça um fork** deste repositório.  
2. **Modifique a branch `prod`** com suas alterações.  
3. **Abra um Pull Request (PR)**.

**Todo PR será aceito**, desde que:
- Esteja **up-to-date** com a branch `prod`.

---

## 🚀 Como rodar o projeto

### Método 1: Script Automático (Windows)

Simplesmente execute:
```powershell
.\scripts\start.bat
```

O script irá:
- ✅ Verificar se o Make está instalado
- ✅ Configurar o ambiente automaticamente
- ✅ Mostrar um menu interativo

### Método 2: Comandos Make (Recomendado)

1. **Clone o repositório:**
```bash
git clone https://github.com/ricardo11t/comunism-project.git
cd comunism-project
```

2. **Configure o ambiente de desenvolvimento:**
```powershell
make setup-dev
```

3. **Inicie o projeto:**
```powershell
make dev-up
```

4. **Acesse:**
- Frontend: http://localhost:3000
- Backend: http://localhost:5122

### Comandos úteis

- `make help` - Lista todos os comandos disponíveis
- `make logs` - Veja os logs em tempo real
- `make dev-down` - Para o ambiente
- `make clean` - Limpa tudo e recomeça

📖 **Documentação completa:** [docs/DOCKER_SETUP.md](docs/DOCKER_SETUP.md)  
⚡ **Guia rápido:** [docs/QUICK_START.md](docs/QUICK_START.md)  
✅ **Checklist de setup:** [docs/SETUP_CHECKLIST.md](docs/SETUP_CHECKLIST.md)  
📁 **Estrutura do projeto:** [docs/PROJECT_STRUCTURE.md](docs/PROJECT_STRUCTURE.md)

---

## E se quebrar?

> “Ah, mas vai quebrar!” — **Essa é a ideia.**

Este é um experimento sobre **caos criativo**, **colaboração aberta** e **imprevisibilidade**.  
Erros, conflitos e falhas fazem parte do processo.

---

## E se eu quiser fazer maldades? 👿
- Faz parte do experimento social, participe do jeito q vc achar melhor!

## Regras não escritas

- Comuniquem-se por issues, PRs e commits.
- Decidam em conjunto os próximos passos.
- Experimentem sem medo.
- **Divirtam-se.**
