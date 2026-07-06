<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="assets/brand/banner-dark.png">
  <img src="assets/brand/banner.png" alt="jobRabbit" width="380">
</picture>

**Candidate-se a vagas pelo terminal — movido a Claude Code + seu navegador.**

[English](README.md) · **Português 🇧🇷**

[![CI](https://img.shields.io/github/actions/workflow/status/renanmpimentel/jobrabbit/ci.yml?branch=main&style=flat-square&label=CI)](https://github.com/renanmpimentel/jobrabbit/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-22c55e?style=flat-square)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-129%20passing-22c55e?style=flat-square)](#-desenvolvedores)
[![Plataformas](https://img.shields.io/badge/Linux%20·%20macOS%20·%20Windows-bin%C3%A1rio%20%C3%BAnico-3178C6?style=flat-square)](#-notas-por-plataforma)
[![i18n](https://img.shields.io/badge/i18n-EN%20·%20pt--BR-8b5cf6?style=flat-square)](#-idioma)

</div>

O jobRabbit conduz a **CLI do Claude Code** com um navegador: navega pelos sites de vaga na sua
sessão *já logada*, pontua o quanto cada vaga combina com o seu perfil, escreve um CV e uma carta
de apresentação sob medida e se candidata por você — parando para perguntar quando esbarra num
captcha, num login ou numa pergunta que não sabe responder. Escolha o backend de navegador: a
extensão **Claude in Chrome** (seu Chrome de verdade) ou **Playwright** (sem extensão). Um único **binário Rust**
roda em Linux, macOS e Windows, com uma **interface web** caprichada (padrão) ou a clássica
**interface de terminal** (`--tui`).

<div align="center">

<img src="dashboard-light.png" width="49%" alt="Dashboard"> <img src="applications-light.png" width="49%" alt="Vagas e candidaturas">
<img src="ats-light.png" width="49%" alt="Avaliação ATS do currículo"> <img src="pending-light.png" width="49%" alt="Pendências com humano no comando">

</div>

## ✨ O que ele faz

- 🤖 **Candidata-se num navegador logado.** Padrão: Claude in Chrome — sua sessão real do Chrome. Ou troque para o backend Playwright: sem extensão, você loga uma vez e o perfil persiste.
- 📄 **Monta seu perfil por você.** Importe de um **currículo** (PDF/DOCX/TXT) ou da sua **URL do LinkedIn**.
- 🎯 **Pontua cada vaga de 0 a 1** contra o seu perfil (senioridade, stack, modelo de trabalho, requisitos).
- 🧩 **Conhece 12 plataformas ATS** — Gupy, LinkedIn, Greenhouse, Lever, Workday, Ashby, SmartRecruiters, Indeed, Solides, Vagas.com.br, InfoJobs, inHire — com receitas por site.
- 🛡️ **Humano no comando por padrão.** Sempre para para sua aprovação antes de preencher ou enviar — mesmo no modo autônomo. Nunca burla captchas.
- 🔔 **Avisa quando trava.** Login, captcha ou pergunta de triagem → alerta no app, badge em Pendências e notificação no desktop, com todo o contexto da vaga junto.
- 📊 **Checker de currículo ATS.** Pontue seu CV de 0 a 100 com um relatório de lacunas de keywords que você aplica em um clique.
- 🌍 **Bilíngue** — inglês e pt-BR, tanto na interface *quanto* no agente; abre no idioma do seu navegador.

## 📋 Requisitos

O jobRabbit *conduz* duas coisas que precisam estar no seu computador:

- **CLI do Claude Code** — instalada e logada (rode `claude` uma vez). Baixe em [claude.com/claude-code](https://claude.com/claude-code). *(Requer um plano pago do Claude.)*
- **Um backend de navegador** (escolha em **Config → Navegador**):
  - **Claude in Chrome** *(padrão)* — Google Chrome + a extensão, instalada e logada.
    Pegue em [claude.com/claude-for-chrome](https://claude.com/claude-for-chrome). Usa o seu
    Chrome de verdade, já logado.
  - **Playwright** *(sem extensão)* — usa o plugin Playwright do Claude Code com um perfil
    persistente. Na primeira execução, faça login nos sites de vaga quando o navegador do
    Playwright pedir (o jobRabbit pausa com uma pendência de *login*); depois disso a sessão
    fica lembrada.

Veja [Backends de navegador](#-backends-de-navegador) para uma comparação lado a lado.

## 🚀 Início rápido

**1. Baixe** o arquivo do seu computador na [última release](https://github.com/renanmpimentel/jobrabbit/releases/latest):

| Seu computador | Arquivo |
|----|------|
| **Windows** | `jobrabbit-windows-x86_64.exe` |
| **Mac** — chip Apple (M1/M2/M3/M4) | `jobrabbit-macos-arm64` |
| **Mac** — chip Intel | `jobrabbit-macos-x86_64` |
| **Linux** | `jobrabbit-linux-x86_64` |

> Não sabe qual Mac?  → **Sobre este Mac**. *"Apple M…"* = chip Apple; *"Intel"* = Intel.

**2. Execute** — uma aba abre no navegador com o dashboard do jobRabbit. Mantenha a janelinha do
terminal/console aberta enquanto usa; fechá-la encerra o jobRabbit.

- **Windows** — dê dois cliques no arquivo. Se aparecer *"O Windows protegeu o seu PC"*, clique em
  **Mais informações → Executar assim mesmo** (o app só ainda não é assinado — não é malware).
- **Mac** — abra o **Terminal** (⌘+Espaço, digite *Terminal*) e cole:
  ```bash
  cd ~/Downloads && chmod +x jobrabbit-macos-* && ./jobrabbit-macos-*
  ```
  Se o macOS disser *"desenvolvedor não identificado"*, vá em **Ajustes do Sistema → Privacidade e
  Segurança**, clique em **Abrir Assim Mesmo** e rode o comando de novo.
- **Linux** — na sua pasta de Downloads:
  ```bash
  chmod +x jobrabbit-linux-x86_64 && ./jobrabbit-linux-x86_64
  ```

> Prefere a interface de terminal clássica? Acrescente `--tui` ao comando.

## 👣 Sua primeira execução

1. **Importe seu perfil** — na página **Perfil**, clique em *Importar perfil* e aponte para seu
   currículo (PDF/DOCX/TXT) ou cole sua URL do LinkedIn. O agente transforma isso em um background,
   um CV base e variantes de busca sugeridas.
2. **Escolha onde buscar** — em **Config**, ative as fontes de vaga que quiser (12 plataformas já
   vêm cadastradas; adicione as suas também).
3. **Rode uma busca** — clique em **Rodar busca**. Deixe o modo de candidatura em **review** na
   primeira vez: o jobRabbit prepara tudo e espera seu OK antes de se candidatar.

> Ou importe direto pela CLI: `jobrabbit --import-cv ~/curriculo.pdf` /
> `jobrabbit --import-linkedin https://www.linkedin.com/in/seu-perfil`.

## 🌐 Backends de navegador

O jobRabbit precisa de um navegador para conduzir. Dois backends são suportados — troque quando
quiser em **Config → Navegador** (web UI) ou na aba **Config** (TUI); a mudança vale para a
próxima execução.

| | **Claude in Chrome** *(padrão)* | **Playwright** |
|---|---|---|
| Extensão necessária | Sim — [Claude in Chrome](https://claude.com/claude-for-chrome) | Não |
| Navegador usado | Seu Google Chrome de verdade | Navegador próprio do Playwright (plugin do Claude Code) |
| Logins | Reusa as sessões que você já tem no Chrome | Logue uma vez; o perfil persistente lembra |
| Detecção de bot | Risco menor (navegador real, sessões reais) | Maior em sites rígidos (LinkedIn, Indeed) |
| Melhor para | Uso no dia a dia — recomendado | Máquinas onde você não pode (ou não quer) instalar a extensão |

**Claude in Chrome (padrão).** O jobRabbit lança `claude --chrome`, que conduz o seu Chrome real,
já logado. Nada mais para configurar: se você consegue navegar no LinkedIn pelo Chrome, o agente
também consegue.

**Playwright (sem extensão).** O jobRabbit deixa o agente usar o plugin Playwright do Claude Code.
O navegador do Playwright mantém um *perfil persistente*: na primeira vez que um site pedir login,
o jobRabbit pausa com uma pendência de **login** — entre uma vez na janela do Playwright, retome, e
as próximas execuções continuam logadas. Captchas e logins nunca são burlados, igual ao modo padrão.

> Arquivos `settings.json` existentes continuam funcionando: o campo legado `use_chrome` é migrado
> automaticamente (`true` → `chrome`, `false` → `playwright`).

## 🌍 Idioma

O jobRabbit fala **inglês** e **português (Brasil)**. A interface web abre no idioma do seu
navegador (inglês nos demais casos) — troque quando quiser na página **Config** (web) ou na aba
**Config** (TUI). A escolha também define o idioma do **agente**, então ele busca nos boards certos
e escreve seu CV e cartas de apresentação nesse idioma.

## 💻 Notas por plataforma

O mesmo binário se adapta a cada SO; alguns extras opcionais mudam:

| | Linux | macOS | Windows |
|---|:---:|:---:|:---:|
| Web UI + agente (núcleo) | ✅ | ✅ | ✅ |
| Notificações no desktop | ✅ (D-Bus) | ✅ | — *(silencioso)* |
| Auto-execução ao ficar ocioso | ✅ | — | — |
| Guarda segura de token | Secret Service | Keychain | Credential Manager |
| Pasta de dados e logs | `~/.local/share/jobrabbit/` | `~/Library/Application Support/dev.jobrabbit.jobrabbit/` | `%APPDATA%\jobrabbit\jobrabbit\data\` |

Onde aparece "—", o recurso é apenas pulado — todo o resto funciona normalmente.
`jobrabbit --doctor` é a forma mais rápida de checar seu ambiente em qualquer SO.

## 🧰 Solução de problemas

- **"`claude` não encontrado"** — instale o Claude Code e faça login (`claude`). Rode `jobrabbit --doctor` para um checkup completo.
- **"port 8787 is already in use" / "another jobRabbit instance is already running"** — você provavelmente deixou um jobRabbit aberto (confira seus terminais). Só uma instância pode usar o banco por vez; para rodar em outra porta use `jobrabbit --port 8788`. `jobrabbit --help` lista todas as flags.
- **Linux: binário não abre** — instale as libs: `sudo apt install libxcb1 libxss1 libdbus-1-3`.
- **macOS: "desenvolvedor não identificado"** — libere em **Ajustes do Sistema → Privacidade e Segurança**, ou rode `xattr -d com.apple.quarantine ./jobrabbit-macos-*` uma vez.
- **Windows: sem notificações no desktop** — esperado; os alertas no app (toast + badge de Pendências) funcionam em todo lugar.
- **TUI quebrada / sem cores** — use um terminal moderno (256 cores / UTF-8).
- **Onde estão meus dados?** — na pasta de [Notas por plataforma](#-notas-por-plataforma) (`jobrabbit.db`, `settings.json`, `jobrabbit.log`, `playbooks/<locale>/`).

## ⚠️ Uso responsável

Automatizar candidaturas pode conflitar com os Termos de Uso de um site. **Verifique os ToS de cada
site de vaga** sobre automação — você é responsável por como usa esta ferramenta. O jobRabbit mantém
um humano no comando por padrão (modo `review`) e nunca burla captchas.

## 🔧 Desenvolvedores

<details>
<summary>Build do código, Docker, atalhos de dev, arquitetura e testes</summary>

### Build do código (qualquer SO)

Requer **Rust** (stable, via [rustup.rs](https://rustup.rs)) e **Node 20+**.

```bash
cd web-ui && npm install && npm run build && cd ..   # build da web UI (é embutida)
cargo build --release                                # compila o binário
./target/release/jobrabbit                           # roda — jobrabbit.exe no Windows
```

Só no **Linux**, instale as libs D-Bus/X11 antes (notificações, idle, TUI):

```bash
sudo apt install libxcb1 libxss1 libdbus-1-3         # Debian/Ubuntu
```

### Docker (hosts Linux, sem Rust/Node)

Constrói um binário **Linux** dentro do Docker (use *Build do código* para macOS/Windows):

```bash
make            # build do bundle web + binário release → abre a web UI
make app        # o mesmo, mas roda a suíte de testes antes
./dist/jobrabbit
```

> O Docker disse *"all predefined address pools have been fully subnetted"*? Sua máquina tem
> redes compose antigas demais — rode `docker network prune` (os serviços daqui usam a bridge
> padrão, então checkouts novos não criam rede própria).

### Atalhos de dev (Docker)

```bash
make build        # compila (debug)           make web-dev   # Vite dev server (HMR) em :5173
make test         # roda os testes            make release   # build de ./dist/jobrabbit pro HOST
make snapshot     # renderiza a TUI em texto  make fmt       # formata o código
make run          # web UI (precisa de claude + um backend de navegador no host)
make tui          # TUI clássica (precisa de TTY)
```

### Arquitetura

- **Front-ends** — uma **web UI** local (React + Vite + Tailwind, servida por um backend Axum) e uma **TUI** (ratatui/crossterm). Ambas compõem o mesmo core.
- **Agente** — o app sobe `claude` num PTY e lê `--output-format stream-json`. O agente emite um **protocolo NDJSON** (`job` / `application` / `pending` / `answer` / `feedback` / `profile` / `cv_review`) que o app persiste no SQLite. O agente só emite eventos; o banco é do loop da UI.
- **Integrações de desktop** — resolvidas por SO: notificações (`notify-rust`, Linux D-Bus / macOS), keyring (`keyring` v3), abertura de links (`xdg-open` / `open` / `start`) e descoberta do Chrome. Idle (`user-idle`) é só Linux.
- **i18n** — strings da UI em `web-ui/src/locales/{en,pt-BR}.json`; prompts/playbooks do agente em `src/locale.rs` + `src/playbooks/{en,pt-br}/`.

### Testes e diagnóstico

```bash
cargo test                     # 129 testes (parser, DB, protocolo, prompts, sanitize, CLI, TUI) — ou `make test`
jobrabbit --selftest-agent     # E2E real: roda o claude (prompt seguro, sem navegar) pelo pipeline
jobrabbit --snapshot           # preview das telas da TUI em texto (sem TTY)
jobrabbit --doctor             # diagnóstico de ambiente (deps + config, por SO)
```

### Contribuindo

PRs são bem-vindos! Boas primeiras contribuições: novos playbooks ATS, novos idiomas e polimento de
UI. Mantenha os testes verdes (`cargo test` / `make test`) e o código em inglês.

Inspirado no [claudia-rh](https://github.com/JohnGabie/claudia-rh) (Windows + Tauri/React).

</details>

## 📄 Licença

MIT — veja [LICENSE](LICENSE).

<div align="center">
<sub>Feito com 🐇 e <a href="https://claude.com/claude-code">Claude Code</a>.</sub>
</div>
