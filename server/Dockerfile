# Rust の公式イメージをベースにする
FROM rust:latest as builder

# 必要なパッケージをインストール（protocを含む）
RUN apt-get update && apt-get install -y \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Cargo.toml と Cargo.lock をコピー
COPY server/Cargo.toml server/Cargo.lock ./

# 依存関係を先にダウンロードしてキャッシュを活用
RUN cargo fetch

# `proto/` をコンテナ内にコピー
COPY proto/ proto/

# ソースコードをコピー
COPY server/ .

# 必要なディレクトリを作成
RUN mkdir -p proto

# 事前にプロトコルバッファをコンパイルして descriptor.bin を生成
RUN cd proto && protoc --include_imports --descriptor_set_out=helloworld_descriptor.bin helloworld.proto

# バイナリをビルド
ENV DOCKER_BUILD=1
RUN cargo build --release

# Debian bookworm を使用
FROM debian:bookworm-slim

WORKDIR /app

# 必要なディレクトリを作成
RUN mkdir -p proto

# 必要なバイナリとファイルをコピー
COPY --from=builder /app/target/release/server /app/server
COPY --from=builder /app/proto/helloworld_descriptor.bin /app/proto/

# サーバーを実行
CMD ["/app/server"]