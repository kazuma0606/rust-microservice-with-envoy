# 実装計画: AuthPulse MVP

**Branch**: `001-authpulse-mvp` | **Date**: 2026-03-08 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `specs/001-authpulse-mvp/spec.md`

## Summary

認証認可イベントを収集・集計・異常検知する監視SaaSのMVPを構築する。
既存の Rust + Envoy gRPC 基盤を拡張し、クリーンアーキテクチャ（rusted-ca パターン）を
適用した2サービス構成（collector / aggregator）を実装する。
Envoy が JWT検証・レート制限・TLS終端を担い、Rust サービスはドメインロジックに集中する。

## Technical Context

**Language/Version**: Rust 1.75+ (stable)
**Primary Dependencies**: tonic 0.12, prost 0.13, sqlx 0.8, tokio 1.x, axum 0.7, metrics 0.23, opentelemetry 0.23
**Storage**: PostgreSQL 15+
**Testing**: cargo test, cargo clippy, Docker Compose 統合テスト
**Target Platform**: Linux server (Docker Compose → Kubernetes)
**Project Type**: gRPC マイクロサービス（2サービス: collector + aggregator）
**Performance Goals**: IngestEvent p99 ≤ 50ms at 1,000 RPS / GetMetrics p99 ≤ 200ms（30日集計）
**Constraints**: テナント分離必須 / .unwrap()本番禁止 / クリーンアーキテクチャ層分離 / Port経由DB抽象化
**Scale/Scope**: MVP 2サービス構成、PostgreSQL 1本、Docker Compose 完結

## Constitution Check

*ゲート: Phase 0 研究前に通過必須。Phase 1 設計後に再確認。*

| 原則 | 確認事項 | 状態 |
|------|---------|------|
| I. クリーンアーキテクチャ | domain/usecase/port/adapter/entrypoint の層構成 | ✅ PASS |
| II. Envoy責務分離 | JWT検証・レート制限・TLS は Envoy で実装（Rust側では実装しない） | ✅ PASS |
| III. テスト優先 | Port mock を使用したユニットテスト + 統合テスト計画あり | ✅ PASS |
| IV. 可観測性 | Prometheus メトリクス + OpenTelemetry トレース計画あり | ✅ PASS |
| V. DB抽象化 | `EventRepository` trait 経由で PostgreSQL を実装 | ✅ PASS |
| VI. セキュリティ | 全クエリに `tenant_id` フィルタ / シークレットは環境変数 | ✅ PASS |
| VII. 段階的分割 | Phase 1: collector+aggregator の2サービスで開始 | ✅ PASS |

**ゲート結果**: 全原則クリア。Phase 0 研究に進む。

## Project Structure

### Documentation (this feature)

```text
specs/001-authpulse-mvp/
├── plan.md              # This file (/speckit.plan)
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (gRPC proto)
└── tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
services/
├── collector/                   # イベント受信 gRPC サービス
│   ├── Cargo.toml
│   ├── build.rs
│   └── src/
│       ├── domain/
│       │   ├── entity/
│       │   │   ├── auth_event.rs
│       │   │   └── tenant.rs
│       │   └── value_object/
│       │       ├── decision.rs
│       │       └── tenant_id.rs
│       ├── usecase/
│       │   └── ingest_event.rs
│       ├── port/
│       │   └── event_repository.rs
│       ├── adapter/
│       │   └── postgres_event_repository.rs
│       ├── entrypoint/
│       │   ├── grpc_handler.rs
│       │   └── dto.rs
│       └── main.rs
│
├── aggregator/                  # 集計・検知・通知サービス
│   ├── Cargo.toml
│   ├── build.rs
│   └── src/
│       ├── domain/
│       │   ├── entity/
│       │   │   ├── metrics_summary.rs
│       │   │   ├── alert.rs
│       │   │   └── webhook_config.rs
│       │   └── value_object/
│       │       ├── alert_rule.rs
│       │       └── tenant_id.rs
│       ├── usecase/
│       │   ├── get_metrics.rs
│       │   ├── detect_anomaly.rs
│       │   ├── list_alerts.rs
│       │   └── notify_webhook.rs
│       ├── port/
│       │   ├── event_read_repository.rs
│       │   ├── alert_repository.rs
│       │   ├── webhook_config_repository.rs
│       │   └── notifier.rs
│       ├── adapter/
│       │   ├── postgres_event_repository.rs
│       │   ├── postgres_alert_repository.rs
│       │   └── webhook_notifier.rs
│       ├── entrypoint/
│       │   ├── grpc_handler.rs
│       │   └── dto.rs
│       └── main.rs
│
├── proto/                       # 共通 gRPC 定義
│   └── authpulse/
│       └── v1/
│           ├── collector.proto
│           └── aggregator.proto
│
└── docker/
    ├── docker-compose.yml
    ├── envoy.yaml
    └── migrations/
        ├── 001_auth_events.sql
        ├── 002_alerts.sql
        └── 003_webhook_configs.sql

tests/
├── contract/                    # gRPC コントラクトテスト
│   ├── test_ingest_event.rs
│   └── test_get_metrics.rs
├── integration/                 # Docker Compose 統合テスト
│   └── test_tenant_isolation.rs
└── unit/                        # Port mock ユニットテスト（各サービス内 tests/ に配置）
```

**Structure Decision**: マルチサービス構成（Rust workspace）を選択。
`services/` 配下に `collector` と `aggregator` を Cargo workspace として管理し、
`proto/` と `docker/` はワークスペースルートで共有する。
テナント分離・クリーンアーキテクチャ・段階的分割の全原則に対応可能な構造。

## Complexity Tracking

> 構成上の複雑性の正当化（Constitution Check で問題なしのため参考記録）

| 項目 | 理由 | 代替案を採用しない理由 |
|------|------|----------------------|
| Cargo workspace（2クレート） | collector と aggregator の独立デプロイを将来に備える | 単一バイナリでは段階的サービス分割が困難 |
| Port trait 抽象化 | テスト時にDBモック差替・TiDB移行を可能にする | 直接DB参照では Repository 変更時に全ユースケースを修正が必要 |
| docker/migrations/ 分離 | 複数サービスが同一 DB スキーマを共有するため | 各サービス内に置くとスキーマ重複が発生する |
