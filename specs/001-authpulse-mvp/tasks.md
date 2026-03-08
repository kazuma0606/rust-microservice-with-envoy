# タスク一覧: AuthPulse MVP

**Input**: `specs/001-authpulse-mvp/` 配下の設計文書
**Prerequisites**: plan.md ✅ / spec.md ✅ / data-model.md ✅ / contracts/ ✅ / research.md ✅

**Organization**: ユーザーストーリー単位でフェーズを構成し、各ストーリーを独立して実装・テスト・デモ可能にする。

## Format: `[ID] [P?] [Story?] 説明 in パス`

- **[P]**: 並列実行可能（異なるファイル、未完了依存なし）
- **[Story]**: 対応ユーザーストーリー（US1〜US4）
- 各タスクに正確なファイルパスを記載

---

## Phase 1: セットアップ（共有インフラ）

**目的**: Cargo workspace と Docker 環境の初期化

- [x] T001 Cargo workspace 設定を作成する in `services/Cargo.toml`（workspace members: collector, aggregator）
- [x] T002 [P] collector クレートを初期化する in `services/collector/Cargo.toml`（tonic, prost, sqlx, tokio 依存関係を含む）
- [x] T003 [P] aggregator クレートを初期化する in `services/aggregator/Cargo.toml`（tonic, prost, sqlx, tokio 依存関係を含む）
- [x] T004 [P] proto ディレクトリ構造を作成する `proto/authpulse/v1/`（collector.proto と aggregator.proto をコントラクト仕様から転記）
- [x] T005 [P] Docker Compose のベース設定を作成する in `docker/docker-compose.yml`（collector, postgres, envoy サービス定義）
- [x] T006 [P] 環境変数テンプレートを作成する in `docker/.env.example`（DATABASE_URL, COLLECTOR_ADDR, AGGREGATOR_ADDR, DENY_THRESHOLD, CONSECUTIVE_FAILURE_THRESHOLD を含む）

---

## Phase 2: 基盤（全ユーザーストーリーをブロックする前提条件）

**目的**: 全サービスが依存するスキーマ・プロト生成・共有型を整備する

**⚠️ 重要**: この Phase が完了するまで US1〜US4 の実装に着手してはならない

- [x] T007 prost によるコード生成を設定する in `services/collector/build.rs`（proto/authpulse/v1/collector.proto を対象）
- [x] T008 [P] prost によるコード生成を設定する in `services/aggregator/build.rs`（proto/authpulse/v1/aggregator.proto を対象）
- [x] T009 auth_events テーブルのマイグレーションを作成する in `services/collector/migrations/20260308000001_auth_events.sql`
- [x] T010 [P] alerts テーブルのマイグレーションを作成する in `services/collector/migrations/20260308000002_alerts.sql`
- [x] T011 [P] webhook_configs テーブルのマイグレーションを作成する in `services/collector/migrations/20260308000003_webhook_configs.sql`
- [x] T012 TenantId 値オブジェクト（newtype）を実装する in `services/collector/src/domain/value_object/tenant_id.rs`（空文字列・64文字超のバリデーション含む）
- [x] T013 [P] TenantId 値オブジェクト（newtype）を実装する in `services/aggregator/src/domain/value_object/tenant_id.rs`
- [x] T014 [P] Decision 列挙型を実装する in `services/collector/src/domain/value_object/decision.rs`（ALLOW / DENY + proto 変換）
- [x] T015 DomainError 型を定義する in `services/collector/src/domain/error.rs`（ValidationError, InfrastructureError, NotFoundError）
- [x] T016 [P] DomainError 型を定義する in `services/aggregator/src/domain/error.rs`
- [x] T017 Envoy 基本設定を作成する in `docker/envoy.yaml`（gRPC ルーティング定義、JWT検証とレート制限はコメントアウト状態で雛形を含む）
- [x] T018 collector main.rs の骨格を作成する in `services/collector/src/main.rs`（DB接続プール初期化・sqlx マイグレーション実行・gRPC サーバー起動の構造のみ）
- [x] T019 [P] aggregator main.rs の骨格を作成する in `services/aggregator/src/main.rs`（同上）

**チェックポイント**: `cargo build --workspace` が通ること。ここから US1〜US4 を並列着手可能。

---

## Phase 3: User Story 1 — 認証認可イベントの受信と保存（Priority: P1）🎯 MVP

**Goal**: `IngestEvent` gRPC エンドポイントがイベントを受け取り、PostgreSQL に保存する

