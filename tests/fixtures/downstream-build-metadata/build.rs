use std::env;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    for key in [
        "DEP_OPENSSL_INCLUDE",
        "DEP_OPENSSL_VERSION_NUMBER",
        "DEP_OPENSSL_BACKEND",
    ] {
        println!("cargo:rerun-if-env-changed={key}");
        assert_present(key);
    }

    let backend = env::var("DEP_OPENSSL_BACKEND")
        .expect("DEP_OPENSSL_BACKEND must be exported by the compatibility layer");
    assert_eq!(
        backend,
        selected_backend_marker(),
        "compatibility marker must match the selected backend feature",
    );
}

fn assert_present(key: &str) {
    assert!(
        env::var_os(key).is_some(),
        "{key} must be exported by the compatibility layer",
    );
}

fn selected_backend_marker() -> &'static str {
    match (
        env::var_os("CARGO_FEATURE_BACKEND_OPENSSL").is_some(),
        env::var_os("CARGO_FEATURE_BACKEND_BORING").is_some(),
    ) {
        (true, false) => "openssl",
        (false, true) => "boring",
        (true, true) => "invalid",
        (false, false) => "invalid",
    }
}