use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use cucumber::{World as _, given, then, when};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Backend {
    Boring,
    Openssl,
}

impl Backend {
    const ALL: [Self; 2] = [Self::Openssl, Self::Boring];

    const fn cargo_feature(self) -> &'static str {
        match self {
            Self::Boring => "backend-boring",
            Self::Openssl => "backend-openssl",
        }
    }

    const fn compatibility_marker(self) -> &'static str {
        match self {
            Self::Boring => "boring",
            Self::Openssl => "openssl",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Fixture {
    BuildMetadata,
    NativeTls,
    PostgresNativeTls,
    ExampleConsumer,
    OpensslExactPin,
    ReqwestNativeTls,
    OpensslSys,
    TokioNativeTls,
}

impl Fixture {
    const fn manifest_relative_path(self) -> &'static str {
        match self {
            Self::BuildMetadata => "tests/fixtures/downstream-build-metadata/Cargo.toml",
            Self::ExampleConsumer => "examples/consumer-app/Cargo.toml",
            Self::NativeTls => "tests/fixtures/downstream-native-tls/Cargo.toml",
            Self::PostgresNativeTls => "tests/fixtures/downstream-postgres-native-tls/Cargo.toml",
            Self::OpensslExactPin => "tests/fixtures/downstream-openssl-exact-pin/Cargo.toml",
            Self::ReqwestNativeTls => "tests/fixtures/downstream-reqwest-native-tls/Cargo.toml",
            Self::OpensslSys => "tests/fixtures/downstream-openssl-sys/Cargo.toml",
            Self::TokioNativeTls => "tests/fixtures/downstream-tokio-native-tls/Cargo.toml",
        }
    }

    fn manifest_path(self) -> PathBuf {
        repo_root().join(self.manifest_relative_path())
    }
}

#[derive(Debug, Default, cucumber::World)]
struct CompatWorld {
    current_fixture: Option<Fixture>,
    expected_error: Option<&'static str>,
    last_runs: Vec<CommandOutcome>,
    selected_backend: Option<Backend>,
}

#[derive(Debug)]
struct CommandOutcome {
    context: String,
    status_code: Option<i32>,
    success: bool,
    stdout: String,
    stderr: String,
}

impl CommandOutcome {
    fn combined_output(&self) -> String {
        format!("{}\n{}", self.stdout, self.stderr)
    }
}

impl CompatWorld {
    fn assert_all_succeeded(&self) {
        assert!(
            !self.last_runs.is_empty(),
            "expected the scenario to run at least one cargo command",
        );

        for outcome in &self.last_runs {
            assert!(
                outcome.success,
                "cargo command failed: {}\nstatus: {:?}\nstdout:\n{}\nstderr:\n{}",
                outcome.context, outcome.status_code, outcome.stdout, outcome.stderr,
            );
        }
    }

    fn assert_failure_contains_expected_error(&self) {
        assert!(
            self.last_runs.len() == 1,
            "expected a single failing cargo command, saw {}",
            self.last_runs.len(),
        );

        if let Some(outcome) = self.last_runs.first() {
            assert!(
                !outcome.success,
                "cargo command unexpectedly succeeded: {}\nstdout:\n{}\nstderr:\n{}",
                outcome.context, outcome.stdout, outcome.stderr,
            );

            if let Some(expected_error) = self.expected_error {
                let combined = outcome.combined_output();
                assert!(
                    combined.contains(expected_error),
                    "cargo command did not report the expected error: {}\nexpected substring: {}\nstdout:\n{}\nstderr:\n{}",
                    outcome.context,
                    expected_error,
                    outcome.stdout,
                    outcome.stderr,
                );
            }
        }
    }

    fn assert_resolution_uses_repo_patch(&self) {
        self.assert_all_succeeded();

        let expected_path =
            repo_root().join("crates").join("openssl").to_string_lossy().into_owned();

        let tree_results: Vec<&CommandOutcome> = self
            .last_runs
            .iter()
            .filter(|outcome| outcome.context.contains("cargo tree"))
            .collect();

        assert!(
            !tree_results.is_empty(),
            "expected cargo tree output when verifying patch resolution",
        );

        for outcome in tree_results {
            let combined = outcome.combined_output();
            assert!(
                combined.contains("openssl v0.10.76"),
                "cargo tree did not include the patched openssl crate: {}\nstdout:\n{}\nstderr:\n{}",
                outcome.context,
                outcome.stdout,
                outcome.stderr,
            );
            assert!(
                combined.contains(&expected_path),
                "cargo tree did not resolve openssl from this repository: {}\nexpected path fragment: {}\nstdout:\n{}\nstderr:\n{}",
                outcome.context,
                expected_path,
                outcome.stdout,
                outcome.stderr,
            );
        }
    }

