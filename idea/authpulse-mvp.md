# AuthPulse アイディアメモ

## 概要
`AuthPulse` は、認証認可イベントを収集し、可視化と異常検知を提供する監視SaaSです。  
目的は「誰が、いつ、どのリソースにアクセスし、許可/拒否されたか」をテナント単位で追跡し、セキュリティ運用を高速化することです。

## この題材が良い理由
- Envoy + gRPC + Rust の強みを活かせる
- HelloWorldから実用的なドメインへ自然に拡張できる
- 認証認可・監視・メトリクスというマイクロサービスらしい責務分離を設計できる

## MVP機能
1. 認証認可イベント収集 API
2. 基本メトリクス集計 API
3. ルールベース異常検知
4. 通知連携（Webhook）
5. テナント別の参照 API

## Rate limit方針
MVPでも導入する。理由は、監視系SaaSはイベント流入が急増しやすく、入口防御が必須だから。

- 初期実装は Envoy 側で実施する（アプリ側で複雑化しない）
- `ingest` API はテナント単位で厳しめに制御
- `query` API はユーザー単位で別枠制御
- `429` 件数もメトリクス化し、運用改善に使う

## 収集イベントの基本項目
- `tenant_id`
- `user_id`
- `service`
- `resource`
- `action`
- `decision` (`ALLOW` / `DENY`)
- `reason_code`
- `latency_ms`
- `source_ip`
- `trace_id`
- `timestamp`

## 主要メトリクス例
- 認可成功率（allow率）
- 拒否率（deny率）
- API レイテンシ（p50/p95/p99）
- ユーザー別・サービス別アクセス件数
- 時間帯別エラートレンド
- Rate limit到達件数（429）

## 異常検知ルール（初期）
- 5分間で `DENY` が閾値超過
- 同一ユーザーの短時間連続失敗
- 通常と異なる地域/IPからのアクセス急増
- 特定リソースへのアクセス偏り

## クリーンアーキテクチャ方針
前に作成したクリーンアーキテクチャの実践を踏襲し、依存方向を固定する。

- `domain`: エンティティ、値オブジェクト、ドメインルール
- `usecase`: 集計、検知、通知などのアプリケーションロジック
- `port`: Repository や Notifier の trait（抽象）
- `adapter`: DB実装、通知実装、外部I/O実装
- `entrypoint`: gRPC ハンドラ、DTO変換、認証文脈の受け渡し

Rustの実装では、`usecase` からインフラ詳細が見えない構造を徹底する。

## マイクロサービス構成案
- `collector`: イベント受信（gRPC）
- `aggregator`: 集計処理
- `detector`: 異常検知
- `notifier`: 通知送信
- `query-api`: ダッシュボード向け参照API

最初は `collector + aggregator` の2サービスで開始し、段階的に分割する。

## DB切替要件
要件としては必要。ただしMVPで複数DBを同時に作り込むのは避ける。

- MVP: Postgres 1本で実装し、Repositoryを抽象化
- 次段階: 同じ Port を実装する形で TiDB アダプタを追加
- 設定ファイルまたは環境変数でDB実装を切替可能にする

この方針なら、開発速度と将来の拡張性を両立できる。

## Kubernetes前提でのTiDB検討
Kubernetes運用を見据えるなら TiDB は有力候補（表記は `TiDB`）。

- メリット: 水平スケール、HA、MySQL互換
- 注意点: 初期運用コストが高めでMVPには重い場合がある
- 推奨: MVPはPostgres、スケール要件が見えたらTiDBへ拡張

## Envoyで担う責務
- TLS/mTLS 終端
- JWT 検証（前段）
- レート制限
- タイムアウト/リトライ
- ログ/トレース連携

Rust側はドメインロジック（集計・検知・通知）に集中する。

## APIイメージ
- `IngestEvent`: 認証認可イベントを受け取る
- `GetMetrics`: テナント/期間指定で集計を返す
- `ListAlerts`: 検知済みアラートを返す

## 開発ロードマップ（例）
1. 単一サービスでイベント受信と集計を実装
2. Rate limitとJWT検証をEnvoyで有効化
3. 異常検知ロジックを追加
4. 通知連携（Webhook）追加
5. サービス分割（collector/aggregator/detector）
6. DB切替対応（Postgres -> TiDBアダプタ追加）
7. 運用監視を強化（Prometheus, OpenTelemetry）

## デモとしての見せ場
- Envoy設定変更だけでRate limitやJWT検証を有効化できる
- gRPCイベント投入後、メトリクスとアラートが即時反映される
- クリーンアーキテクチャでDB差し替えが可能な構成を示せる