**Independent Test**: `grpcurl -d '{"tenant_id":"t1","user_id":"u1",...}' localhost:8080 authpulse.v1.collector.CollectorService/IngestEvent` が `event_id` を含むレスポンスを返す

### User Story 1 実装

- [x] T020 [P] [US1] AuthEvent エンティティを実装する in `services/collector/src/domain/entity/auth_event.rs`（全フィールド・バリデーションルール含む）
- [x] T021 [P] [US1] Tenant エンティティを実装する in `services/collector/src/domain/entity/tenant.rs`
- [x] T022 [US1] EventRepository Port trait を定義する in `services/collector/src/port/event_repository.rs`
- [x] T023 [US1] IngestEventUseCase を実装する in `services/collector/src/usecase/ingest_event.rs`（バリデーション→保存の順序。インフラ依存なし）
- [x] T024 [US1] PostgresEventRepository アダプターを実装する in `services/collector/src/adapter/postgres_event_repository.rs`
- [x] T025 [US1] gRPC DTO 変換ロジックを実装する in `services/collector/src/entrypoint/dto.rs`
- [x] T026 [US1] CollectorService gRPC ハンドラーを実装する in `services/collector/src/entrypoint/grpc_handler.rs`（DomainError → tonic Status マッピング含む）
- [x] T027 [US1] collector main.rs を完成させる in `services/collector/src/main.rs`（DI 完了・gRPC サーバー起動）

**チェックポイント**: この時点で User Story 1 が独立して動作可能。grpcurl でエンドツーエンドを検証する。

---

## Phase 4: User Story 2 — メトリクス集計の参照（Priority: P2）

**Goal**: `GetMetrics` gRPC エンドポイントがテナント・期間指定で集計値を返す

**Independent Test**: US1 でイベントを投入後に `GetMetrics` を呼び出し、ALLOW/DENY 数・成功率・レイテンシ分位点が正しく返る

### User Story 2 実装

- [x] T028 [P] [US2] MetricsSummary エンティティを実装する in `services/aggregator/src/domain/entity/metrics_summary.rs`
- [x] T029 [P] [US2] LatencyPercentiles 値オブジェクトを実装する in `services/aggregator/src/domain/value_object/latency_percentiles.rs`（データなし時は no_data=true）
- [x] T030 [US2] EventReadRepository Port trait を定義する in `services/aggregator/src/port/event_read_repository.rs`
- [x] T031 [US2] GetMetricsUseCase を実装する in `services/aggregator/src/usecase/get_metrics.rs`（インフラ依存なし、分位点計算含む）
- [x] T032 [US2] PostgresEventReadRepository アダプターを実装する in `services/aggregator/src/adapter/postgres_event_repository.rs`
- [x] T033 [US2] GetMetrics gRPC DTO 変換を実装する in `services/aggregator/src/entrypoint/dto.rs`
- [x] T034 [US2] AggregatorService gRPC ハンドラー（GetMetrics）を実装する in `services/aggregator/src/entrypoint/grpc_handler.rs`
- [x] T035 [US2] aggregator main.rs に GetMetrics ユースケースを組み込む in `services/aggregator/src/main.rs`

**チェックポイント**: User Story 1 と 2 が独立して動作可能。US1 のデータを使って US2 を検証する。

---

## Phase 5: User Story 3 — ルールベース異常検知とアラート参照（Priority: P3）

**Goal**: バックグラウンドループが異常パターンを検知して Alert を生成し、`ListAlerts` / `ResolveAlert` で参照・解決できる

**Independent Test**: DENY イベントを 12件投入後に最大5分待ち、`ListAlerts` が `DenyThresholdExceeded` アラートを返す

### User Story 3 実装

- [x] T036 [P] [US3] Alert エンティティを実装する in `services/aggregator/src/domain/entity/alert.rs`
- [x] T037 [P] [US3] AlertRuleName と AlertSeverity 値オブジェクトを実装する in `services/aggregator/src/domain/value_object/alert_rule.rs`
- [x] T038 [US3] AlertRepository Port trait を定義する in `services/aggregator/src/port/alert_repository.rs`
- [x] T039 [US3] DetectAnomalyUseCase を実装する in `services/aggregator/src/usecase/detect_anomaly.rs`（2ルール: DenyThresholdExceeded / ConsecutiveAuthFailure）
- [x] T040 [US3] ListAlertsUseCase を実装する in `services/aggregator/src/usecase/list_alerts.rs`
- [x] T041 [US3] ResolveAlertUseCase を実装する in `services/aggregator/src/usecase/resolve_alert.rs`
- [x] T042 [US3] PostgresAlertRepository アダプターを実装する in `services/aggregator/src/adapter/postgres_alert_repository.rs`（全クエリに `WHERE tenant_id = $1`）
- [x] T043 [US3] 異常検知バックグラウンドループを実装する in `services/aggregator/src/usecase/anomaly_detection_loop.rs`（tokio::spawn, 環境変数で閾値設定）
- [x] T044 [US3] ListAlerts と ResolveAlert gRPC ハンドラーを追加する in `services/aggregator/src/entrypoint/grpc_handler.rs`
- [x] T045 [US3] aggregator main.rs にバックグラウンドタスクとアラート関連ハンドラーを組み込む in `services/aggregator/src/main.rs`

