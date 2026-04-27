# ArkCore 部署指南

本指南涵盖 ArkCore 在各种环境下的部署方案，包括开发环境、测试环境、生产环境以及云平台部署。

## 目录

- [环境要求](#环境要求)
- [构建发布版本](#构建发布版本)
- [本地部署](#本地部署)
- [Docker 部署](#docker-部署)
- [系统服务部署](#系统服务部署)
- [反向代理配置](#反向代理配置)
- [高可用部署](#高可用部署)
- [监控与日志](#监控与日志)
- [备份与恢复](#备份与恢复)
- [升级与回滚](#升级与回滚)

## 环境要求

### 硬件要求

| 环境 | CPU | 内存 | 磁盘 |
|------|-----|------|------|
| 开发/测试 | 2 核 | 4 GB | 10 GB |
| 小规模生产 | 4 核 | 8 GB | 50 GB |
| 中等规模生产 | 8 核 | 16 GB | 100 GB |
| 大规模生产 | 16+ 核 | 32+ GB | 200+ GB |

### 软件要求

- Rust 1.85+
- SQLite 3.30+
- 操作系统: Linux (glibc 2.31+), macOS 11+, Windows 10+

### 平台特定依赖

**Ubuntu/Debian:**
```bash
sudo apt-get update
sudo apt-get install -y build-essential pkg-config libsqlite3-dev
```

**CentOS/RHEL:**
```bash
sudo yum groupinstall -y "Development Tools"
sudo yum install -y sqlite-devel
```

**macOS:**
```bash
brew install sqlite
xcode-select --install
```

**Windows:**
- 安装 Visual Studio Build Tools
- 或使用 rustup 默认安装的 MSVC 工具链

## 构建发布版本

### 优化构建

```bash
# Release 构建（推荐生产使用）
cargo build --release

# 特定平台目标
cargo build --release --target x86_64-unknown-linux-musl  # Alpine Linux

# 链接时优化（LTO）
cargo build --release -C lto=on -C codegen-units=1
```

### 构建产物

```
target/release/
├── arkcore          # Linux/macOS
└── arkcore.exe      # Windows
```

### 验证构建

```bash
# 验证二进制文件
./target/release/arkcore --version

# 验证依赖
ldd target/release/arkcore  # Linux
otool -L target/release/arkcore  # macOS
```

## 本地部署

### 目录结构

```
/opt/arkcore/
├── arkcore                    # 二进制文件
├── config/
│   └── config.toml            # 配置文件
├── data/
│   └── memory.db              # SQLite 数据库
├── logs/
│   └── arkcore.log            # 日志文件
└── run/
    └── arkcore.pid            # PID 文件
```

### 安装步骤

```bash
# 1. 创建目录结构
sudo mkdir -p /opt/arkcore/{config,data,logs,run}

# 2. 复制二进制文件
sudo cp target/release/arkcore /opt/arkcore/

# 3. 创建配置文件
sudo cat > /opt/arkcore/config/config.toml << 'EOF'
[server]
port = 8080
host = "127.0.0.1"

[database]
path = "/opt/arkcore/data/memory.db"

[security]
audit_enabled = true
sandbox_enabled = true
EOF

# 4. 设置权限
sudo chown -R arkcore:arkcore /opt/arkcore
sudo chmod +x /opt/arkcore/arkcore
```

### 启动服务

```bash
# 直接启动（前台）
/opt/arkcore/arkcore daemon --port 8080

# 后台运行
nohup /opt/arkcore/arkcore daemon --port 8080 \
    > /opt/arkcore/logs/arkcore.log 2>&1 &

# 验证运行
curl http://127.0.0.1:8080/health
```

## Docker 部署

### Dockerfile

```dockerfile
FROM rust:1.85-slim as builder

WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN apt-get update && apt-get install -y pkg-config libsqlite3-dev
RUN cargo build --release
RUN strip target/release/arkcore

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    libsqlite3-0 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /build/target/release/arkcore .
COPY config ./config

EXPOSE 8080

VOLUME ["/app/data", "/app/logs"]

ENTRYPOINT ["./arkcore"]
CMD ["daemon", "--port", "8080"]
```

### 构建镜像

```bash
# 构建
docker build -t arkcore:latest .

# 查看镜像大小
docker images arkcore
```

### 运行容器

```bash
# 基本运行
docker run -d \
  --name arkcore \
  -p 8080:8080 \
  -v arkcore-data:/app/data \
  -v arkcore-logs:/app/logs \
  arkcore:latest

# 带配置文件
docker run -d \
  --name arkcore \
  -p 8080:8080 \
  -v /path/to/config.toml:/app/config/config.toml:ro \
  -v arkcore-data:/app/data \
  arkcore:latest

# 查看日志
docker logs -f arkcore

# 进入容器
docker exec -it arkcore sh
```

### Docker Compose

```yaml
version: '3.8'

services:
  arkcore:
    image: arkcore:latest
    container_name: arkcore
    ports:
      - "8080:8080"
    volumes:
      - ./config:/app/config:ro
      - arkcore-data:/app/data
      - arkcore-logs:/app/logs
    environment:
      - RUST_LOG=arkcore=info
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3

volumes:
  arkcore-data:
  arkcore-logs:
```

```bash
# 启动
docker-compose up -d

# 查看状态
docker-compose ps

# 停止
docker-compose down
```

## 系统服务部署

### systemd (Linux)

```ini
[Unit]
Description=ArkCore Local-First OS Agent Engine
After=network.target
Wants=network-online.target

[Service]
Type=simple
User=arkcore
Group=arkcore
WorkingDirectory=/opt/arkcore
ExecStart=/opt/arkcore/arkcore daemon --port 8080
ExecReload=/bin/kill -HUP $MAINPID
Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal
SyslogIdentifier=arkcore

# 安全配置
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/arkcore/data /opt/arkcore/logs

# 资源限制
LimitNOFILE=65536
MemoryMax=1G

[Install]
WantedBy=multi-user.target
```

```bash
# 复制服务文件
sudo cp arkcore.service /etc/systemd/system/

# 重新加载 systemd
sudo systemctl daemon-reload

# 启用服务
sudo systemctl enable arkcore

# 启动服务
sudo systemctl start arkcore

# 查看状态
sudo systemctl status arkcore

# 查看日志
sudo journalctl -u arkcore -f
```

### launchd (macOS)

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "...">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.arkcore.daemon</string>
    <key>ProgramArguments</key>
    <array>
        <string>/opt/arkcore/arkcore</string>
        <string>daemon</string>
        <string>--port</string>
        <string>8080</string>
    </array>
    <key>WorkingDirectory</key>
    <string>/opt/arkcore</string>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>/opt/arkcore/logs/arkcore.log</string>
    <key>StandardErrorPath</key>
    <string>/opt/arkcore/logs/arkcore.log</string>
</dict>
</plist>
```

```bash
# 安装服务
sudo cp com.arkcore.daemon.plist /Library/LaunchDaemons/

# 启动服务
sudo launchctl load /Library/LaunchDaemons/com.arkcore.daemon.plist

# 停止服务
sudo launchctl unload /Library/LaunchDaemons/com.arkcore.daemon.plist
```

### Windows 服务

使用 `nssm` (Non-Sucking Service Manager):

```bash
# 下载 nssm
choco install nssm

# 创建服务
nssm install arkcore "C:\Program Files\arkcore\arkcore.exe" "daemon --port 8080"
nssm set arkcore AppDirectory "C:\Program Files\arkcore"
nssm set arkcore DisplayName "ArkCore Service"
nssm set arkcore Description "Local-First OS Agent Engine"

# 启动服务
nssm start arkcore

# 查看状态
nssm status arkcore
```

## 反向代理配置

### Nginx

```nginx
upstream arkcore_backend {
    server 127.0.0.1:8080;
    keepalive 32;
}

server {
    listen 443 ssl http2;
    server_name arkcore.example.com;

    ssl_certificate /etc/ssl/certs/arkcore.crt;
    ssl_certificate_key /etc/ssl/private/arkcore.key;

    # WebSocket 支持
    location /ws {
        proxy_pass http://arkcore_backend;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_read_timeout 86400;
    }

    # SSE 支持
    location /events {
        proxy_pass http://arkcore_backend;
        proxy_http_version 1.1;
        proxy_set_header Accept "text/event-stream";
        proxy_set_header Connection "keep-alive";
        proxy_cache off;
        proxy_buffering off;
    }

    # 其他请求
    location / {
        proxy_pass http://arkcore_backend;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # 超时设置
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
    }

    # 健康检查（无需代理）
    location /health {
        proxy_pass http://arkcore_backend;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
    }
}
```

### Caddy

```caddy
arkcore.example.com {
    reverse_proxy /ws/* 127.0.0.1:8080 {
        header_up Upgrade {header.Connection}
        header_up Connection "upgrade"
    }

    reverse_proxy /events/* 127.0.0.1:8080 {
        header_up Accept "text/event-stream"
    }

    reverse_proxy /* 127.0.0.1:8080
}
```

## 高可用部署

### 多实例部署

```
                    ┌─────────────┐
                    │   Load      │
                    │   Balancer  │
                    └──────┬──────┘
                           │
        ┌──────────────────┼──────────────────┐
        │                  │                  │
   ┌────▼────┐       ┌────▼────┐       ┌────▼────┐
   │ ArkCore │       │ ArkCore │       │ ArkCore │
   │   8080  │       │   8081  │       │   8082  │
   └────┬────┘       └────┬────┘       └────┬────┘
        │                  │                  │
        └──────────────────┼──────────────────┘
                           │
                    ┌──────▼──────┐
                    │  SQLite/S3  │
                    │  共享存储   │
                    └─────────────┘
```

### Docker Swarm 部署

```yaml
version: '3.8'

services:
  arkcore:
    image: arkcore:latest
    deploy:
      replicas: 3
      update_config:
        parallelism: 1
        delay: 10s
      restart_policy:
        condition: on-failure
    ports:
      - "8080:8080"
    volumes:
      - arkcore-data:/app/data
    environment:
      - RUST_LOG=info
    configs:
      - source: arkcore_config
        target: /app/config/config.toml

configs:
  arkcore_config:
    file: ./config/config.toml

volumes:
  arkcore-data:
    driver: rexray/gcepd  # 云存储驱动
```

### Kubernetes 部署

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: arkcore
  labels:
    app: arkcore
spec:
  replicas: 3
  selector:
    matchLabels:
      app: arkcore
  template:
    metadata:
      labels:
        app: arkcore
    spec:
      containers:
      - name: arkcore
        image: arkcore:latest
        ports:
        - containerPort: 8080
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "1Gi"
            cpu: "1000m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
        volumeMounts:
        - name: config
          mountPath: /app/config
          readOnly: true
      volumes:
      - name: config
        configMap:
          name: arkcore-config
---
apiVersion: v1
kind: Service
metadata:
  name: arkcore
spec:
  type: LoadBalancer
  ports:
  - port: 80
    targetPort: 8080
  selector:
    app: arkcore
```

## 监控与日志

### 日志配置

```toml
[logging]
level = "info"  # trace, debug, info, warn, error
format = "json"  # text, json

[logging.file]
enabled = true
path = "/var/log/arkcore"
filename = "arkcore.log"
max_size_mb = 100
max_files = 10
```

### 结构化日志字段

```json
{
  "timestamp": "2024-01-01T12:00:00Z",
  "level": "info",
  "message": "Command executed",
  "service": "arkcore",
  "trace_id": "abc123",
  "user_id": "user-123",
  "command": "ls -la",
  "duration_ms": 45,
  "exit_code": 0
}
```

### Prometheus 指标

```bash
# 拉取指标
curl http://localhost:8080/metrics

# 格式
# arkcore_commands_total{status="success"} 1234
# arkcore_command_duration_seconds_bucket{le="0.1"} 1000
# arkcore_active_agents 5
```

### Grafana 仪表板

推荐面板:
- Commands/sec
- Command Success Rate
- Average Latency
- Active Agents
- Memory Usage
- Cache Hit Rate

## 备份与恢复

### 数据库备份

```bash
# 关闭服务（备份时）
sudo systemctl stop arkcore

# 备份数据库
sudo cp /opt/arkcore/data/memory.db /backup/memory-$(date +%Y%m%d).db

# 备份配置
sudo tar -czf /backup/arkcore-config-$(date +%Y%m%d).tar.gz /opt/arkcore/config

# 重启服务
sudo systemctl start arkcore
```

### 自动备份脚本

```bash
#!/bin/bash
# backup.sh

BACKUP_DIR="/backup/arkcore"
DATE=$(date +%Y%m%d)

mkdir -p $BACKUP_DIR

# 停止服务
systemctl stop arkcore

# 备份
cp /opt/arkcore/data/memory.db $BACKUP_DIR/memory-$DATE.db
tar -czf $BACKUP_DIR/config-$DATE.tar.gz /opt/arkcore/config

# 启动服务
systemctl start arkcore

# 清理 7 天前的备份
find $BACKUP_DIR -name "*.db" -mtime +7 -delete
find $BACKUP_DIR -name "*.tar.gz" -mtime +7 -delete
```

### 恢复数据

```bash
# 停止服务
sudo systemctl stop arkcore

# 恢复数据库
sudo cp /backup/memory-20240101.db /opt/arkcore/data/memory.db

# 恢复配置
sudo tar -xzf /backup/config-20240101.tar.gz -C /

# 启动服务
sudo systemctl start arkcore
```

## 升级与回滚

### 升级步骤

```bash
# 1. 备份当前版本
sudo cp /opt/arkcore/arkcore /opt/arkcore/arkcore.bak

# 2. 下载新版本
wget https://example.com/arkcore-v2.0.0
chmod +x arkcore-v2.0.0

# 3. 停止服务
sudo systemctl stop arkcore

# 4. 替换二进制
sudo mv arkcore-v2.0.0 /opt/arkcore/arkcore

# 5. 启动服务
sudo systemctl start arkcore

# 6. 验证
curl http://localhost:8080/health
```

### 回滚步骤

```bash
# 1. 停止服务
sudo systemctl stop arkcore

# 2. 恢复备份版本
sudo mv /opt/arkcore/arkcore.bak /opt/arkcore/arkcore

# 3. 启动服务
sudo systemctl start arkcore

# 4. 验证
curl http://localhost:8080/health
```

### Docker 升级

```bash
# 拉取新镜像
docker pull arkcore:latest

# 重启容器（使用 docker-compose）
docker-compose pull
docker-compose up -d

# 或手动重建
docker stop arkcore
docker rm arkcore
docker run -d \
  --name arkcore \
  -p 8080:8080 \
  -v arkcore-data:/app/data \
  arkcore:latest
```
