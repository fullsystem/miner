# miner

Minerador solo de Bitcoin (modo "loteria") **docker-first**: um container, configuração
100% por variáveis de ambiente, dashboard com dados 100% reais. Feito para rodar em
qualquer VPS/Dokploy/homelab ao lado dos seus outros projetos.

> Inspirado no [btc-lottery-miner](https://github.com/Educabral/btc-lottery-miner),
> repensado para servidores: sem instalador, sem estado local, sem números simulados.

## Uso

```bash
docker run -d \
  -e WALLET=bc1qsuacarteira... \
  -e POWER=50 \
  -p 3500:3500 \
  ghcr.io/fullsystem/miner:latest
```

Ou com compose: copie `.env.example` para `.env`, preencha `WALLET` e rode
`docker compose up -d`.

Verifique seu worker em `https://web.public-pool.io/#/app/SUA_WALLET`.

## Configuração (env)

| Variável | Padrão | Descrição |
|---|---|---|
| `WALLET` | — | **Obrigatória.** Sua carteira BTC (recebe o prêmio se achar um bloco) |
| `POWER` | `50` | % dos núcleos da CPU usados pelo minerador (1-100) |
| `WORKER_NAME` | `docker` | Nome do worker na pool (útil com várias instâncias) |
| `POOL_URL` | `stratum+tcp://public-pool.io:21496` | Pool solo (stratum) |
| `PORT` | `3500` | Porta do dashboard |
| `DASHBOARD_PASSWORD` | — | Senha do painel; sem ela, painel público somente-leitura |
| `MINER_BIN` | `/usr/local/bin/minerd` | Binário do minerador |
| `MINER_ARGS` | — | Argumentos customizados do minerador (veja abaixo) |

Para limitar o consumo além do `POWER`, use o limite de CPU do próprio Docker
(`cpus: "2.0"` no compose ou o limite de recursos do Dokploy).

## Engine plugável (GPU, outros miners)

O motor de mineração é trocável via env: monte seu binário no container e
aponte `MINER_BIN` + `MINER_ARGS`. Nos argumentos, `{POOL}`, `{USER}` e
`{THREADS}` são substituídos pela configuração:

```yaml
services:
  miner:
    image: ghcr.io/fullsystem/miner:latest
    volumes:
      - ./meu-gpu-miner:/opt/gpu-miner:ro
    environment:
      WALLET: bc1q...
      MINER_BIN: /opt/gpu-miner
      MINER_ARGS: "--url {POOL} --user {USER} --pass x --gpu 0"
    # GPU NVIDIA: requer nvidia-container-toolkit no host
    # deploy:
    #   resources:
    #     reservations:
    #       devices:
    #         - driver: nvidia
    #           count: all
    #           capabilities: [gpu]
```

O supervisor (restart com backoff, shutdown limpo, `/health`) funciona igual
para qualquer engine.

> **Honestidade sobre GPU + Bitcoin**: SHA-256d em GPU perdeu a corrida para os
> ASICs em ~2013. Uma GPU melhora suas chances em ~1000x sobre a CPU, mas a
> loteria continua sendo loteria. O suporte existe pela flexibilidade (outros
> algoritmos/pools/miners), não por viabilidade econômica em BTC.

## Arquitetura

- **Motor de hash**: [pooler/cpuminer](https://github.com/pooler/cpuminer) (`minerd`),
  compilado do fonte no build da imagem — nenhum binário pré-compilado no repo,
  suporte nativo a amd64 e arm64.
- **Supervisor/dashboard**: binário Rust (axum + tokio) que gerencia o minerador
  (restart com backoff exponencial), expõe `/health` e serve o painel.
- **Shutdown limpo**: `SIGTERM` encerra minerador e servidor graciosamente.

## Aviso honesto

Minerar Bitcoin solo com CPU é uma loteria de verdade: as chances de encontrar um
bloco são efetivamente zero (a rede opera em EH/s; uma CPU, em MH/s). Rode por
diversão, aprendizado e para contribuir com a descentralização — não por renda.

## Desenvolvimento

```bash
cargo test
WALLET=bc1q... MINER_BIN=/caminho/minerd cargo run
```

## Licença

MIT
