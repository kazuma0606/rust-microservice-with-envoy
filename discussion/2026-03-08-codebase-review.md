# コードベースレビュー: AuthPulse MVP

**日付**: 2026-03-08
**対象**: `services/collector`, `services/aggregator`, `docker/`, `proto/`
**規模**: Rustソース 52ファイル / 約2,418行

---

## 良い点

### アーキテクチャの完成度
ヘキサゴナルアーキテクチャ（ポート/アダプタパターン）を Rust で実装できている。

```
domain/       ← ビジネスルール・エンティティ（外部依存なし）
usecase/      ← アプリケーションロジック
entrypoint/   ← gRPC ハンドラ（inbound adapter）
gateway/      ← DB・外部API（outbound adapter）
```

依存の方向が `entrypoint → usecase → domain ← gateway` と正しく保たれており、多くのプロジェクトで崩れるところを守れている。

### 型安全性への投資
- `sqlx` のコンパイル時クエリ検証（`query!` マクロ）
- `tonic` + `prost` による proto 由来の型生成
- Rust の `Result`/`Option` による明示的なエラー処理

### オブザーバビリティの実装
Prometheus + OpenTelemetry OTLP の両方が稼働し、Envoy プロキシ設定まで揃っている。
E2E 検証済み: `authpulse_ingest_event_total{status="ok"} 13` を Prometheus で確認。

---

## 課題（重要度順）

### P1: テストがゼロ（最大の問題）

52ファイル・2,418行のコードに `#[cfg(test)]` が1行もない。

**影響**:
- リファクタリング・依存アップグレード・新機能追加のたびにリグレッションの恐怖
- 異常検知ロジック（`detect_anomaly.rs`）やアラートルール（`alert_rule.rs`）の正確性を誰も保証できない
- CI を回せない

**テスト可能な優先箇所**:

| 対象 | テスト種別 | 外部依存 |
|------|-----------|---------|
| `domain/alert_rule.rs` のルール評価 | ユニット | なし |
| `usecase/detect_anomaly.rs` の異常検知 | ユニット | なし |
| `gateway/postgres_*_repository.rs` | 統合（testcontainers） | PostgreSQL |
| `entrypoint/grpc_handler.rs` | 統合（tonic test client） | gRPC server |

---

### P2: `notify_webhook.rs` が未接続のデッドコード

```rust
// services/aggregator/src/usecase/notify_webhook.rs
#[allow(dead_code)]
pub struct NotifyWebhook { ... }
```

`allow(dead_code)` でコンパイルを通しているが、ユースケース層から呼ばれていない。
Webhook 通知はシステムの重要機能のはずが未接続のまま。

**選択肢**: 接続して機能完成させる、または一時削除してイシューに積む。宙ぶらりんが最も悪い。

---

### P3: `TenantId` の二重定義

collector と aggregator それぞれに同一の `TenantId` ドメイン型が定義されている。

**解決策**: 共有 crate の導入。

```
services/
  shared/           ← 新規作成
    Cargo.toml
    src/
      lib.rs
      tenant_id.rs
      error.rs
  collector/
    Cargo.toml      # shared = { path = "../shared" }
  aggregator/
    Cargo.toml      # shared = { path = "../shared" }
```

---

### P4: proto のタイムスタンプが `int64` (Unix ms)

```protobuf
// proto/authpulse/v1/collector.proto
int64 occurred_at = 4;  // Unix milliseconds ← 暗黙の慣習
```

`google.protobuf.Timestamp` を使えば言語をまたいだ相互運用が容易になる。
現状は「ミリ秒」という慣習がコメントにしか記録されていない。

```protobuf
// 推奨
import "google/protobuf/timestamp.proto";
google.protobuf.Timestamp occurred_at = 4;
```

---

### P5: `DOCKER_BUILD` 環境変数によるパス分岐

```rust
// services/collector/build.rs
let proto_base = if std::env::var("DOCKER_BUILD").is_ok() {
    format!("{}/../proto", manifest_dir)   // Docker
} else {
    format!("{}/../../proto", manifest_dir) // Local
};
```

