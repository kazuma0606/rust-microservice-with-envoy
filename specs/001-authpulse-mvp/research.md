# Phase 0 研究レポート: AuthPulse MVP

**Date**: 2026-03-08
**Feature**: 001-authpulse-mvp
**ステータス**: 完了（NEEDS CLARIFICATION なし）

---

## 1. Rust gRPC スタック選定

**Decision**: tonic 0.12 + prost 0.13

**Rationale**:
- tonic は Rust エコシステムで最も成熟した gRPC ライブラリ（tokio 非同期モデルと直結）
- prost は `.proto` → Rust コード生成（build.rs での統合が確立済み）
- 既存リポジトリ（rusted-ca）も同スタックを参照可能
- HTTP/2 ストリーミングへの将来拡張が容易

**Alternatives considered**:
- `grpcio`（C バインディングで環境依存が高い → 不採用）
- 独自 HTTP/2 実装（工数過多 → 不採用）

---

## 2. 非同期ランタイム

**Decision**: tokio 1.x（マルチスレッドスケジューラ）

**Rationale**:
- tonic / sqlx / reqwest すべてが tokio ベース
- `#[tokio::main]` で最小設定から起動可能
- 本番環境での実績が豊富

**Alternatives considered**:
- async-std（tokio との互換性問題が多い → 不採用）

---

## 3. DB アクセス層

**Decision**: sqlx 0.8（compile-time クエリ検証）

**Rationale**:
- `query!` / `query_as!` マクロで SQL をコンパイル時に検証 → 実行時エラー低減
- PostgreSQL ドライバが組み込み（追加バインディング不要）
- マイグレーションツール（`sqlx migrate`）が標準付属
- `sqlx::Pool<Postgres>` をアダプター層に隠蔽し、Port trait から DB詳細を隠せる

**Alternatives considered**:
- Diesel（マクロ複雑性・非同期対応が間接的 → 不採用）
- SeaORM（ORM オーバーヘッドで Port 抽象化と重複する → 不採用）

---

## 4. メトリクス公開

**Decision**: metrics 0.23 クレート + metrics-exporter-prometheus

**Rationale**:
- `metrics::counter!` / `metrics::histogram!` マクロで計装を統一
- バックエンドを Prometheus エクスポーターに差替可能（将来 StatsD 等へ変更容易）
- OpenTelemetry Metrics との共存が可能

**Alternatives considered**:
- prometheus クレート直接（metrics クレートの抽象化なしで依存が密結合 → 不採用）

---

## 5. 分散トレース

**Decision**: opentelemetry 0.23 + opentelemetry-otlp（OTLP/gRPC）

**Rationale**:
- tonic インターセプターで `trace_id` を伝播可能
- OTLP はベンダー中立（Jaeger/Grafana Tempo/Datadog 対応）
- `trace_id` をイベントエンティティに直接埋め込み、ログと紐付け可能

---

## 6. Envoy 設定パターン

**Decision**: Envoy 外部設定（`envoy.yaml`）+ Docker Compose で管理

JWT 検証設定（コメントアウトで初期無効）:
```yaml
http_filters:
  - name: envoy.filters.http.jwt_authn
    typed_config:
      "@type": type.googleapis.com/envoy.extensions.filters.http.jwt_authn.v3.JwtAuthentication
      providers:
        authpulse:
          issuer: "https://your-auth-provider"
          remote_jwks:
            http_uri:
              uri: "https://your-auth-provider/.well-known/jwks.json"
```

レート制限設定:
```yaml
http_filters:
  - name: envoy.filters.http.local_ratelimit
    typed_config:
      "@type": type.googleapis.com/envoy.extensions.filters.http.local_rate_limit.v3.LocalRateLimit
      token_bucket:
        max_tokens: 1000          # テナント単位で調整
        tokens_per_fill: 100
        fill_interval: 1s
```

**Rationale**: Envoy 設定変更のみで JWT/レート制限を有効化できるデモ価値を実現。
Rust コードへの影響ゼロ。

---

## 7. 異常検知ルール実行パターン

**Decision**: aggregator サービス内のバックグラウンドタスク（tokio::spawn）

**Rationale**:
- MVP では外部スケジューラ（Kubernetes CronJob 等）不要
- `tokio::time::interval` で5分ごとに評価ループを実行
- スケール要件が明確化した段階でイベントドリブン（Kafka 等）へ移行可能

**Alternatives considered**:
- cron クレート（プロセス外依存不要なので tokio で十分 → 不採用）
- Kafka/NATS（MVP 段階では複雑性過多 → 不採用）

---

## 8. Webhook 通知実装

**Decision**: reqwest 0.12（非同期 HTTP クライアント）+ 指数バックオフリトライ

**Rationale**:
- tokio エコシステムと親和性が高い
- `reqwest::Client` を Notifier Port の実装として隠蔽
- リトライロジックは `tower::retry` または独自実装（MVP では簡易実装）

---

## 9. マイグレーション管理

**Decision**: sqlx-migrate（`sqlx migrate run`）

マイグレーションファイル配置:
```
docker/migrations/
├── 20260308000001_auth_events.sql
├── 20260308000002_alerts.sql
└── 20260308000003_webhook_configs.sql
```

起動時に `sqlx::migrate!()` マクロでマイグレーションを自動適用する。

---

## 10. テナント分離実装パターン

**Decision**: 全 SQL クエリに `WHERE tenant_id = $1` を強制

**Rationale**:
- Row Level Security (PostgreSQL RLS) は MVP では過剰
- アダプター層（PostgresXxxRepository）で全クエリに `tenant_id` を引数として要求
- Port trait のシグネチャに `TenantId` を含め、省略不可能にする

```rust
// Port trait の例
#[async_trait]
pub trait EventRepository: Send + Sync {
    async fn save(&self, tenant_id: TenantId, event: AuthEvent) -> Result<(), DomainError>;
    async fn find_by_tenant(
        &self,
        tenant_id: TenantId,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<AuthEvent>, DomainError>;
}
```

---

## 11. 未解決事項

なし — 全 NEEDS CLARIFICATION 項目は意思決定済み。

---

## 12. 採用技術スタック最終確認

| 分類 | 採用 | バージョン |
|------|------|----------|
| gRPC | tonic + prost | 0.12 / 0.13 |
| 非同期 | tokio | 1.x |
| DB | sqlx + PostgreSQL | 0.8 / 15+ |
| メトリクス | metrics + metrics-exporter-prometheus | 0.23 |
| トレース | opentelemetry-otlp | 0.23 |
| HTTP クライアント | reqwest | 0.12 |
| Proxy | Envoy | 最新安定版 |
| コンテナ | Docker Compose | V2 |
