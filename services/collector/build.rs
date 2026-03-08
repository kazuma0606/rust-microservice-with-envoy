fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    // Docker: package at /app/collector/, proto at /app/proto/ → go up one level
    // Local:  package at services/collector/, proto at ../../proto/ → go up two levels
    let proto_base = if std::env::var("DOCKER_BUILD").is_ok() {
        format!("{}/../proto", manifest_dir)
    } else {
        format!("{}/../../proto", manifest_dir)
    };

    tonic_build::configure().compile_protos(
        &[&format!("{}/authpulse/v1/collector.proto", proto_base)],
        &[&proto_base],
    )?;
    Ok(())
}
