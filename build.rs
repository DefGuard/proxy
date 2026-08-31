use vergen_git2::{Emitter, Git2};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // set VERGEN_GIT_SHA env variable based on git commit hash
    let git2 = Git2::builder().branch(true).sha(true).build();
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
            "DevicePostureCheckRequest",
            "InstanceInfoResponse",
            "NewDevice",
            "PasswordResetRequest",
        ])
        // Enable optional fields.
        .protoc_arg("--experimental_allow_proto3_optional")
        // Make all messages serde-serializable.
        .type_attribute(".", "#[derive(serde::Serialize,serde::Deserialize)]")
        // Legacy (pre-2.2) clients omit `selected_methods`; default it to an empty list.
        .field_attribute(
            "ClientMfaStartRequest.selected_methods",
            "#[serde(default)]",
        )
        // Protobuf enum values carry the enum name prefix to avoid package-scope
        // collisions, so the generated Rust variants all share a prefix that clippy
        // flags. Suppress it on the generated type.
        .type_attribute(
            "MfaStartRejectionReason",
            "#[allow(clippy::enum_variant_names)]",
        )
        // Compiling protos using path on build time.
        .compile_protos(
            &[
                "proto/v2/proxy.proto",
                "proto/v2/common.proto",
                "proto/enterprise/v2/posture/posture.proto",
                "proto/common/client_types.proto",
            ],
            &["proto"],
        )?;

    println!("cargo:rerun-if-changed=proto");
    Ok(())
}
