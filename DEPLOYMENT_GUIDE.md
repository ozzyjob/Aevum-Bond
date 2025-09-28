# 🚀 Deployment Guide - Aevum & Bond

## 🎯 Overview
Comprehensive deployment guide for the Aevum & Bond dual-ledger blockchain ecosystem.

## 📋 Prerequisites

### System Requirements
- **OS**: Linux (Ubuntu 20.04+ recommended)
- **CPU**: 4+ cores, 64-bit architecture
- **RAM**: 8GB minimum, 16GB recommended
- **Storage**: 100GB+ SSD space
- **Network**: Stable internet connection, open ports 8080, 8081, 30303

### Software Dependencies
```bash
# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default stable
rustup component add clippy rustfmt

# System packages
sudo apt update
sudo apt install -y build-essential pkg-config libssl-dev git curl

# Docker (optional, for containerized deployment)
curl -fsSL https://get.docker.com | sh
sudo usermod -aG docker $USER
```

## 🔧 Environment Setup

### 1. Clone Repository
```bash
git clone https://github.com/ozzyjob/Aevum-Bond.git
cd Aevum-Bond
```

### 2. Environment Configuration
Create `.env` file:
```bash
# Network Configuration
BOND_PORT=8080
AEVUM_PORT=8081
P2P_PORT=30303
RPC_HOST=127.0.0.1

# Security Configuration  
ENABLE_TLS=true
KEY_STORE_PATH=./keystore
LOG_LEVEL=info

# Performance Configuration
MINING_THREADS=4
VALIDATOR_STAKE=1000
CACHE_SIZE=1024

# Database Configuration
BOND_DB_PATH=./data/bond
AEVUM_DB_PATH=./data/aevum
BACKUP_INTERVAL=3600
```

### 3. Build Project
```bash
# Full build with optimizations
cargo build --release --all

# Verify build
cargo test --all
```

## 🏗️ Deployment Options

### Option 1: Native Binary Deployment

#### Production Build
```bash
# Build optimized binaries
cargo build --release

# Install system-wide (optional)
sudo cp target/release/aevum-bond /usr/local/bin/
sudo cp target/release/bond-miner /usr/local/bin/
sudo cp target/release/aevum-validator /usr/local/bin/
```

#### System Service Setup
Create systemd service file `/etc/systemd/system/aevum-bond.service`:
```ini
[Unit]
Description=Aevum & Bond Blockchain Node
After=network.target
Wants=network.target

[Service]
Type=simple
User=aevum
Group=aevum
WorkingDirectory=/opt/aevum-bond
ExecStart=/usr/local/bin/aevum-bond node start --config /etc/aevum-bond/config.toml
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal
SyslogIdentifier=aevum-bond

# Security hardening
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/aevum-bond/data
PrivateTmp=true

[Install]
WantedBy=multi-user.target
```

Enable and start service:
```bash
sudo systemctl daemon-reload
sudo systemctl enable aevum-bond
sudo systemctl start aevum-bond
sudo systemctl status aevum-bond
```

### Option 2: Docker Deployment

#### Dockerfile
```dockerfile
FROM rust:1.70 as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/aevum-bond /usr/local/bin/
COPY --from=builder /app/config/ /app/config/

EXPOSE 8080 8081 30303
VOLUME ["/app/data"]

CMD ["aevum-bond", "node", "start"]
```

#### Docker Compose
```yaml
version: '3.8'

services:
  aevum-bond-node:
    build: .
    container_name: aevum-bond-node
    restart: unless-stopped
    ports:
      - "8080:8080"  # Bond Chain RPC
      - "8081:8081"  # Aevum Chain RPC  
      - "30303:30303" # P2P Network
    volumes:
      - ./data:/app/data
      - ./config:/app/config
      - ./logs:/app/logs
    environment:
      - RUST_LOG=info
      - BOND_PORT=8080
      - AEVUM_PORT=8081
      - P2P_PORT=30303
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  bond-miner:
    build: .
    container_name: bond-miner
    restart: unless-stopped
    command: ["aevum-bond", "mining", "start"]
    depends_on:
      - aevum-bond-node
    environment:
      - MINING_THREADS=4
      - NODE_URL=http://aevum-bond-node:8080

  aevum-validator:
    build: .
    container_name: aevum-validator
    restart: unless-stopped
    command: ["aevum-bond", "validator", "start"]
    depends_on:
      - aevum-bond-node
    environment:
      - VALIDATOR_STAKE=1000
      - NODE_URL=http://aevum-bond-node:8081

  monitoring:
    image: prom/prometheus
    container_name: monitoring
    ports:
      - "9090:9090"
    volumes:
      - ./monitoring/prometheus.yml:/etc/prometheus/prometheus.yml
```

#### Deploy with Docker
```bash
# Build and start services
docker-compose up -d

# Check service status
docker-compose ps

# View logs
docker-compose logs -f aevum-bond-node
```

### Option 3: Kubernetes Deployment

#### Namespace and ConfigMap
```yaml
# namespace.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: aevum-bond

---
# configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: aevum-bond-config
  namespace: aevum-bond
data:
  config.toml: |
    [network]
    bond_port = 8080
    aevum_port = 8081
    p2p_port = 30303
    
    [security]
    enable_tls = true
    key_store_path = "/app/keystore"
    
    [performance]
    mining_threads = 4
    validator_stake = 1000
```

