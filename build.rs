use vergen_git2::{Emitter, Git2Builder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // set VERGEN_GIT_SHA env variable based on git commit hash
    let git2 = Git2Builder::default().branch(true).sha(true).build()?;
    Emitter::default().add_instructions(&git2)?.emit()?;

    tonic_prost_build::configure()
        // These types contain sensitive data.
        .skip_debug([
            "ActivateUserRequest",
            "AuthInfoResponse",
            "AuthenticateRequest",
            "AuthenticateResponse",
            "ClientMfaFinishResponse",
            "CodeMfaSetupStartResponse",
            "CodeMfaSetupFinishResponse",
            "CoreRequest",
            "CoreResponse",
            "DeviceConfigResponse",
            "InstanceInfoResponse",
            "NewDevice",
            "PasswordResetRequest",
        ])
        // Enable optional fields.
        .protoc_arg("--experimental_allow_proto3_optional")
        // Make all messages serde-serializable.
        .type_attribute(".", "#[derive(serde::Serialize,serde::Deserialize)]")
        // Compiling protos using path on build time.
        .compile_protos(&["proto/core/proxy.proto"], &["proto/core"])?;

    println!("cargo:rerun-if-changed=proto");
    Ok(())
}