    fn build_workspace(&mut self, backend: Backend) {
        self.selected_backend = Some(backend);
        self.expected_error = None;
        self.last_runs = vec![run_repo_cargo([
            "build".to_owned(),
            "--workspace".to_owned(),
            "--no-default-features".to_owned(),
            "--features".to_owned(),
            backend.cargo_feature().to_owned(),
        ])];
    }

    fn build_workspace_with_defaults(&mut self) {
        self.current_fixture = None;
        self.selected_backend = Some(Backend::Boring);
        self.expected_error = None;
        self.last_runs = vec![run_repo_cargo(["build".to_owned(), "--workspace".to_owned()])];
    }

    fn build_fixture(&mut self, fixture: Fixture, backend: Backend) {
        self.current_fixture = Some(fixture);
        self.selected_backend = Some(backend);
        self.expected_error = None;
        self.last_runs = vec![run_fixture_build(fixture, backend)];
    }

    fn build_fixture_for_both_backends(&mut self, fixture: Fixture) {
        self.current_fixture = Some(fixture);
        self.selected_backend = None;
        self.expected_error = None;
        self.last_runs.clear();

        for backend in Backend::ALL {
            self.last_runs.push(run_fixture_build(fixture, backend));
            self.last_runs.push(run_fixture_tree(fixture, backend));
        }
    }

    fn build_with_both_backends(&mut self) {
        self.current_fixture = None;
        self.selected_backend = None;
        self.expected_error = Some("backend-openssl and backend-boring are mutually exclusive");
        self.last_runs = vec![run_repo_cargo([
            "check".to_owned(),
            "-p".to_owned(),
            "openssl".to_owned(),
            "--no-default-features".to_owned(),
            "--features".to_owned(),
            "backend-openssl,backend-boring".to_owned(),
        ])];
    }

    fn build_without_backend(&mut self) {
        self.current_fixture = None;
        self.selected_backend = None;
        self.expected_error = Some("either backend-openssl or backend-boring must be enabled");
        self.last_runs = vec![run_repo_cargo([
            "check".to_owned(),
            "-p".to_owned(),
            "openssl".to_owned(),
            "--no-default-features".to_owned(),
        ])];
    }
}

#[given("the compatibility crates expose a backend-openssl feature")]
fn compatibility_crates_expose_backend_openssl_feature(_: &mut CompatWorld) {
    assert_backend_feature_declared(Backend::Openssl);
}

#[given("the compatibility crates expose a backend-boring feature")]
fn compatibility_crates_expose_backend_boring_feature(_: &mut CompatWorld) {
    assert_backend_feature_declared(Backend::Boring);
}

#[given("the compatibility crates default to backend-boring")]
fn compatibility_crates_default_to_backend_boring(_: &mut CompatWorld) {
    assert_default_backend(Backend::Boring);
}

#[given("backend-openssl and backend-boring are mutually exclusive")]
fn backend_features_are_mutually_exclusive(world: &mut CompatWorld) {
    world.expected_error = Some("backend-openssl and backend-boring are mutually exclusive");
}

#[given("backend selection is required once default features are disabled")]
fn backend_selection_is_required_once_defaults_are_disabled(world: &mut CompatWorld) {
    world.expected_error = Some("either backend-openssl or backend-boring must be enabled");
}

#[given("the downstream-native-tls fixture patches the openssl crates from this repository")]
fn downstream_native_tls_fixture_patches_repo(world: &mut CompatWorld) {
    assert_fixture_patches_repo(Fixture::NativeTls);
    world.current_fixture = Some(Fixture::NativeTls);
}

#[given("the downstream-tokio-native-tls fixture patches the openssl crates from this repository")]
fn downstream_tokio_native_tls_fixture_patches_repo(world: &mut CompatWorld) {
    assert_fixture_patches_repo(Fixture::TokioNativeTls);
    world.current_fixture = Some(Fixture::TokioNativeTls);
}

#[given(
    "the downstream-reqwest-native-tls fixture patches the openssl crates from this repository"
)]
fn downstream_reqwest_native_tls_fixture_patches_repo(world: &mut CompatWorld) {
    assert_fixture_patches_repo(Fixture::ReqwestNativeTls);
    world.current_fixture = Some(Fixture::ReqwestNativeTls);
}

#[given(
    "the downstream-postgres-native-tls fixture patches the openssl crates from this repository"
)]
fn downstream_postgres_native_tls_fixture_patches_repo(world: &mut CompatWorld) {
    assert_fixture_patches_repo(Fixture::PostgresNativeTls);
    world.current_fixture = Some(Fixture::PostgresNativeTls);
}

