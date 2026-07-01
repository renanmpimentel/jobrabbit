# jobRabbit — Design System

## Princípio
Interface clara, densa mas respirável, estilo Notion/Height. O conteúdo é
o herói, não o cromo. Superfícies quase brancas, bordas finas, tipografia
com hierarquia real por tamanho/peso, cor usada com parcimônia e sempre
com significado (score, status, ação).

## Cor (tokens — light)
- bg/base:        #FFFFFF
- bg/subtle:      #FAFAF9   (fundo da app, sidebar)
- bg/muted:       #F5F5F4   (hover de linha, blocos secundários)
- border/subtle:  #EAEAE8
- border/default: #E0E0DD
- text/primary:   #1A1A18
- text/secondary: #6B6B66
- text/tertiary:  #9A9A94   (metadados, timestamps, "linkedin")
- accent:         #4F46E5   (ação primária, um único tom)
- accent/hover:   #4338CA

## Cor semântica de SCORE (escala contínua, não binária)
Aplicar como faixa de fundo sutil no item + cor do número.
- >= 0.75  forte    fundo #ECFDF3  número #067647  (verde)
- 0.55–0.74 médio   fundo #FEFBEB  número #B54708  (âmbar)
- < 0.55   fraco    fundo #FEF3F2  número #B42318  (vermelho)
Nunca só cor: sempre número + faixa, para acessibilidade.

## Status semântico (badges de Pendências)
- Bloqueante (Pergunta, Login): fundo #FEF3F2, texto #B42318, borda vermelha
- Decisão:    fundo #EFF4FF, texto #175CD3
- Rotina:     fundo #F5F5F4, texto #6B6B66 (neutro, discreto)

## Tipografia
- Font: Inter (UI). Mono só para logs/código (JetBrains Mono ou ui-monospace).
- Escala: 12 / 13 / 14(base) / 16 / 20 / 28
- Título de página: 20 semibold
- Título de card/seção: 14 semibold
- Corpo: 14 regular, text/primary
- Metadado: 13 regular, text/tertiary
- Peso: só 400, 500, 600. Nada de 700+.

## Espaçamento (grid de 4)
4 / 8 / 12 / 16 / 24 / 32 / 48
- Padding de card: 24
- Gap entre itens de lista: 0 (linhas com borda), ou 8 (cards soltos)
- Padding de linha de lista: 12 vertical / 16 horizontal

## Raio e sombra
- radius/sm: 6   radius/md: 8   radius/lg: 12
- sombra: quase nenhuma. shadow/subtle: 0 1px 2px rgba(0,0,0,0.04)
- Cards se definem por borda + fundo, não por sombra pesada.

## Layout base (TODAS as telas seguem)
- Sidebar fixa 240px, bg/subtle, borda direita.
- Header sticky: título 20 semibold à esquerda, ações à direita.
- Conteúdo: max-width 960px, centralizado, padding 32 topo/lateral.
  (fim das telas com metade da tela vazia — coluna única centrada e consistente)
- Listas densas podem usar até 1120px.

## Componentes canônicos (criar como componentes reutilizáveis)
1. AppShell (sidebar + header + slot de conteúdo)
2. PageHeader (título + subtítulo + ações)
3. Card (borda, radius/md, padding 24)
4. ListRow (linha de lista com hover bg/muted, sem card por item)
5. ScoreBadge (número + faixa de cor por escala acima)
6. StatusBadge (bloqueante / decisão / rotina)
7. Button (variantes: primary accent, secondary outline, ghost, danger)
8. StatCard (número grande + label — pros contadores do Dashboard)
9. EmptyState / LoadingState padronizados

## Regras de aplicação por tela
- Dashboard: StatCards no topo; lista de vagas ordenada por score desc;
  ScoreBadge com faixa; empresa como metadado secundário.
- Vagas: corrigir score bugado (1%/0%); usar mesmo ScoreBadge do Dashboard;
  tabs Disponíveis/Aplicadas viram segmented control; agrupar por faixa de fit.
- Pendências: separar visualmente Bloqueante (topo, destaque vermelho) de
  Rotina (lista compacta). Ação irreversível "Aprovar e submeter" = danger/
  primary forte; "Resolver" = ghost. Nada de listar tudo com peso igual.
- ATS: score 94/100 como anel/medidor; layout 2 colunas (relatório + painel
  de ações) num grid consistente, não flutuando.
- Sessão / Feedback: parar de centralizar em coluna estreita com vazio à
  esquerda; usar o mesmo container 960px. Logs em bloco mono com bg/muted.
- Perfil: variantes de busca como lista com toggle; agrupar Salvar perfil
  perto dos campos.