**チェックポイント**: US1〜US3 が独立して動作可能。異常検知ループの5分サイクルをローカルで確認する。

---

## Phase 6: User Story 4 — Webhook 通知連携（Priority: P4）

**Goal**: アラート生成時に登録済み Webhook URL へ POST 通知を送信し、失敗時は最大3回リトライする

**Independent Test**: Webhook 受信用ローカルサーバー（例: `nc -l 8888`）を起動し、アラートトリガー後に POST リクエストが届く

### User Story 4 実装

- [x] T046 [P] [US4] WebhookConfig エンティティを実装する in `services/aggregator/src/domain/entity/webhook_config.rs`（https:// のみバリデーション）
- [x] T047 [US4] WebhookConfigRepository Port trait を定義する in `services/aggregator/src/port/webhook_config_repository.rs`
- [x] T048 [US4] Notifier Port trait を定義する in `services/aggregator/src/port/notifier.rs`
- [x] T049 [US4] UpsertWebhookConfigUseCase を実装する in `services/aggregator/src/usecase/upsert_webhook_config.rs`
- [x] T050 [US4] NotifyWebhookUseCase を実装する in `services/aggregator/src/usecase/notify_webhook.rs`（inactive なら送信スキップ）
- [x] T051 [US4] PostgresWebhookConfigRepository アダプターを実装する in `services/aggregator/src/adapter/postgres_webhook_config_repository.rs`（ON CONFLICT upsert）
- [x] T052 [US4] WebhookNotifier アダプターを実装する in `services/aggregator/src/adapter/webhook_notifier.rs`（reqwest, 最大3回指数バックオフリトライ）
- [x] T053 [US4] UpsertWebhookConfig gRPC ハンドラーを追加する in `services/aggregator/src/entrypoint/grpc_handler.rs`
- [x] T054 [US4] DetectAnomalyUseCase に Webhook 通知トリガーを追加する in `services/aggregator/src/usecase/detect_anomaly.rs`
- [x] T055 [US4] aggregator main.rs に WebhookNotifier と WebhookConfig 関連コンポーネントを DI する in `services/aggregator/src/main.rs`

**チェックポイント**: 全ユーザーストーリー（US1〜US4）が独立して動作可能。quickstart.md のシナリオ全体を通してデモ実行する。

---

## Phase N: 仕上げ & 横断的関心事

**目的**: 可観測性・コード品質・運用性の強化

- [x] T056 [P] Prometheus メトリクス計装を追加する in `services/collector/src/entrypoint/grpc_handler.rs`（`authpulse_ingest_event_total` カウンター、`authpulse_ingest_event_duration_seconds` ヒストグラム）
- [x] T057 [P] Prometheus メトリクス計装を追加する in `services/aggregator/src/entrypoint/grpc_handler.rs`（GetMetrics, ListAlerts, ResolveAlert のカウンター・ヒストグラム）
- [x] T058 [P] Prometheus メトリクス計装を追加する in `services/aggregator/src/usecase/anomaly_detection_loop.rs`（`authpulse_alert_generated_total`, `authpulse_webhook_notification_total`）
- [x] T059 [P] OpenTelemetry トレース初期化を実装する in `services/collector/src/main.rs`（OTLP エクスポーター設定、tonic インターセプターで trace_id 伝播）
- [x] T060 [P] OpenTelemetry トレース初期化を実装する in `services/aggregator/src/main.rs`
- [x] T061 docker/docker-compose.yml に Prometheus サービスと prometheus.yml 設定を追加する in `docker/docker-compose.yml` および `docker/prometheus.yml`
- [x] T062 [P] Envoy レート制限設定をコメントアウト形式で追加する in `docker/envoy.yaml`（テナント単位のローカルレート制限。コメントアウト解除でデモ可能）
- [x] T063 [P] Envoy JWT 検証設定をコメントアウト形式で追加する in `docker/envoy.yaml`（コメントアウト解除で JWT 検証有効化デモ可能）
- [x] T064 `cargo clippy --workspace -- -D warnings` で全警告を解消する（全ファイル対象）
- [x] T065 `cargo fmt --all` でフォーマットを統一する（全ファイル対象）
- [x] T066 quickstart.md のシナリオ全手順を実行して動作確認する（grpcurl コマンド、Envoy コメント解除デモ、Prometheus 確認を含む）
- [x] T067 [P] docker/docker-compose.yml の Dockerfile を最適化する（cargo-chef マルチステージビルド・バイナリ strip で本番イメージを軽量化）