#[given("the example consumer project patches the openssl crates from this repository")]
fn example_consumer_project_patches_repo(world: &mut CompatWorld) {
    assert_fixture_patches_repo(Fixture::ExampleConsumer);
    world.current_fixture = Some(Fixture::ExampleConsumer);
}

#[given("the downstream-openssl-sys fixture patches the openssl crates from this repository")]
fn downstream_openssl_sys_fixture_patches_repo(world: &mut CompatWorld) {
    assert_fixture_patches_repo(Fixture::OpensslSys);
    world.current_fixture = Some(Fixture::OpensslSys);
}

#[given("the downstream-openssl-exact-pin fixture depends on openssl = \"=0.10.76\"")]
fn downstream_exact_pin_fixture_uses_current_version(world: &mut CompatWorld) {
    assert_fixture_patches_repo(Fixture::OpensslExactPin);
    assert_fixture_contains(
        Fixture::OpensslExactPin,
        "openssl = { version = \"=0.10.76\", default-features = false }",
    );
    world.current_fixture = Some(Fixture::OpensslExactPin);
}

#[given("a downstream fixture patches the openssl crates from this repository")]
fn downstream_fixture_patches_repo(_: &mut CompatWorld) {
    assert_fixture_patches_repo(Fixture::BuildMetadata);
}

#[when("the workspace is built with only the OpenSSL backend enabled")]
fn workspace_is_built_with_only_the_openssl_backend(world: &mut CompatWorld) {
    world.build_workspace(Backend::Openssl);
}

#[when("the workspace is built with only the BoringSSL backend enabled")]
fn workspace_is_built_with_only_the_boringssl_backend(world: &mut CompatWorld) {
    world.build_workspace(Backend::Boring);
}

#[when("the workspace is built with default features")]
fn workspace_is_built_with_default_features(world: &mut CompatWorld) {
    world.build_workspace_with_defaults();
}

#[when("a consumer enables both backend features")]
fn consumer_enables_both_backend_features(world: &mut CompatWorld) {
    world.build_with_both_backends();
}

#[when("a consumer builds without backend-openssl or backend-boring")]
fn consumer_builds_without_any_backend_feature(world: &mut CompatWorld) {
    world.build_without_backend();
}

#[when("the fixture is built with backend-openssl")]
fn fixture_is_built_with_backend_openssl(world: &mut CompatWorld) {
    if let Some(fixture) = world.current_fixture {
        world.build_fixture(fixture, Backend::Openssl);
    } else {
        assert!(false, "expected a fixture to be selected before building it");
    }
}

#[when("the fixture is built with backend-boring")]
fn fixture_is_built_with_backend_boring(world: &mut CompatWorld) {
    if let Some(fixture) = world.current_fixture {
        world.build_fixture(fixture, Backend::Boring);
    } else {
        assert!(false, "expected a fixture to be selected before building it");
    }
}

#[when("the fixture is built with either supported backend")]
fn fixture_is_built_with_either_supported_backend(world: &mut CompatWorld) {
    if let Some(fixture) = world.current_fixture {
        world.build_fixture_for_both_backends(fixture);
    } else {
        assert!(false, "expected a fixture to be selected before building it");
    }
}

#[when("the downstream-build-metadata fixture is built with backend-openssl")]
fn downstream_build_metadata_fixture_is_built_with_backend_openssl(world: &mut CompatWorld) {
    assert_fixture_patches_repo(Fixture::BuildMetadata);
    world.build_fixture(Fixture::BuildMetadata, Backend::Openssl);
}

#[when("the downstream-build-metadata fixture is built with backend-boring")]
fn downstream_build_metadata_fixture_is_built_with_backend_boring(world: &mut CompatWorld) {
    assert_fixture_patches_repo(Fixture::BuildMetadata);
    world.build_fixture(Fixture::BuildMetadata, Backend::Boring);
}

#[then("the build succeeds")]
fn build_succeeds(world: &mut CompatWorld) {
    world.assert_all_succeeded();
}

#[then("Cargo reports a configuration error before compilation continues")]
fn cargo_reports_a_configuration_error(world: &mut CompatWorld) {
    world.assert_failure_contains_expected_error();
}

#[then("Cargo resolves the patched openssl package from this repository")]
fn cargo_resolves_the_patched_openssl_package_from_this_repository(world: &mut CompatWorld) {
    world.assert_resolution_uses_repo_patch();
}

#[then("the build script sees DEP_OPENSSL_INCLUDE")]
fn build_script_sees_dep_openssl_include(world: &mut CompatWorld) {
    world.assert_all_succeeded();
}

