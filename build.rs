use vergen_git2::{Emitter, Git2Builder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // set VERGEN_GIT_SHA env variable based on git commit hash
    let git2 = Git2Builder::default().branch(true).sha(true).build()?;
    Emitter::default().add_instructions(&git2)?.emit()?;

    // compiling protos using path on build time
    let mut config = prost_build::Config::new();
    // enable optional fields
    config.protoc_arg("--experimental_allow_proto3_optional");
    // Make all messages serde-serializable
    config.type_attribute(".", "#[derive(serde::Serialize,serde::Deserialize)]");
    tonic_build::configure().compile_protos_with_config(
        config,
        &["proto/core/proxy.proto"],
        &["proto/core"],
    )?;
    println!("cargo:rerun-if-changed=proto");
    Ok(())
}
