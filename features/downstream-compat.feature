Feature: Downstream compatibility
  Scenario: A native-tls consumer builds with the OpenSSL backend
    Given the downstream-native-tls fixture patches the openssl crates from this repository
    When the fixture is built with backend-openssl
    Then the build succeeds

  Scenario: A native-tls consumer builds with the BoringSSL backend
    Given the downstream-native-tls fixture patches the openssl crates from this repository
    When the fixture is built with backend-boring
    Then the build succeeds

  Scenario: A direct openssl-sys consumer builds with either supported backend
    Given the downstream-openssl-sys fixture patches the openssl crates from this repository
    When the fixture is built with either supported backend
    Then the build succeeds

  Scenario: A tokio-native-tls consumer builds with either supported backend
    Given the downstream-tokio-native-tls fixture patches the openssl crates from this repository
    When the fixture is built with either supported backend
    Then the build succeeds

  Scenario: A reqwest native-tls consumer builds with either supported backend
    Given the downstream-reqwest-native-tls fixture patches the openssl crates from this repository
    When the fixture is built with either supported backend
    Then the build succeeds

  Scenario: A postgres native-tls consumer builds with either supported backend
    Given the downstream-postgres-native-tls fixture patches the openssl crates from this repository
    When the fixture is built with either supported backend
    Then the build succeeds

  Scenario: An exact pinned OpenSSL consumer resolves against the current compatibility line
    Given the downstream-openssl-exact-pin fixture depends on openssl = "=0.10.76"
    When the fixture is built with either supported backend
    Then Cargo resolves the patched openssl package from this repository

  Scenario: The example consumer project builds with either supported backend
    Given the example consumer project patches the openssl crates from this repository
    When the fixture is built with either supported backend
    Then Cargo resolves the patched openssl package from this repository