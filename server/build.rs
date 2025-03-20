use std::env;
use std::path::PathBuf;

fn main() {
    // 環境変数 `DOCKER_BUILD` がセットされていたら Docker 向けのパスを使用
    let proto_path = if env::var("DOCKER_BUILD").is_ok() {
        "proto/helloworld.proto"
    } else {
        "../proto/helloworld.proto"
    };

    println!("Compiling Protobuf: {}", proto_path);

    // 出力先パスを設定
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let descriptor_path = out_dir.join("helloworld_descriptor.bin");

    // FILE_DESCRIPTOR_SETを生成するように設定
    tonic_build::configure()
        .file_descriptor_set_path(&descriptor_path)
        .compile(&[proto_path], &[env::var("DOCKER_BUILD").map(|_| "proto").unwrap_or("../proto")])
        .unwrap();

    // もし必要なら、生成されたファイルを別の場所にコピー
    if env::var("DOCKER_BUILD").is_ok() {
        std::fs::copy(&descriptor_path, "proto/helloworld_descriptor.bin").unwrap();
    } else {
        std::fs::copy(&descriptor_path, "../proto/helloworld_descriptor.bin").unwrap();
    }
}