<div align="center">

# jobRabbit 🐇

**Auto-candidatura a vagas direto do terminal — movida pelo Claude Code + seu Chrome real.**

[English](README.md) · **Português 🇧🇷**

[![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![React](https://img.shields.io/badge/React-20232A?style=for-the-badge&logo=react&logoColor=61DAFB)](https://react.dev/)
[![TypeScript](https://img.shields.io/badge/TypeScript-3178C6?style=for-the-badge&logo=typescript&logoColor=white)](https://www.typescriptlang.org/)
[![Vite](https://img.shields.io/badge/Vite-646CFF?style=for-the-badge&logo=vite&logoColor=white)](https://vitejs.dev/)
[![SQLite](https://img.shields.io/badge/SQLite-07405E?style=for-the-badge&logo=sqlite&logoColor=white)](https://www.sqlite.org/)
[![Docker](https://img.shields.io/badge/Docker-2496ED?style=for-the-badge&logo=docker&logoColor=white)](https://www.docker.com/)
[![Linux](https://img.shields.io/badge/Linux-FCC624?style=for-the-badge&logo=linux&logoColor=black)](https://www.kernel.org/)
[![macOS](https://img.shields.io/badge/macOS-000000?style=for-the-badge&logo=apple&logoColor=white)](https://www.apple.com/macos/)
[![Windows](https://img.shields.io/badge/Windows-0078D6?style=for-the-badge&logo=windows&logoColor=white)](https://www.microsoft.com/windows/)
[![Claude](https://img.shields.io/badge/Claude_Code-D97757?style=for-the-badge&logo=anthropic&logoColor=white)](https://claude.com/claude-code)

[![CI](https://img.shields.io/github/actions/workflow/status/renanmpimentel/jobrabbit/ci.yml?branch=main&style=flat-square&label=CI)](https://github.com/renanmpimentel/jobrabbit/actions/workflows/ci.yml)
[![Licença: MIT](https://img.shields.io/badge/Licen%C3%A7a-MIT-22c55e?style=flat-square)](LICENSE)
[![Testes](https://img.shields.io/badge/testes-115%20passando-22c55e?style=flat-square)](#-validação)
[![i18n](https://img.shields.io/badge/i18n-EN%20%C2%B7%20pt--BR-3178C6?style=flat-square)](#-idiomas--i18n)
[![PRs bem-vindos](https://img.shields.io/badge/PRs-bem--vindos-8b5cf6?style=flat-square)](#-contribuindo)
[![Feito com Claude Code](https://img.shields.io/badge/feito%20com-Claude%20Code-D97757?style=flat-square&logo=anthropic&logoColor=white)](https://claude.com/claude-code)

</div>

---

O jobRabbit orquestra o **Claude Code CLI** (`claude`) que, através da extensão **Claude in
Chrome**, navega em sites de vaga no seu Chrome já logado, avalia o *fit* de cada vaga
contra o seu perfil, gera um CV / carta customizados e tenta se candidatar — pedindo a sua
ajuda (via **alerta no app** e notificação no desktop) quando esbarra em captcha, login ou um
campo que não sabe preencher sozinho.

Ele entrega **dois front-ends a partir de um único binário Rust**: uma **UI web local**
caprichada (o padrão) e uma **interface de terminal** clássica (`--tui`). Inspirado no
[claudia-rh](https://github.com/JohnGabie/claudia-rh) (Windows + Tauri/React); o jobRabbit é
um binário Rust único que roda em Linux, macOS e Windows.

<div align="center">

<img src="dashboard-light.png" width="49%" alt="Dashboard — tema claro"> <img src="dashboard-dark.png" width="49%" alt="Dashboard — tema escuro">
<img src="session-light.png" width="49%" alt="Sessão de execução ao vivo"> <img src="applications-light.png" width="49%" alt="Vagas">

</div>

## ✨ Funcionalidades

| | |
|---|---|
| 🧭 **Dois front-ends** | **UI web** local (padrão, abre no navegador) + **TUI** clássica (`--tui`) — mesmo motor, mesmo SQLite. |
| 🤖 **Agente em navegador real** | Faz spawn do `claude` num PTY, lê o `stream-json` e conduz o **Claude in Chrome** (seu navegador real e logado — sem headless, sem Playwright). |
| 📄 **Import de perfil** | Monte seu perfil a partir de um **currículo** (PDF / DOCX / TXT) ou da sua **URL do LinkedIn** — o agente extrai background, CV base e variantes de busca sugeridas. |
| 🎯 **Avaliação de fit** | Cada vaga recebe nota 0.0–1.0 contra o seu perfil (senioridade, stack, modelo de trabalho, requisitos). |
| 🧩 **Playbooks por ATS** | Detecta **12 plataformas** — Gupy, LinkedIn, Greenhouse, Lever, Workday, Ashby, SmartRecruiters, Indeed, Solides, Vagas.com.br, InfoJobs, inHire (+ fallback genérico) — com receitas por plataforma para o agente saber navegar em cada site. |
| 🗂️ **Banco de respostas** | Respostas de triagem reutilizáveis (pretensão salarial, aviso prévio, modelo de trabalho, …) que o agente usa para preencher formulários; aprende novas no caminho. |
| 🌐 **Fontes de vaga** | Escolha em quais sites o agente busca — 12 plataformas conhecidas já incluídas (LinkedIn, Gupy, Greenhouse, Lever, Workday, Indeed, …), e adicione as suas — no Config. |
| ⚖️ **Modos de candidatura** | `review` (prepara → você aprova), `autonomous` (auto-candidata com fit alto), `hybrid` (auto acima de um limiar). Mais um **dry-run** global e uma trava mestra de **revisão humana** (ligada por padrão) que sempre para para sua aprovação antes de preencher ou enviar — mesmo em autonomous/hybrid. |
| 🔔 **Alertas de pendência ao vivo** | Quando o agente trava (login, captcha, uma pergunta de triagem) você recebe um **toast no app**, um **banner** na tela de Execução e um **contador** na aba Pendências — além da notificação no desktop. A tela de Pendências mostra cada bloqueio com o **contexto completo da vaga inline** (fit, empresa, descrição expansível). |
| 📎 **Upload confiável** | O currículo é enviado como **PDF renderizado do seu CV**, então uploads nunca falham porque o site não aceita `.docx`. |
| 📊 **Avaliador de currículo (ATS)** | Dá nota 0–100 ao seu CV com um relatório acionável e uma **análise de keywords** (presentes vs. faltando, por importância) que você aplica a um CV melhorado em um clique. |
| 🌍 **Bilíngue** | **Inglês por padrão**, **pt-BR** selecionável — idioma da UI *e* do agente. Veja [Idiomas & i18n](#-idiomas--i18n). |

## 🏗️ Arquitetura

- **Front-ends** — uma **UI web** local (React + Vite + Tailwind, servida por um backend Axum) e uma **TUI** (ratatui/crossterm). Ambas compõem o mesmo core.
- **Agente** — o app faz spawn do `claude` num PTY e lê o `--output-format stream-json`. O agente emite um **protocolo NDJSON** (`job` / `application` / `pending` / `answer` / `feedback` / `profile` / `cv_review`) que o app persiste em SQLite. O agente só emite eventos; o DB pertence ao loop da UI.
- **Integrações de desktop** — multiplataforma, resolvidas por SO: notificações (`notify-rust`) no Linux (D-Bus) e macOS (no Windows é um no-op best-effort), keyring (`keyring` v3 — Secret Service / Keychain / Credential Manager), abertura de links (`xdg-open` / `open` / `start`) e descoberta do Chrome. A detecção de idle (`user-idle`, para auto-execução ao ficar ocioso) é só Linux por enquanto.

## 📋 O que você precisa para rodar

O jobRabbit roda em **Linux, macOS e Windows**. Em qualquer SO, ele *controla* duas coisas
que precisam estar no seu desktop:

- **Claude Code CLI** — instalado e autenticado (rode `claude` uma vez para logar).
- **Google Chrome** + a extensão **Claude in Chrome** — instalados e logados.

Isso é tudo que um binário pronto precisa. Buildar do fonte exige, além disso, Rust e Node
(veja *Buildar do fonte* abaixo).

## 🚀 Instalação

### Opção A — Baixar um binário pronto (mais fácil, sem toolchain)

Pegue o arquivo do seu SO no [último release](https://github.com/renanmpimentel/jobrabbit/releases/latest):

| SO | Arquivo |
|----|---------|
| Linux (x86_64) | `jobrabbit-linux-x86_64` |
| macOS (Apple Silicon) | `jobrabbit-macos-arm64` |
| macOS (Intel) | `jobrabbit-macos-x86_64` |
| Windows (x86_64) | `jobrabbit-windows-x86_64.exe` |

**Linux / macOS** — dê permissão de execução e rode:

```bash
chmod +x ./jobrabbit-*        # permite executar
./jobrabbit-*                 # abre a UI web no navegador
```

**Windows** — dê dois cliques no `.exe`, ou pelo PowerShell:

```powershell
.\jobrabbit-windows-x86_64.exe
```

> **Gatekeeper no macOS**: a primeira execução pode ser bloqueada ("desenvolvedor não
> identificado"). Libere em **Ajustes do Sistema → Privacidade e Segurança**, ou rode
> `xattr -d com.apple.quarantine ./jobrabbit-macos-*` uma vez.

### Opção B — Buildar do fonte (qualquer SO)

Exige **Rust** (stable) e **Node 20+**.

```bash
cd web-ui && npm install && npm run build && cd ..   # builda a UI web (fica embutida)
cargo build --release                                # compila o binário
./target/release/jobrabbit                           # rode — jobrabbit.exe no Windows
```

**Só no Linux**, instale antes as libs de D-Bus/X11 (necessárias para linkar notificações,
detecção de idle e a TUI):

```bash
sudo apt install libxcb1 libxss1 libdbus-1-3         # Debian/Ubuntu
```

### Opção C — Docker (hosts Linux, sem Rust/Node)

Num host Linux dá pra buildar dentro do Docker sem instalar Rust nem Node. O binário gerado
é um executável **Linux** (use a Opção A ou B para macOS/Windows):

```bash
make            # testa → builda o bundle web + binário release → abre a UI web
./dist/jobrabbit
```

Por padrão o binário abre a **UI web** no navegador. Prefere a interface de terminal
clássica? Adicione `--tui` a qualquer comando acima.

## 💻 Notas por plataforma

O mesmo binário se adapta a cada SO; alguns opcionais mudam:

| | Linux | macOS | Windows |
|---|:---:|:---:|:---:|
| UI web + agente (núcleo) | ✅ | ✅ | ✅ |
| Notificações de desktop | ✅ (D-Bus) | ✅ | — *(silencioso)* |
| Auto-execução ao ficar ocioso | ✅ | — | — |
| Armazenamento seguro de tokens | Secret Service | Keychain | Credential Manager |
| Pasta de dados & logs | `~/.local/share/jobrabbit/` | `~/Library/Application Support/dev.jobrabbit.jobrabbit/` | `%APPDATA%\jobrabbit\jobrabbit\data\` |

Onde aparece "—", o recurso é simplesmente pulado — todo o resto funciona normalmente.

## 🛠️ Atalhos de dev (Docker, Linux)

```bash
make build        # compila (debug)
make test         # roda os testes
make snapshot     # renderiza as telas da TUI como texto (sem TTY)
make run          # UI web (precisa de claude + Chrome no host)
make tui          # TUI clássica (precisa de TTY)
make web-dev      # Vite dev server (HMR) em :5173, proxy /api → backend
make release      # builda ./dist/jobrabbit para o HOST
make fmt          # formata o código
make shell        # abre um shell no container de dev
```

## 📥 Importar perfil (CV ou LinkedIn)

Em vez de digitar o perfil na mão, importe de um **currículo** (PDF/DOCX/TXT) ou da sua
**URL do LinkedIn** — o `claude` estrutura em background + CV base + variantes de busca
sugeridas.

Na **UI web**: página **Perfil** → *Importar perfil*. Na **TUI**: aba Perfil (`2`) → `m`
(arquivo de CV) ou `l` (URL do LinkedIn).

Ou via CLI (headless):

```bash
jobrabbit --import-cv ~/curriculo.pdf
jobrabbit --import-linkedin https://www.linkedin.com/in/seu-perfil
```

> O background + CV importados **substituem** o perfil atual; as variantes sugeridas são
> **adicionadas** (sem duplicar). O import do LinkedIn navega via Chrome (logado).

## 🌍 Idiomas & i18n

O jobRabbit é **inglês primeiro** e totalmente internacionalizado:

- **UI web** — troque o idioma na página **Config** (English · Português). A escolha é
  persistida e também atualiza o idioma do **agente**, então ele busca e escreve no mesmo
  idioma.
- **TUI** — a aba **Config** tem a configuração **Language** (alterne com `espaço`).
- **Agente** — os prompts e os playbooks de ATS são sensíveis ao idioma. Com pt-BR, o agente
  busca em sites brasileiros e escreve CVs/cartas em português (o comportamento original);
  com inglês, opera em inglês.

As traduções ficam em `web-ui/src/locales/{en,pt-BR}.json` e em `src/locale.rs` +
`src/playbooks/{en,pt-br}/`. Novos idiomas são bem-vindos — veja [Contribuindo](#-contribuindo).

## ✅ Validação

```bash
cargo test                     # 115 testes (parser, DB, protocolo, prompts, sanitize, TUI)
                               #   — ou `make test` para rodar no Docker
jobrabbit --selftest-agent     # E2E real: roda o claude (prompt seguro, sem browsing)
                               #   por todo o pipeline e confere o SQLite
jobrabbit --snapshot           # preview das telas como texto (sem TTY)
jobrabbit --doctor             # diagnóstico do ambiente (deps + config, por SO)
```

> `jobrabbit` aqui é o binário que você instalou — `./jobrabbit-<so>`,
> `./target/release/jobrabbit`, `./dist/jobrabbit`, ou `jobrabbit.exe` no Windows.
> **`--doctor` é o jeito mais rápido de checar seu setup em qualquer SO.**

## 🧰 Solução de problemas

- **"`claude` não encontrado"** — instale o Claude Code e autentique (`claude`). O jobRabbit avisa no startup e ao tentar rodar o agente. Rode `jobrabbit --doctor` para um check completo.
- **Linux: binário não inicia** — instale as libs: `sudo apt install libxcb1 libxss1 libdbus-1-3`.
- **macOS: "desenvolvedor não identificado"** — libere em **Ajustes do Sistema → Privacidade e Segurança**, ou rode `xattr -d com.apple.quarantine ./jobrabbit-macos-*` uma vez.
- **Windows: sem notificações de desktop** — esperado; os toasts são no-op por enquanto. Os alertas in-app (toast + badge no Pending) funcionam em todos os SOs.
- **TUI sem cores / quebrada** — use um terminal moderno (256 cores / UTF-8).
- **Notificações / idle não funcionam (Linux)** — precisam de D-Bus e sessão gráfica (X11/Wayland). São best-effort; o resto do app funciona sem eles.
- **Onde ficam meus dados?** — veja a pasta em [Notas por plataforma](#-notas-por-plataforma) (`jobrabbit.db`, `settings.json`, `jobrabbit.log` e `playbooks/<locale>/<slug>.md`).

## ⚠️ Uso responsável

Automatizar candidaturas pode conflitar com os Termos de Serviço do site. **Verifique os ToS
de cada site de vaga** quanto à automação — o uso é de sua responsabilidade. O jobRabbit
mantém um humano no loop por padrão (modo `review`) e nunca burla captchas.

## 🤝 Contribuindo

PRs são bem-vindos! Boas primeiras contribuições: novos playbooks de ATS, idiomas adicionais
e polimento de UI. Mantenha a suíte de testes verde (`cargo test`, ou `make test` via Docker) e o código somente em inglês.

## 📄 Licença

MIT — veja [LICENSE](LICENSE).

<div align="center">
<sub>Feito com 🐇 e <a href="https://claude.com/claude-code">Claude Code</a>.</sub>
</div>
