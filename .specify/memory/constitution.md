<!--
SYNC IMPACT REPORT
==================
Version change: (unversioned template) → 1.0.0
Modified principles: N/A (initial ratification)
Added sections:
  - I. クリーンアーキテクチャ原則
  - II. Envoy責務分離原則
  - III. テスト優先原則
  - IV. 可観測性原則
  - V. DB抽象化・拡張性原則
  - VI. セキュリティ設計原則
  - VII. マイクロサービス段階的分割原則
  - アーキテクチャ制約
  - 開発ワークフロー
  - Governance
Removed sections: N/A
Templates requiring updates:
  ✅ .specify/memory/constitution.md (this file — completed)
  ⚠ .specify/templates/plan-template.md (Constitution Check section should reference these 7 principles)
  ⚠ .specify/templates/spec-template.md (functional requirements should align with FR principles below)
  ⚠ .specify/templates/tasks-template.md (task categories should include observability, security, adapter tasks)
Follow-up TODOs:
  - TODO(RATIFICATION_DATE): Confirm exact project kickoff date if different from 2026-03-08
  - TODO(TEAM_REVIEW): Conduct first compliance review after MVP (collector + aggregator) is complete
-->

# AuthPulse Constitution
<!-- 認証認可イベント監視SaaS — Rust + Envoy + gRPC マイクロサービス基盤 -->

## コア原則

### I. クリーンアーキテクチャ原則

本プロジェクトは **rusted-ca** で確立したクリーンアーキテクチャを踏襲し、依存方向を厳格に固定する。

- **依存方向**: `entrypoint → usecase → domain` ← `adapter`（外向き矢印禁止）
- レイヤー構成は以下を遵守する：
  - `domain`: エンティティ・値オブジェクト・ドメインルール（外部依存ゼロ）
  - `usecase`: 集計・検知・通知などのアプリケーションロジック（domain のみ参照可）
  - `port`: Repository / Notifier の trait 定義（抽象インターフェース）
  - `adapter`: DB実装・通知実装・外部I/O実装（port を実装する）
  - `entrypoint`: gRPC ハンドラ・DTO変換・認証コンテキスト受け渡し
- `usecase` からインフラ詳細（SQL・HTTP・ファイルI/O）が直接見えてはならない。
- CQRS（Command/Query 責務分離）を適用し、書き込みと読み取りのユースケースを分離する。

**根拠**: 依存方向の固定はテスト可能性・DB切替・サービス分割の前提条件であり、
長期保守コストを最小化する。

### II. Envoy責務分離原則

インフラ横断関心事は Envoy に委譲し、Rust サービスはドメインロジックに集中する。

Envoy が担う責務（Rust 側で再実装してはならない）:
- TLS / mTLS 終端
- JWT 検証（前段フィルタ）
- レート制限（テナント単位・ユーザー単位）
- タイムアウト / リトライ制御
- アクセスログ / トレース連携

Rust サービスが担う責務:
- イベント集計・異常検知・通知送信などのドメインロジック
- gRPC インターフェース実装
- ビジネスルールに基づくバリデーション

**根拠**: Envoy 設定変更だけでレート制限・JWT検証を有効化できることがデモ価値であり、
Rust 側の複雑化を防ぐ。

### III. テスト優先原則

新機能はテストを先に書き、Red-Green-Refactor サイクルを遵守する。

- 新しいユースケース・エンティティ・アダプターは `cargo test` で単体テスト可能でなければならない。
- Port trait のモック実装を用意し、インフラなしでユースケースをテストできる構造を維持する。
- 統合テストは `tests/integration/` に配置し、Docker Compose 環境で実行可能とする。
- gRPC コントラクトテストを `tests/contract/` に配置し、プロトコル互換性を保証する。
- テストを書かずに実装してはならない（緊急修正を除き、事後にテストを追加する）。

**根拠**: Port 抽象化と組み合わせることで、インフラ起動なしの高速フィードバックループを実現する。

### IV. 可観測性原則

本番相当の可観測性をMVPから組み込む。

- 全 gRPC エンドポイントはリクエスト数・レイテンシ（p50/p95/p99）・エラー率を
  Prometheus メトリクスとして公開する。
- OpenTelemetry によるトレース伝播を実装し、`trace_id` をイベントに記録する。
- 認可イベント（ALLOW/DENY）・異常検知アラート・レート制限到達（429）を
  構造化ログとして出力する。
- `429` の件数もメトリクス化し、レート制限閾値の運用改善に使用する。
- ログレベルは環境変数で制御し、本番では `INFO` 以上をデフォルトとする。

**根拠**: 監視SaaSが自身の可観測性を欠いては信頼性の証明にならない。
また Envoy のアクセスログと組み合わせることで全レイヤーのトレーサビリティを確保する。

### V. DB抽象化・拡張性原則

データベース実装は Port 経由で抽象化し、切替可能な設計を維持する。

- MVP は PostgreSQL 1本で実装する（TiDB は使用しない）。
- Repository trait（Port）の実装として `PostgresRepository` を作成する。
- 同一 Port を実装する形で `TiDbRepository` アダプターを後工程で追加できる構造とする。
- DB実装の選択は設定ファイルまたは環境変数（`DATABASE_BACKEND`）で切替可能にする。
- スキーママイグレーションは SQLx / Diesel 等のツールで管理し、手動 DDL を禁止する。

