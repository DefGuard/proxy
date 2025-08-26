use defguard_version::Version;

pub const MIN_CORE_VERSION: Version = Version::new(1, 6, 0);

/// Checks if the core version meets minimum version requirements.
pub(crate) fn is_core_version_supported(core_version: Option<&Version>) -> bool {
    let Some(core_version) = core_version else {
        error!("Missing core component version information. This most likely means that core component uses unsupported version.");
		return false;
    };
    if core_version < &MIN_CORE_VERSION {
        error!("Core version {core_version} is not supported. Minimal supported core version is {MIN_CORE_VERSION}.");
		return false;
    }

    info!("Core version {core_version} is supported");
	true
}
