# クイックスタートガイド: AuthPulse MVP

**Date**: 2026-03-08
**対象**: 開発者・デモ実施者
**前提**: Docker / Docker Compose V2 インストール済み, grpcurl または grpc-client

---

## 1. 環境起動

```bash
# リポジトリルートで実行
git clone https://github.com/kazuma0606/rust-microservice-with-envoy.git
cd rust-microservice-with-envoy

# 環境変数を設定（シークレット管理）
cp docker/.env.example docker/.env
# docker/.env を編集して DATABASE_URL 等を設定

# 全サービスをビルド & 起動
docker compose -f docker/docker-compose.yml up --build
```

起動後に以下のポートが利用可能:
| サービス | ポート | 用途 |
|---------|--------|------|
| Envoy | 8080 | gRPC-Web / クライアント向け入口 |
| collector | 50051 | gRPC（Envoy 経由で接続） |
| aggregator | 50052 | gRPC（Envoy 経由で接続） |
| PostgreSQL | 5432 | DB（内部のみ） |
| Prometheus | 9090 | メトリクス確認 |

---

## 2. DB マイグレーション確認

```bash
# コンテナ起動時に自動実行されるが、手動確認する場合:
docker compose exec collector sqlx migrate info --database-url $DATABASE_URL
```

---

## 3. 認証認可イベントの投入（IngestEvent）

### grpcurl を使用する場合

```bash
# Envoy 経由で送信（ポート 8080）
grpcurl -plaintext \
  -d '{
    "tenant_id": "tenant-a",
    "user_id": "user-001",
    "service": "payment-service",
    "resource": "/api/payments/transfer",
    "action": "write",
    "decision": "DECISION_ALLOW",
    "latency_ms": 12,
    "trace_id": "abc123",
    "timestamp": "2026-03-08T09:00:00Z"
  }' \
  localhost:8080 \
  authpulse.v1.collector.CollectorService/IngestEvent
```

**期待するレスポンス**:
```json
{
  "event_id": "550e8400-e29b-41d4-a716-446655440000",
  "recorded_at": "2026-03-08T09:00:00.123Z"
}
```

### DENY イベントを大量投入してアラートをトリガーする

```bash
# 5分以内に 10件以上の DENY を同一テナントに投入
for i in $(seq 1 12); do
  grpcurl -plaintext \
    -d "{
      \"tenant_id\": \"tenant-a\",
      \"user_id\": \"user-attacker\",
      \"service\": \"auth-service\",
      \"resource\": \"/admin\",
      \"action\": \"read\",
      \"decision\": \"DECISION_DENY\",
      \"reason_code\": \"PERMISSION_DENIED\",
      \"timestamp\": \"2026-03-08T09:01:0${i}Z\"
    }" \
    localhost:8080 \
    authpulse.v1.collector.CollectorService/IngestEvent
done
```

---

## 4. メトリクス集計の参照（GetMetrics）

```bash
grpcurl -plaintext \
  -d '{
    "tenant_id": "tenant-a",
    "period_start": "2026-03-08T09:00:00Z",
    "period_end": "2026-03-08T10:00:00Z"
  }' \
  localhost:8080 \
  authpulse.v1.aggregator.AggregatorService/GetMetrics
```

**期待するレスポンス（例）**:
```json
{
  "tenant_id": "tenant-a",
  "allow_count": "5",
  "deny_count": "12",
  "allow_rate": 0.294,
  "latency": {
    "p50_ms": "11",
    "p95_ms": "18",
    "p99_ms": "23"
  },
  "rate_limit_count": "0",
  "computed_at": "2026-03-08T10:00:01Z"
}
```

---

## 5. アラートの確認（ListAlerts）

```bash
grpcurl -plaintext \
  -d '{
    "tenant_id": "tenant-a",
    "include_resolved": false,
    "page_size": 20
  }' \
  localhost:8080 \
  authpulse.v1.aggregator.AggregatorService/ListAlerts
```

**期待するレスポンス（例）**:
```json
{
  "alerts": [
    {
      "id": "660e8400-...",
      "tenant_id": "tenant-a",
      "rule_name": "DenyThresholdExceeded",
      "severity": "ALERT_SEVERITY_HIGH",
      "detected_at": "2026-03-08T09:06:00Z",
      "related_user_id": "user-attacker",
      "detail": "5分間で DENY イベントが 12件（閾値: 10件）を超過しました",
      "is_resolved": false
    }
  ],
  "total_count": 1
}
```

---

## 6. Envoy設定変更デモ（JWT検証の有効化）

```bash
# envoy.yaml を編集して jwt_authn フィルタのコメントアウトを解除
vi docker/envoy.yaml

# Envoy をホットリスタートまたは再起動
docker compose restart envoy

# JWT なしでリクエストを送ると 401 が返ることを確認
grpcurl -plaintext -d '{"tenant_id": "tenant-a", ...}' localhost:8080 \
  authpulse.v1.collector.CollectorService/IngestEvent
# → 401 Unauthorized（Envoy が弾く）
```

これにより **Rust コードを一切変更せずに** JWT検証を有効化できることを示せる。

---

## 7. Prometheus メトリクス確認

```
http://localhost:9090/graph?g0.expr=authpulse_ingest_event_total
```

主要メトリクス:
| メトリクス名 | 説明 |
|------------|------|
| `authpulse_ingest_event_total` | IngestEvent 呼び出し総数 |
| `authpulse_ingest_event_duration_seconds` | IngestEvent レイテンシヒストグラム |
| `authpulse_alert_generated_total` | 生成アラート総数 |
| `authpulse_webhook_notification_total` | Webhook 送信総数 |

---

## 8. テスト実行

```bash
# ユニットテスト（インフラ不要）
cargo test --workspace

# リント
cargo clippy --workspace -- -D warnings

# フォーマット確認
cargo fmt --check --all

# 統合テスト（Docker Compose 起動必須）
docker compose -f docker/docker-compose.yml up -d
cargo test --test '*' -- --ignored  # integration tests
```

---

## 9. トラブルシューティング

| 症状 | 確認箇所 |
|------|---------|
| `connection refused` | `docker compose ps` でコンテナ稼働確認 |
| `INVALID_ARGUMENT` | リクエストの必須フィールドを確認 |
| アラートが生成されない | aggregator ログで検知ループの動作を確認 |
| Webhook 到達しない | `webhook_configs` テーブルに URL が登録されているか確認 |