#### Deployment Configuration
```yaml
# deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: aevum-bond-node
  namespace: aevum-bond
spec:
  replicas: 3
  selector:
    matchLabels:
      app: aevum-bond-node
  template:
    metadata:
      labels:
        app: aevum-bond-node
    spec:
      containers:
      - name: aevum-bond
        image: aevum-bond:latest
        ports:
        - containerPort: 8080
        - containerPort: 8081
        - containerPort: 30303
        volumeMounts:
        - name: data
          mountPath: /app/data
        - name: config
          mountPath: /app/config
        env:
        - name: RUST_LOG
          value: "info"
        resources:
          requests:
            memory: "2Gi"
            cpu: "1000m"
          limits:
            memory: "4Gi"
            cpu: "2000m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 30
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 10
      volumes:
      - name: data
        persistentVolumeClaim:
          claimName: aevum-bond-data
      - name: config
        configMap:
          name: aevum-bond-config

---
# service.yaml
apiVersion: v1
kind: Service
metadata:
  name: aevum-bond-service
  namespace: aevum-bond
spec:
  selector:
    app: aevum-bond-node
  ports:
  - name: bond-rpc
    port: 8080
    targetPort: 8080
  - name: aevum-rpc
    port: 8081
    targetPort: 8081
  - name: p2p
    port: 30303
    targetPort: 30303
  type: LoadBalancer
```

## 🔐 Security Configuration

### TLS Certificate Setup
```bash
# Generate self-signed certificates for development
openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes

# For production, use Let's Encrypt
sudo apt install certbot
sudo certbot certonly --standalone -d your-domain.com
```

### Firewall Configuration
```bash
# UFW firewall setup
sudo ufw allow 22/tcp      # SSH
sudo ufw allow 8080/tcp    # Bond Chain RPC
sudo ufw allow 8081/tcp    # Aevum Chain RPC
sudo ufw allow 30303/tcp   # P2P Network
sudo ufw enable
```

### Key Management
```bash
# Create secure keystore directory
sudo mkdir -p /etc/aevum-bond/keystore
sudo chmod 700 /etc/aevum-bond/keystore
sudo chown aevum:aevum /etc/aevum-bond/keystore

# Generate node keys
aevum-bond keys generate --output /etc/aevum-bond/keystore/node.key
```

## 📊 Monitoring Setup

### Prometheus Configuration
```yaml
# prometheus.yml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'aevum-bond'
    static_configs:
      - targets: ['localhost:8080', 'localhost:8081']
    scrape_interval: 10s
    metrics_path: /metrics
```

### Grafana Dashboard
```json
{
  "dashboard": {
    "title": "Aevum & Bond Monitoring",
    "panels": [
      {
        "title": "Block Height",
        "targets": [
          {
            "expr": "aevum_bond_block_height"
          }
        ]
      },
      {
        "title": "Transaction Throughput",
        "targets": [
          {
            "expr": "rate(aevum_bond_transactions_total[5m])"
          }
        ]  
      }
    ]
  }
}
```

## 🚀 Production Deployment Checklist

### Pre-Deployment
- [ ] Security audit completed
- [ ] Performance testing passed
- [ ] Backup procedures tested
- [ ] Monitoring configured
- [ ] Documentation updated

### Deployment Steps
1. [ ] Deploy infrastructure
2. [ ] Configure networking
3. [ ] Install certificates
4. [ ] Deploy application
5. [ ] Configure monitoring
6. [ ] Run health checks
7. [ ] Configure backups
8. [ ] Document procedures

### Post-Deployment
- [ ] Monitor system health
- [ ] Verify blockchain sync
- [ ] Test all endpoints
- [ ] Check security logs
- [ ] Validate performance metrics

## 🔄 Maintenance Procedures

### Regular Maintenance
```bash
# Weekly maintenance script
#!/bin/bash

# Update system packages
sudo apt update && sudo apt upgrade -y

# Restart services
sudo systemctl restart aevum-bond

# Clean old logs
find /var/log/aevum-bond -name "*.log" -mtime +7 -delete

# Backup database
tar -czf backup-$(date +%Y%m%d).tar.gz /opt/aevum-bond/data

# Check disk space
df -h /opt/aevum-bond/data
```

### Emergency Procedures
```bash
# Stop all services
sudo systemctl stop aevum-bond

# Restore from backup
tar -xzf backup-latest.tar.gz -C /opt/aevum-bond/

# Restart services
sudo systemctl start aevum-bond

# Verify system health
aevum-bond node status
```

## 📈 Scaling Considerations

### Horizontal Scaling
- Multiple validator nodes for Aevum chain
- Mining pool setup for Bond chain
- Load balancer configuration
- Database sharding strategies

### Vertical Scaling
- CPU optimization for mining
- Memory allocation for caching
- Storage performance tuning
- Network bandwidth optimization

## 🆘 Troubleshooting

### Common Issues
1. **Port binding errors**: Check firewall and port availability
2. **Database corruption**: Restore from backup and resync
3. **Network connectivity**: Verify P2P port accessibility
4. **Memory issues**: Adjust cache settings and limits

### Debug Commands
```bash
# Check service status
sudo systemctl status aevum-bond

# View detailed logs
journalctl -u aevum-bond -f

# Network diagnostics
aevum-bond network peers
aevum-bond network status

# Database integrity check
aevum-bond db verify
```

---

**Deployment Guide Version**: 1.0.0  
**Last Updated**: September 27, 2025  
**Status**: Production Ready ✅
