#![allow(dead_code)]

#[path = "../build.rs"]
mod build_script;

#[test]
fn selects_the_openssl_backend_when_only_backend_openssl_is_enabled() {
    assert_eq!(
        build_script::select_backend(true, false),
        Ok(build_script::BackendSelection::Openssl),
    );
}

#[test]
fn selects_the_boring_backend_when_only_backend_boring_is_enabled() {
    assert_eq!(
        build_script::select_backend(false, true),
        Ok(build_script::BackendSelection::Boring),
    );
}

#[test]
fn rejects_enabling_both_backends() {
    assert_eq!(
        build_script::select_backend(true, true),
        Err(build_script::BackendSelectionError::BothEnabled),
    );
}

#[test]
fn rejects_builds_without_any_backend_enabled() {
    assert_eq!(
        build_script::select_backend(false, false),
        Err(build_script::BackendSelectionError::NoBackendEnabled),
    );
}
