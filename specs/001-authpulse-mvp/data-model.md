# データモデル: AuthPulse MVP

**Date**: 2026-03-08
**Feature**: 001-authpulse-mvp
**Layer**: Domain エンティティ（インフラ実装詳細を含まない）

---

## エンティティ一覧

### 1. AuthEvent（認証認可イベント）

**概要**: システムに投入される最小記録単位。テナントが管理するサービスで
発生した認証認可の結果を表す。

**フィールド**:

| フィールド名 | 型 | 必須 | 説明 |
|------------|-----|------|------|
| `id` | UUID | 自動生成 | イベント一意識別子 |
| `tenant_id` | TenantId | MUST | テナント識別子（全クエリで使用） |
| `user_id` | String（≤128文字） | MUST | 認証対象ユーザーの識別子 |
| `service` | String（≤128文字） | MUST | 認可を実施したサービス名 |
| `resource` | String（≤256文字） | MUST | アクセス対象リソースのパス/名称 |
| `action` | String（≤64文字） | MUST | 実行しようとした操作（read/write/delete等） |
| `decision` | Decision | MUST | 認可結果: `ALLOW` または `DENY` |
| `reason_code` | String（≤64文字） | オプション | 拒否理由コード（例: `PERMISSION_DENIED`） |
| `latency_ms` | u64 | オプション | 認可処理レイテンシ（ミリ秒） |
| `source_ip` | String（≤45文字） | オプション | リクエスト元 IP アドレス（IPv4/IPv6） |
| `trace_id` | String（≤64文字） | オプション | 分散トレース ID |
| `timestamp` | DateTime<Utc> | MUST | イベント発生時刻（UTC） |
| `recorded_at` | DateTime<Utc> | 自動生成 | システム受信時刻（UTC） |

**バリデーションルール**:
- `tenant_id` は空文字列不可
- `decision` は `ALLOW` / `DENY` のいずれかのみ
- `timestamp` は未来すぎる値（システム時刻 +5分超）を拒否
- `user_id`, `service`, `resource`, `action` は空文字列不可

**状態遷移**: なし（イベントはイミュータブル）

---

### 2. MetricsSummary（メトリクス集計）

**概要**: 期間・テナントを軸に集計したスナップショット。
クエリ時にオンデマンドで計算するか、キャッシュする（MVP ではオンデマンド）。

**フィールド**:

| フィールド名 | 型 | 説明 |
|------------|-----|------|
| `tenant_id` | TenantId | 集計対象テナント |
| `period_start` | DateTime<Utc> | 集計期間開始 |
| `period_end` | DateTime<Utc> | 集計期間終了 |
| `allow_count` | u64 | ALLOW イベント数 |
| `deny_count` | u64 | DENY イベント数 |
| `allow_rate` | f64 | 認可成功率（0.0〜1.0） |
| `latency_p50_ms` | Option<u64> | レイテンシ中央値（ms） |
| `latency_p95_ms` | Option<u64> | レイテンシ 95パーセンタイル（ms） |
| `latency_p99_ms` | Option<u64> | レイテンシ 99パーセンタイル（ms） |
| `rate_limit_count` | u64 | レート制限到達（429）件数 |
| `computed_at` | DateTime<Utc> | 計算実行時刻 |

**補足**: `latency_*` は `latency_ms` を持つイベントのみで計算。
データなし時は `None`。

---

### 3. Alert（アラート）

**概要**: 異常検知ルールが発火した結果エンティティ。
生成後はイミュータブル（解決フラグのみ変更可能）。

**フィールド**:

| フィールド名 | 型 | 必須 | 説明 |
|------------|-----|------|------|
| `id` | UUID | 自動生成 | アラート一意識別子 |
| `tenant_id` | TenantId | MUST | 対象テナント |
| `rule_name` | AlertRuleName | MUST | 発火したルール名（列挙型） |
| `severity` | AlertSeverity | MUST | 重要度: `HIGH` / `MEDIUM` / `LOW` |
| `detected_at` | DateTime<Utc> | MUST | 検知時刻 |
| `related_user_id` | Option<String> | オプション | 関連ユーザー ID |
| `related_service` | Option<String> | オプション | 関連サービス名 |
| `detail` | String | MUST | 人間可読な詳細メッセージ |
| `is_resolved` | bool | MUST | 解決済みフラグ（初期値: false） |
| `resolved_at` | Option<DateTime<Utc>> | オプション | 解決時刻 |

**AlertRuleName（列挙）**:
- `DenyThresholdExceeded`: 5分間 DENY 件数 > 閾値（デフォルト 10件）
- `ConsecutiveAuthFailure`: 60秒以内に同一ユーザーが5回連続失敗