#[then("the build script sees DEP_OPENSSL_VERSION_NUMBER")]
fn build_script_sees_dep_openssl_version_number(world: &mut CompatWorld) {
    world.assert_all_succeeded();
}

#[then(regex = r"^the build script sees DEP_OPENSSL_BACKEND set to (openssl|boring)$")]
fn build_script_sees_dep_openssl_backend(world: &mut CompatWorld, marker: String) {
    world.assert_all_succeeded();

    if let Some(selected_backend) = world.selected_backend {
        assert_eq!(
            selected_backend.compatibility_marker(),
            marker.as_str(),
            "scenario requested a backend marker that does not match the selected backend",
        );
        assert_eq!(
            selected_backend.compatibility_marker(),
            marker,
            "compatibility marker assertion did not match the selected backend",
        );
        return;
    }

    assert!(false, "expected a selected backend before asserting DEP_OPENSSL_BACKEND",);
}

fn assert_backend_feature_declared(backend: Backend) {
    let feature_declaration = format!("{} = [", backend.cargo_feature());
    assert_manifest_contains(repo_root().join("crates/openssl/Cargo.toml"), &feature_declaration);
    assert_manifest_contains(
        repo_root().join("crates/openssl-sys/Cargo.toml"),
        &feature_declaration,
    );
}

fn assert_default_backend(backend: Backend) {
    let default_declaration = format!("default = [\"{}\"]", backend.cargo_feature());
    assert_manifest_contains(repo_root().join("crates/openssl/Cargo.toml"), &default_declaration);
    assert_manifest_contains(
        repo_root().join("crates/openssl-sys/Cargo.toml"),
        &default_declaration,
    );
}

fn assert_fixture_patches_repo(fixture: Fixture) {
    assert_fixture_contains(fixture, "[patch.crates-io]");

    if fixture == Fixture::ExampleConsumer {
        assert_fixture_contains(fixture, "openssl = { path = \"../../crates/openssl\" }");
        assert_fixture_contains(fixture, "openssl-sys = { path = \"../../crates/openssl-sys\" }");
        return;
    }

    assert_fixture_contains(fixture, "openssl = { path = \"../../../crates/openssl\" }");
    assert_fixture_contains(fixture, "openssl-sys = { path = \"../../../crates/openssl-sys\" }");
}

fn assert_fixture_contains(fixture: Fixture, needle: &str) {
    assert_manifest_contains(fixture.manifest_path(), needle);
}

fn assert_manifest_contains(path: PathBuf, needle: &str) {
    let contents = read_text_file(&path);
    assert!(
        contents.contains(needle),
        "{} did not contain the expected fragment: {}",
        path.display(),
        needle,
    );
}

fn read_text_file(path: &Path) -> String {
    match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(error) => {
            assert!(false, "failed to read {}: {}", path.display(), error);
            String::new()
        }
    }
}

fn repo_root() -> PathBuf {
    let candidate = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    match candidate.canonicalize() {
        Ok(path) => path,
        Err(_) => candidate,
    }
}

fn run_fixture_build(fixture: Fixture, backend: Backend) -> CommandOutcome {
    run_repo_cargo([
        "build".to_owned(),
        "--manifest-path".to_owned(),
        fixture.manifest_path().to_string_lossy().into_owned(),
        "--no-default-features".to_owned(),
        "--features".to_owned(),
        backend.cargo_feature().to_owned(),
    ])
}

fn run_fixture_tree(fixture: Fixture, backend: Backend) -> CommandOutcome {
    run_repo_cargo([
        "tree".to_owned(),
        "--manifest-path".to_owned(),
        fixture.manifest_path().to_string_lossy().into_owned(),
        "--no-default-features".to_owned(),
        "--features".to_owned(),
        backend.cargo_feature().to_owned(),
        "--depth".to_owned(),
        "1".to_owned(),
    ])
}

fn run_repo_cargo<I>(args: I) -> CommandOutcome
where
    I: IntoIterator<Item = String>,
{
    let collected: Vec<OsString> = args.into_iter().map(OsString::from).collect();
    let context = format!(
        "cargo {}",
        collected
            .iter()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect::<Vec<_>>()
            .join(" "),
    );

    match Command::new("cargo")
        .args(&collected)
        .current_dir(repo_root())
        .env("CARGO_TERM_COLOR", "never")
        .output()
    {
        Ok(output) => CommandOutcome {
            context,
            status_code: output.status.code(),
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        },
        Err(error) => CommandOutcome {
            context,
            status_code: None,
            success: false,
            stdout: String::new(),
            stderr: error.to_string(),
        },
    }
}

#[tokio::main]
async fn main() {
    CompatWorld::cucumber()
        .max_concurrent_scenarios(1)
        .run_and_exit(repo_root().join("features"))
        .await;
}