---

## 依存関係 & 実行順序

### フェーズ依存関係

- **Phase 1（Setup）**: 依存なし — 即座に着手可能
- **Phase 2（Foundational）**: Phase 1 完了が必須 — 全 US をブロック
- **Phase 3〜6（US1〜US4）**: Phase 2 完了後に着手可能
  - US1 → US2 → US3 → US4 の順（US2 は US1 のデータを活用するため）
  - または US1 完了後に US2〜US4 を並列着手（チーム開発時）
- **Phase N（仕上げ）**: 必要な US フェーズ完了後

### ユーザーストーリー依存関係

- **US1（P1）**: Phase 2 完了後に開始可能 — 他 US への依存なし
- **US2（P2）**: Phase 2 完了後に開始可能 — US1 のデータ（auth_events テーブル）を利用
- **US3（P3）**: Phase 2 完了後に開始可能 — US2 の EventReadRepository を再利用
- **US4（P4）**: Phase 2 完了後に開始可能 — US3 の DetectAnomalyUseCase を拡張

### 各ストーリー内の順序

```
値オブジェクト [P] → エンティティ [P] → Port trait → UseCase → Adapter → Entrypoint → main.rs 組み込み
```

---

## 並列実行例

### Phase 2 の並列タスク

```bash
# 同時実行可能（独立ファイル）
T007 collector/build.rs
T008 aggregator/build.rs
T009 migrations/001_auth_events.sql
T010 migrations/002_alerts.sql
T011 migrations/003_webhook_configs.sql
T012 collector TenantId
T013 aggregator TenantId
T014 Decision enum
T017 Envoy yaml
```

### Phase 3（US1）の並列タスク

```bash
# 並列実行可能
T020 AuthEvent エンティティ
T021 Tenant エンティティ

# T020, T021 完了後に開始
T022 EventRepository Port trait

# T022 完了後に並列実行可能
T023 IngestEventUseCase
T024 PostgresEventRepository
```

### Phase N の並列タスク

```bash
# 全て独立して並列実行可能
T056 collector Prometheus 計装
T057 aggregator Prometheus 計装
T058 anomaly detection loop 計装
T059 collector OpenTelemetry
T060 aggregator OpenTelemetry
T062 Envoy レート制限設定
T063 Envoy JWT 設定
```

---

## 実装戦略

### MVP ファースト（US1 のみ）

1. Phase 1: Setup を完了する
2. Phase 2: Foundational を完了する（**クリティカルパス**）
3. Phase 3: US1 を完了する（T020〜T027）
4. **停止 & 検証**: grpcurl で IngestEvent をエンドツーエンドでテストする
5. デモ可能な状態でコミット

### インクリメンタルデリバリー

1. Setup + Foundational → 基盤完成
2. US1 追加 → IngestEvent が動作（MVP！）
3. US2 追加 → GetMetrics が動作（集計デモ可能）
4. US3 追加 → ListAlerts が動作（異常検知デモ可能）
5. US4 追加 → Webhook 通知が動作（完全デモ可能）
6. Polish → 本番品質に引き上げ

### チーム並列戦略（チーム開発時）

Phase 2 完了後:
- 開発者A: US1（collector 全体）
- 開発者B: US2（aggregator GetMetrics）
- Phase 2 の DB スキーマは共有のため事前確定が重要

---

## Notes

- `[P]` = 異なるファイル、依存なし（並列実行可能）
- `[USx]` = 対応するユーザーストーリー（トレーサビリティ）
- 各 US は独立して完成・テスト可能
- `.unwrap()` / `.expect()` は本番コードに使用しない（DomainError で代替）
- 全 Port trait のメソッドに `TenantId` を必須引数として含めること
- 各チェックポイントで `cargo build --workspace` が通ることを確認する
- Phase N の T064（clippy）は最後ではなく各フェーズ完了時にも随時実行することを推奨