**根拠**: MVPで複数DB実装を並行作成することは開発速度を損なう。
抽象化さえ正しければ、スケール要件確定後に TiDB アダプターを追加できる。

### VI. セキュリティ設計原則

認証認可監視SaaSとして、自システムのセキュリティを最優先とする。

- テナント分離: 全APIはリクエスト元 `tenant_id` を検証し、他テナントのデータに
  アクセスできてはならない。
- 最小権限: サービスアカウントは必要最小限の DB 権限のみ付与する。
- シークレット管理: 接続文字列・APIキーは環境変数または Secrets Manager 経由で注入し、
  コードやリポジトリにハードコードしてはならない。
- 入力バリデーション: gRPC エントリポイントで全フィールドの型・長さ・フォーマット検証を実施する。
- セキュリティヘッダー: HTTP エンドポイント（query-api）には CSP・HSTS・
  X-Frame-Options・X-Content-Type-Options を設定する。

**根拠**: 顧客の認証認可ログを扱うシステムが侵害された場合の被害は致命的であり、
セキュリティは後付けではなく設計に組み込む。

### VII. マイクロサービス段階的分割原則

サービス分割は段階的に行い、複雑性を段階的に導入する。

- **フェーズ1（MVP）**: `collector` + `aggregator` の2サービスで開始する。
- **フェーズ2**: `detector`（異常検知）を独立サービスとして追加する。
- **フェーズ3**: `notifier`（Webhook通知）・`query-api`（参照API）を分離する。
- サービス間通信は gRPC を使用し、プロトコルは `.proto` ファイルで契約管理する。
- 単一サービス内でユースケースが10を超えた場合、分割を検討する。

**根拠**: 最初から5サービスに分割すると運用・テスト・デバッグの複雑性が急増する。
段階分割により各フェーズで動作可能な状態を維持する。

## アーキテクチャ制約

### 技術スタック

| 分類 | 採用技術 | バージョン方針 |
|------|---------|-------------|
| 言語 | Rust | stable（MSRV: 1.75+） |
| gRPC | tonic + prost | 最新安定版 |
| HTTP | Axum | 最新安定版 |
| プロキシ | Envoy | 公式 Docker イメージ |
| DB（MVP） | PostgreSQL | 15+ |
| DB（拡張） | TiDB | MySQL互換モード |
| メトリクス | Prometheus + metrics-rs | — |
| トレース | OpenTelemetry（OTLP） | — |
| コンテナ | Docker Compose → Kubernetes | 段階移行 |

### パフォーマンス目標

- `IngestEvent` gRPC: p99 ≦ 50ms（シングルノード、1000 RPS）
- `GetMetrics` gRPC: p99 ≦ 200ms（30日集計クエリ）
- `ListAlerts` gRPC: p99 ≦ 100ms

### 禁止事項

- `usecase` レイヤーからの直接 DB アクセス（Port 経由必須）
- 環境変数以外でのシークレット注入
- テナント `id` 未検証でのデータ返却
- `unsafe` ブロックの使用（外部ライブラリ内部を除く）
- `.unwrap()` / `.expect()` の本番コードへの使用（テストコードは除く）

## 開発ワークフロー

### ブランチ戦略

- `main`: 常にデプロイ可能な状態を維持
- `feature/[###]-[name]`: 機能開発ブランチ
- `fix/[###]-[description]`: バグ修正ブランチ

### マージ要件

- 全 `cargo test` がパスすること
- `cargo clippy -- -D warnings` でエラーなし
- `cargo fmt --check` でフォーマット確認済み
- PR レビュー最低1名の承認
- Constitution Check の全項目確認済み

### 品質ゲート

1. ユニットテスト: Port モック使用、インフラ不要
2. 統合テスト: Docker Compose 環境で実行
3. コントラクトテスト: gRPC プロトコル互換性確認
4. 負荷テスト（フェーズ2以降）: 上記パフォーマンス目標を達成すること

## Governance

本 Constitution はプロジェクトの全開発判断の基準であり、他のすべての慣習に優先する。

### 改訂手順

1. 改訂案を PR として提出し、変更理由・影響範囲・移行計画を記述する。
2. チームメンバー全員の承認を得る。
3. `CONSTITUTION_VERSION` をセマンティックバージョニングに従い更新する：
   - MAJOR: 原則の削除・非互換な再定義
   - MINOR: 新原則の追加・既存原則の大幅拡張
   - PATCH: 表現改善・誤字修正・非意味的な変更
4. `LAST_AMENDED_DATE` を更新する。
5. 影響するテンプレートを同一PRで更新する。

### コンプライアンスレビュー

- MVP完了時（フェーズ1）に第1回コンプライアンスレビューを実施する。
- 各フェーズ完了時にレビューを実施し、原則への準拠を確認する。
- 原則に違反する実装は、マージ前にリファクタリングするか、
  Complexity Tracking セクションに正当化理由を記録する。

### 参照ガイダンス

- 実装詳細: `CLAUDE.md`（存在する場合）
- アーキテクチャ参考: [rusted-ca](https://github.com/kazuma0606/rusted-ca)
- 拡張計画: `idea/authpulse-mvp.md`

**Version**: 1.0.0 | **Ratified**: 2026-03-08 | **Last Amended**: 2026-03-08
