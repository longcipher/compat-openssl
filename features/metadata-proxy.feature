Feature: Build metadata proxy
  Scenario: OpenSSL backend forwards build metadata to downstream build scripts
    Given a downstream fixture patches the openssl crates from this repository
    When the downstream-build-metadata fixture is built with backend-openssl
    Then the build script sees DEP_OPENSSL_INCLUDE
    And the build script sees DEP_OPENSSL_VERSION_NUMBER
    And the build script sees DEP_OPENSSL_BACKEND set to openssl

  Scenario: BoringSSL backend emits the compatibility metadata profile
    Given a downstream fixture patches the openssl crates from this repository
    When the downstream-build-metadata fixture is built with backend-boring
    Then the build script sees DEP_OPENSSL_INCLUDE
    And the build script sees DEP_OPENSSL_VERSION_NUMBER
    And the build script sees DEP_OPENSSL_BACKEND set to boring