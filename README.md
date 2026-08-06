<div align="center">

# 💰 Invest Wallet

**Plataforma web fullstack para gestão de investimentos**

![Rust](https://img.shields.io/badge/Rust-2024-orange?logo=rust&logoColor=white)
![Axum](https://img.shields.io/badge/Axum-Web%20Framework-black)
![PostgreSQL](https://img.shields.io/badge/PostgreSQL-Database-4169E1?logo=postgresql&logoColor=white)
![Docker](https://img.shields.io/badge/Docker-Containerized-2496ED?logo=docker&logoColor=white)

</div>

---

##  Índice

- [O que o projeto faz](#-o-que-o-projeto-faz)
- [Tecnologias utilizadas](#-tecnologias-utilizadas)
- [Melhorias implementadas](#-melhorias-implementadas)
- [Como executar a aplicação](#-como-executar-a-aplicação)
- [Como testar sua versão](#-como-testar-sua-versão)
- [O que você aprendeu](#-o-que-você-aprendeu-durante-o-desafio)

---

##  O que o projeto faz

O **Invest Wallet** é uma plataforma web fullstack focada na gestão de investimentos. Ele permite que os usuários registrem suas compras de diversos ativos (como criptomoedas, ações e moedas fiduciárias) e acompanhem o desempenho financeiro em tempo real.

A aplicação calcula automaticamente:

-  O valor total da carteira
-  O rendimento histórico (lucro/prejuízo) desde a primeira compra
-  A distribuição do portfólio através de gráficos dinâmicos

> **Diferencial:** sincronização de preços ao vivo — o sistema atualiza o valor dos ativos na tela do usuário automaticamente, sem necessidade de recarregar a página.

---

##  Tecnologias utilizadas

O projeto foi construído com foco em **performance**, **concorrência** e **segurança de memória**, utilizando uma stack moderna e robusta.

### Backend

| Tecnologia | Função |
|---|---|
| **Rust** (Edição 2024) | Linguagem principal do servidor |
| **Axum** | Framework web de alta performance para roteamento e APIs |
| **Tokio** | Runtime assíncrono para múltiplas requisições e workers em segundo plano |
| **SQLx** | Interação assíncrona com o banco, com validação de queries em tempo de compilação |
| **Askama** | Motor de templates (SSR) seguro e tipado para renderizar HTML a partir do Rust |
| **Reqwest** | Cliente HTTP para consumir APIs externas (CoinGecko) |

### Frontend

| Tecnologia | Função |
|---|---|
| **HTML5 & CSS3** | Design customizado com tema escuro (*Dark Mode*) e tipografia fluida (`clamp()`) |
| **Vanilla JS** | Reatividade do lado do cliente, minimizando frameworks pesados |
| **Chart.js** | Renderização do gráfico de rosca (*Doughnut chart*) da distribuição do portfólio |
| **Server-Sent Events (SSE)** | Comunicação em tempo real unidirecional do servidor para o cliente |

### Infraestrutura e Banco de Dados

| Tecnologia | Função |
|---|---|
| **PostgreSQL** | Banco de dados relacional |
| **Docker & Docker Compose** | Containerização multi-stage para isolar o ambiente e facilitar o deploy |
| **Bash Scripting** | Automação do povoamento inicial (*seed*) do banco de dados |

---

##  Melhorias implementadas

Ao longo do desenvolvimento, várias melhorias arquiteturais e de interface foram implementadas em relação a um CRUD tradicional:

- ** Worker assíncrono em segundo plano**
  Em vez de travar a thread principal ou fazer o cliente esperar, foi implementado um worker em Rust que roda isolado a cada 5 minutos consultando os preços na CoinGecko, economizando banda e requisições HTTP.

- ** Streaming de preços ao vivo (SSE)**
  Substituição do modelo tradicional de *polling* por uma conexão SSE persistente, permitindo que a interface pisque em verde ou vermelho instantaneamente quando o preço de um ativo muda no banco de dados.

- ** Injeção segura de dados (Rust → JS)**
  Serialização matemática do portfólio em Rust (`serde_json`) diretamente para o motor do Chart.js no carregamento da página, eliminando a necessidade de chamadas de API adicionais no frontend.

- ** Layout assimétrico e responsivo**
  Adoção de um design de painel financeiro profissional, posicionando KPIs 2x2 ao lado do gráfico de forma a otimizar o espaço em telas de desktop sem quebrar em dispositivos móveis.

---

##  Como executar a aplicação

### Pré-requisitos
*   **Docker Compose** instalado.
*   **Docker Desktop** instalado e rodando.
*   **Rust e Cargo** instalados (via [rustup](https://rustup.rs/)).
*   **sqlx-cli** instalado localmente. Se não tiver, instale com: `cargo install sqlx-cli --no-default-features --features native-tls,postgres`

### 1. Clone o repositório e configure as variáveis

Crie um arquivo `.env` na raiz do projeto com o seguinte conteúdo:

```env
DATABASE_URL=postgres://admin:pass1234@127.0.0.1:5432/invest_wallet_db
ADMIN_SECRET_KEY="supersecretkey_to_use_for_a_supersecret"
```

### 2. Construa e inicie os containers

Isso vai baixar o PostgreSQL e compilar a aplicação Rust em um ambiente isolado.

```bash
docker compose up --build -d
```

### 3. Execute as migrações do banco de dados

```bash
DATABASE_URL=postgres://admin:pass1234@127.0.0.1:5432/invest_wallet_db sqlx migrate run
```

### 4. Execute a aplicação Rust localmente

```bash
cargo run
```

---

##  Como testar

### 1. Povoe o banco de dados (*Seed*)

Com os containers rodando, execute o script de bash para popular o sistema com os IDs corretos das APIs:

```bash
chmod +x seed_assets.sh
./seed_assets.sh
```
#### 1.1. Se você já tiver executado `cargo run` antes, pare a aplicação e execute novamente para que o worker seja iniciado com os dados corretos do seed:

ctrl + C para parar a aplicação e depois execute:
```bash
cargo run
```

### 2. Acesse a aplicação

Abra seu navegador em [http://localhost:3000](http://localhost:3000).

- Crie uma conta e faça login.
- **Registre uma compra:** use o modal "Registrar Compra" para adicionar Bitcoin ou Ethereum ao seu portfólio (com a quantidade e o preço que você pagou na época).

### 3. Verifique a reatividade

- Observe o gráfico de distribuição se preencher automaticamente com os valores reais da sua carteira.
- Observe a faixa de "Preços ao Vivo" e aguarde um momento; os valores vão atualizar automaticamente e piscar na tela graças ao worker e ao SSE, sem que você precise recarregar a página.

---

##  O que aprendi durante o desafio

Durante a construção deste projeto, foram enfrentados e superados diversos desafios de engenharia de software:

- **Gerenciamento do ciclo de vida em Rust**
  Entendimento prático de por que o compilador do Rust é tão rigoroso com referências cruzadas, e como lidar com estruturas como `Option<String>` dentro do ecossistema do Askama (desempacotando valores corretamente para evitar erros de tipagem no HTML).

- **Comunicação Frontend/Backend**
  Diferença entre renderização no servidor e no cliente, e como injetar estruturas de dados complexas (vetores em Rust) como strings JSON seguras para serem consumidas pelo JavaScript do navegador (Chart.js).

- **Comportamento de rede HTTP/2**
  Desmistificação de logs verbosos de bibliotecas como o `hyper`, entendendo que fechamentos de conexão (`GoAway`) são rotinas normais de *connection pooling* para poupar memória, e não erros na aplicação.
