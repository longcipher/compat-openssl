use std::{env, path::PathBuf};

// Intentional OpenSSL metadata whitelist forwarded by the compatibility layer.
const OPENSSL_METADATA_WHITELIST: [(&str, &str); 5] = [
    ("DEP_REAL_OPENSSL_INCLUDE", "include"),
    ("DEP_REAL_OPENSSL_VERSION_NUMBER", "version_number"),
    ("DEP_REAL_OPENSSL_LIBRESSL", "libressl"),
    ("DEP_REAL_OPENSSL_LIBRESSL_VERSION_NUMBER", "libressl_version_number"),
    ("DEP_REAL_OPENSSL_CONF", "conf"),
];

const BORINGSSL_INCLUDE_ENV: &str = "DEP_BORINGSSL_INCLUDE";
const BORINGSSL_ROOT_ENV: &str = "DEP_BORINGSSL_ROOT";

const BORINGSSL_METADATA_WHITELIST: [(&str, &str); 1] = [(BORINGSSL_INCLUDE_ENV, "include")];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BackendSelection {
    Boring,
    Openssl,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BackendSelectionError {
    BothEnabled,
    NoBackendEnabled,
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    for (env_key, _) in OPENSSL_METADATA_WHITELIST {
        println!("cargo:rerun-if-env-changed={env_key}");
    }

    for (env_key, _) in BORINGSSL_METADATA_WHITELIST {
        println!("cargo:rerun-if-env-changed={env_key}");
    }

    println!("cargo:rerun-if-env-changed={BORINGSSL_ROOT_ENV}");

    match select_backend(
        env::var_os("CARGO_FEATURE_BACKEND_OPENSSL").is_some(),
        env::var_os("CARGO_FEATURE_BACKEND_BORING").is_some(),
    ) {
        Ok(BackendSelection::Boring) => relay_boring_as_ossl111(),
        Ok(BackendSelection::Openssl) => relay_real_openssl(),
        Err(error) => panic!("{}", error.message()),
    }
}

pub fn select_backend(
    has_backend_openssl: bool,
    has_backend_boring: bool,
) -> Result<BackendSelection, BackendSelectionError> {
    match (has_backend_openssl, has_backend_boring) {
        (true, false) => Ok(BackendSelection::Openssl),
        (false, true) => Ok(BackendSelection::Boring),
        (true, true) => Err(BackendSelectionError::BothEnabled),
        (false, false) => Err(BackendSelectionError::NoBackendEnabled),
    }
}

impl BackendSelectionError {
    fn message(self) -> &'static str {
        match self {
            Self::BothEnabled => "backend-openssl and backend-boring are mutually exclusive",
            Self::NoBackendEnabled => "either backend-openssl or backend-boring must be enabled",
        }
    }
}

fn relay_real_openssl() {
    for (env_key, cargo_key) in OPENSSL_METADATA_WHITELIST {
        forward_if_present(env_key, cargo_key);
    }

    println!("cargo:backend=openssl");
}

fn relay_boring_as_ossl111() {
    if !forward_if_present(BORINGSSL_INCLUDE_ENV, "include") {
        forward_boring_include_from_root();
    }

    println!("cargo:version_number=1010100f");
    println!("cargo:backend=boring");
}

fn forward_boring_include_from_root() {
    if let Some(include_dir) = boring_include_from_root() {
        println!("cargo:include={}", include_dir.to_string_lossy());
    }
}

fn boring_include_from_root() -> Option<PathBuf> {
    let root = env::var_os(BORINGSSL_ROOT_ENV)?;
    let include_dir = PathBuf::from(root).join("boringssl").join("include");
    include_dir.is_dir().then_some(include_dir)
}

fn forward_if_present(env_key: &str, cargo_key: &str) -> bool {
    if let Some(value) = env::var_os(env_key) {
        println!("cargo:{cargo_key}={}", value.to_string_lossy());
        return true;
    }

    false
}
