fn main() -> Result<(), Box<dyn std::error::Error>> {
    // compiling protos using path on build time
    let mut config = prost_build::Config::new();
    // enable optional fields
    config.protoc_arg("--experimental_allow_proto3_optional");
    // Make all messages serde-serializable
    config.type_attribute(".", "#[derive(serde::Serialize,serde::Deserialize)]");
    tonic_build::configure().compile_with_config(
        config,
        &["proto/core/proxy.proto"],
        &["proto/core"],
    )?;
    println!("cargo:rerun-if-changed=proto");
    Ok(())
}