CI 環境や他のビルダーで `DOCKER_BUILD` が設定されていると予期しない挙動になりうる。
`CARGO_MANIFEST_DIR` から常に相対パスで解決できれば分岐不要になる可能性がある。

---

### P6: gRPC リフレクション未搭載

`grpcurl` でのデバッグに `-proto` と `-import-path` フラグが必須になっている。
`tonic-reflection` を追加するだけで `grpcurl list` が使えるようになる。

```toml
# Cargo.toml
tonic-reflection = "0.12"
```

---

### P7: CI/CD パイプラインが存在しない

`.github/workflows/` ディレクトリ自体が存在しない。現状は開発者がローカルで手動確認するしかなく、以下のリスクがある。

**影響**:
- `cargo clippy -D warnings` を忘れたままプッシュできる（実際に今回も手動で回した）
- `cargo fmt` 未適用のコードが混入する
- PR マージ時にビルド壊れを検知できない
- Docker イメージのビルド成否が本番デプロイ前まで不明

**最小構成の CI（GitHub Actions）**:

```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [main]
  pull_request:

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt
      - uses: Swatinem/rust-cache@v2
        with:
          workspaces: services
      - name: Install protobuf-compiler
        run: sudo apt-get install -y protobuf-compiler
      - name: fmt
        run: cargo fmt --manifest-path services/Cargo.toml --all -- --check
      - name: clippy
        run: cargo clippy --manifest-path services/Cargo.toml --all-targets -- -D warnings
      - name: test
        run: cargo test --manifest-path services/Cargo.toml --all

  docker-build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Build collector image
        run: docker build -f services/collector/Dockerfile -t authpulse-collector:ci .
      - name: Build aggregator image
        run: docker build -f services/aggregator/Dockerfile -t authpulse-aggregator:ci .
```

**段階的な CD 追加案**:

| フェーズ | 内容 | トリガー |
|---------|------|---------|
| CD Phase 1 | Docker Hub / GHCR へイメージプッシュ | main マージ時 |
| CD Phase 2 | ステージング環境への自動デプロイ | main マージ時 |
| CD Phase 3 | E2E テスト（grpcurl スクリプト）実行 | デプロイ後 |
| CD Phase 4 | 本番デプロイ（手動承認ゲート付き） | タグプッシュ時 |

**注意点**: `sqlx` のコンパイル時クエリ検証は DB 接続が必要なため、CI では `SQLX_OFFLINE=true` + `sqlx prepare` で生成したキャッシュ（`.sqlx/`）をリポジトリに含める必要がある。

```bash
# ローカルで一度実行してコミット
cargo sqlx prepare --workspace -- --all-targets
```

---

## 改善ロードマップ

```
Week 1: テスト基盤 + CI 構築
  ├── domain層ユニットテスト (alert_rule, detect_anomaly)
  ├── testcontainers-postgres セットアップ
  ├── sqlx オフラインキャッシュ生成 (cargo sqlx prepare)
  └── CI ワークフロー (.github/workflows/ci.yml)
      fmt / clippy -D warnings / test / docker-build

Week 2: 機能完成
  ├── notify_webhook を接続 or 削除
  └── gRPC リフレクション追加

Week 3: リファクタリング
  ├── shared crate 作成 (TenantId, 共通エラー型)
  └── proto タイムスタンプを google.protobuf.Timestamp へ移行

Week 4+: 発展
  ├── CD パイプライン (GHCR へのイメージプッシュ)
  ├── E2E テスト (Docker Compose + grpcurl スクリプト)
  ├── JWT 認証の有効化 (Envoy 設定コメントアウト解除)
  └── レート制限の有効化 (Envoy 設定コメントアウト解除)
```

---

## 総評

「動くプロトタイプ」としての完成度は高い。アーキテクチャの判断は正しく、Rust の難しい部分（非同期、型システム、プロトコルバッファ、glibc 互換性）と格闘した形跡がある。

ただし「プロダクションに出せるか」という観点では **テスト不在**・**CI/CD なし**・**Webhook未接続** が致命的。次の一手は迷わず **CI 構築 + テスト追加** を同時に進めることを推奨する。CI があれば「テストが通らないとマージできない」という強制力が生まれ、品質の維持が自然に組み込まれる。
