# DNS Manager

Um gerenciador de servidores DNS para Linux com monitoramento automático e benchmarking, escrito em Rust.

## Problema

Você usa provedores de DNS que podem parar de responder sem aviso (como aconteceu com os servidores IPv6 da Hetzner). Este programa resolve isso:

- Monitora constantemente a saúde dos servidores DNS
- Faz benchmark automático e seleciona os mais rápidos
- Atualiza automaticamente `/etc/resolv.conf` com os melhores servidores
- Funciona perfeitamente no Rocky Linux 9/10

## Funcionalidades

1. **Configuração via YAML**: Lista de servidores DNS configurável
2. **Modo Check**: Apenas verifica se os DNS estão online/offline
3. **Modo Benchmark**: Mede latência e seleciona os DNS mais rápidos
4. **Timer Check**: Verifica saúde dos DNS em intervalos configuráveis
5. **Timer Benchmark**: Executa benchmark em intervalos configuráveis
6. **Atualização Automática**: Atualiza `/etc/resolv.conf` com pelo menos 2 servidores DNS

## Instalação

### Requisitos

- Rust 1.70 ou superior
- Rocky Linux 9/10 (ou qualquer distribuição Linux moderna)
- Permissões de root/sudo para editar `/etc/resolv.conf`

### Compilar

```bash
cargo build --release
```

O binário estará em `target/release/dns-manager`

### Instalar

```bash
sudo cp target/release/dns-manager /usr/local/bin/
sudo chmod +x /usr/local/bin/dns-manager
```

## Configuração

Crie um arquivo `config.yaml`:

```yaml
# Lista de servidores DNS para monitorar
dns_servers:
  - name: "Cloudflare-1"
    address: "1.1.1.1"
  - name: "Cloudflare-2"
    address: "1.0.0.1"
  - name: "Google-1"
    address: "8.8.8.8"
  - name: "Google-2"
    address: "8.8.4.4"
  - name: "Quad9"
    address: "9.9.9.9"
  - name: "Hetzner-IPv6-1"
    address: "2a01:4ff:ff00::add:1"
  - name: "Hetzner-IPv6-2"
    address: "2a01:4ff:ff00::add:2"

# Modo de operação: "check" (apenas verifica) ou "benchmark" (testa e seleciona)
mode: benchmark

# Intervalo para verificar se DNS estão online (em segundos)
# Recomendado: 120 (2 minutos)
check_interval_seconds: 120

# Intervalo para fazer benchmark (em segundos)
# Recomendado: 1800 (30 minutos)
benchmark_interval_seconds: 1800

# Caminho do resolv.conf (opcional, padrão: /etc/resolv.conf)
resolv_conf_path: "/etc/resolv.conf"
```

## Uso

### Verificar DNS uma vez

```bash
sudo dns-manager --config config.yaml check
```

### Fazer benchmark uma vez

```bash
sudo dns-manager --config config.yaml benchmark
```

### Executar como daemon

```bash
sudo dns-manager --config config.yaml run
# ou simplesmente
sudo dns-manager --config config.yaml
```

## Instalação como Serviço Systemd

1. Copie o arquivo de serviço:

```bash
sudo cp dns-manager.service /etc/systemd/system/
```

2. Edite o arquivo de serviço se necessário:

```bash
sudo nano /etc/systemd/system/dns-manager.service
```

3. Crie o diretório de configuração:

```bash
sudo mkdir -p /etc/dns-manager
sudo cp config.yaml /etc/dns-manager/
```

4. Habilite e inicie o serviço:

```bash
sudo systemctl daemon-reload
sudo systemctl enable dns-manager
sudo systemctl start dns-manager
```

5. Verifique o status:

```bash
sudo systemctl status dns-manager
```

6. Ver logs:

```bash
sudo journalctl -u dns-manager -f
```

## Modos de Operação

### Modo Check

- Verifica periodicamente se os DNS estão respondendo
- Intervalo configurável via `check_interval_seconds`
- Apenas monitora, não altera `/etc/resolv.conf`
- Útil para monitoramento e alertas

### Modo Benchmark

- Mede a latência de cada servidor DNS
- Seleciona os 2 servidores mais rápidos
- Atualiza automaticamente `/etc/resolv.conf`
- Intervalo configurável via `benchmark_interval_seconds`
- Também faz verificação de saúde no intervalo `check_interval_seconds`

## Funcionamento

### Health Check (a cada 2 minutos)

1. Testa cada servidor DNS fazendo uma consulta para `google.com`
2. Marca como ONLINE ou OFFLINE
3. Exibe resultados no log

### Benchmark (a cada 30 minutos)

1. Testa cada servidor DNS medindo tempo de resposta
2. Ordena por latência (mais rápido primeiro)
3. Seleciona os 2 melhores servidores
4. Cria backup de `/etc/resolv.conf`
5. Atualiza `/etc/resolv.conf` com os melhores servidores

## Segurança

- Cria backup automático de `/etc/resolv.conf` antes de modificar
- Verifica permissões antes de iniciar
- Usa arquivo temporário para gravação atômica
- Registra todas as operações em log

## Estrutura do Projeto

```
dns-manager/
├── src/
│   ├── main.rs           # Aplicação principal e CLI
│   ├── config.rs         # Parser de configuração YAML
│   ├── dns_checker.rs    # Lógica de verificação e benchmark
│   └── resolv_conf.rs    # Gerenciador de /etc/resolv.conf
├── Cargo.toml            # Dependências Rust
├── config.yaml           # Configuração exemplo
├── dns-manager.service   # Serviço systemd
└── README.md             # Este arquivo
```

## Dependências

- `tokio` - Runtime assíncrono
- `trust-dns-resolver` - Cliente DNS
- `serde` / `serde_yaml` - Parser YAML
- `clap` - Parser de argumentos CLI
- `anyhow` - Tratamento de erros
- `log` / `env_logger` - Sistema de logs
- `chrono` - Timestamps

## Troubleshooting

### Erro de permissão

```
Error: No write permission for /etc/resolv.conf
```

**Solução**: Execute com `sudo`

### DNS não estão sendo atualizados

1. Verifique se o serviço está rodando:
   ```bash
   sudo systemctl status dns-manager
   ```

2. Verifique o modo de operação no `config.yaml`:
   ```yaml
   mode: benchmark  # Deve ser "benchmark" para atualizar
   ```

3. Verifique logs:
   ```bash
   sudo journalctl -u dns-manager -n 100
   ```

### Todos os DNS estão offline

- Verifique conectividade de rede
- Verifique firewall (porta 53 UDP)
- Teste manualmente: `dig @1.1.1.1 google.com`

## Exemplos de Uso

### Testar configuração

```bash
# Verificar sintaxe do YAML
sudo dns-manager --config config.yaml check

# Fazer benchmark e ver resultados
sudo dns-manager --config config.yaml benchmark
```

### Monitoramento contínuo

```bash
# Modo check apenas (não altera resolv.conf)
mode: check
check_interval_seconds: 120  # A cada 2 minutos

# Modo benchmark (atualiza resolv.conf)
mode: benchmark
check_interval_seconds: 120      # Verifica saúde a cada 2 min
benchmark_interval_seconds: 1800  # Benchmark a cada 30 min
```

## Licença

MIT

## Autor

Criado para resolver problemas reais de DNS no Rocky Linux com servidores Hetzner.
