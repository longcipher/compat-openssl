use std::env;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    for key in [
        "DEP_OPENSSL_BACKEND",
        "DEP_OPENSSL_INCLUDE",
        "DEP_OPENSSL_VERSION_NUMBER",
    ] {
        println!("cargo:rerun-if-env-changed={key}");
    }

    let backend = required_env("DEP_OPENSSL_BACKEND");
    let include = required_env("DEP_OPENSSL_INCLUDE");
    let version = required_env("DEP_OPENSSL_VERSION_NUMBER");

    println!("cargo:rustc-env=COMPAT_OPENSSL_BACKEND={backend}");
    println!("cargo:rustc-env=COMPAT_OPENSSL_INCLUDE={include}");
    println!("cargo:rustc-env=COMPAT_OPENSSL_VERSION_NUMBER={version}");
}

fn required_env(key: &str) -> String {
    match env::var(key) {
        Ok(value) => value,
        Err(error) => panic!("{key} must be available for the example consumer: {error}"),
    }
}