**AlertSeverity（列挙）**:
- `HIGH`: 即時対応を要する（DenyThresholdExceeded）
- `MEDIUM`: 要注意（ConsecutiveAuthFailure）
- `LOW`: 参考情報

**状態遷移**:
```
[生成] OPEN → [解決] RESOLVED
```
`is_resolved = true` かつ `resolved_at` 設定済みで解決済みとみなす。

---

### 4. WebhookConfig（Webhook設定）

**概要**: テナントが登録する通知先 URL の設定。

**フィールド**:

| フィールド名 | 型 | 必須 | 説明 |
|------------|-----|------|------|
| `id` | UUID | 自動生成 | 設定一意識別子 |
| `tenant_id` | TenantId | MUST | 対象テナント（ユニーク制約） |
| `url` | Url | MUST | 通知先 URL（https:// のみ許可） |
| `is_active` | bool | MUST | 有効フラグ（初期値: true） |
| `last_notified_at` | Option<DateTime<Utc>> | オプション | 最終通知成功時刻 |
| `created_at` | DateTime<Utc> | 自動生成 | 登録時刻 |

**バリデーションルール**:
- `url` は `https://` スキームのみ受け付ける
- テナントごとに1件のみ登録可能（`tenant_id` に UNIQUE 制約）

---

### 5. 値オブジェクト（Value Objects）

| 型名 | 内部表現 | バリデーション |
|------|---------|--------------|
| `TenantId` | String（newtype） | 空文字列不可、≤64文字 |
| `Decision` | enum（ALLOW / DENY） | 2値のみ |
| `AlertRuleName` | enum | 定義済みルールのみ |
| `AlertSeverity` | enum | HIGH / MEDIUM / LOW |
| `Url` | url::Url（newtype） | https:// のみ |

---

## DB スキーマ（PostgreSQL）

### auth_events テーブル

```sql
CREATE TABLE auth_events (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id    VARCHAR(64)  NOT NULL,
    user_id      VARCHAR(128) NOT NULL,
    service      VARCHAR(128) NOT NULL,
    resource     VARCHAR(256) NOT NULL,
    action       VARCHAR(64)  NOT NULL,
    decision     VARCHAR(8)   NOT NULL CHECK (decision IN ('ALLOW', 'DENY')),
    reason_code  VARCHAR(64),
    latency_ms   BIGINT,
    source_ip    VARCHAR(45),
    trace_id     VARCHAR(64),
    timestamp    TIMESTAMPTZ  NOT NULL,
    recorded_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_auth_events_tenant_timestamp
    ON auth_events (tenant_id, timestamp DESC);

CREATE INDEX idx_auth_events_tenant_user_timestamp
    ON auth_events (tenant_id, user_id, timestamp DESC);
```

### alerts テーブル

```sql
CREATE TABLE alerts (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id        VARCHAR(64)  NOT NULL,
    rule_name        VARCHAR(64)  NOT NULL,
    severity         VARCHAR(16)  NOT NULL CHECK (severity IN ('HIGH', 'MEDIUM', 'LOW')),
    detected_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    related_user_id  VARCHAR(128),
    related_service  VARCHAR(128),
    detail           TEXT         NOT NULL,
    is_resolved      BOOLEAN      NOT NULL DEFAULT FALSE,
    resolved_at      TIMESTAMPTZ
);

CREATE INDEX idx_alerts_tenant_resolved
    ON alerts (tenant_id, is_resolved, detected_at DESC);
```

### webhook_configs テーブル

```sql
CREATE TABLE webhook_configs (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id        VARCHAR(64)  NOT NULL UNIQUE,
    url              TEXT         NOT NULL,
    is_active        BOOLEAN      NOT NULL DEFAULT TRUE,
    last_notified_at TIMESTAMPTZ,
    created_at       TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);
```

---

## エンティティ関係図

```
Tenant (概念)
    │
    ├─── 1:N ──▶ AuthEvent
    ├─── 1:N ──▶ Alert
    └─── 1:1 ──▶ WebhookConfig

MetricsSummary ──(集計元)──▶ AuthEvent (期間フィルタ)
Alert ──(関連情報として参照)──▶ AuthEvent (直接FK なし、tenant_id で論理結合)
WebhookConfig ──(通知トリガー)──▶ Alert (生成イベント)
```

---

## クリーンアーキテクチャ レイヤーマッピング

| レイヤー | AuthEvent の扱い |
|---------|----------------|
| domain/entity | `AuthEvent` struct、`Decision` enum 定義 |
| domain/value_object | `TenantId`, `Decision`, `Url` newtype |
| usecase | `IngestEventUseCase`（domain エンティティのみ依存） |
| port | `EventRepository` trait（DB詳細なし） |
| adapter | `PostgresEventRepository`（sqlx 実装） |
| entrypoint | gRPC → DTO → `AuthEvent` への変換 |
