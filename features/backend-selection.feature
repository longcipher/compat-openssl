Feature: Backend selection
  Scenario: Building with the OpenSSL backend succeeds
    Given the compatibility crates expose a backend-openssl feature
    When the workspace is built with only the OpenSSL backend enabled
    Then the build succeeds

  Scenario: Building with the BoringSSL backend succeeds
    Given the compatibility crates expose a backend-boring feature
    When the workspace is built with only the BoringSSL backend enabled
    Then the build succeeds

  Scenario: Building the workspace with default features succeeds on BoringSSL
    Given the compatibility crates default to backend-boring
    When the workspace is built with default features
    Then the build succeeds

  Scenario: Enabling both backends fails fast
    Given backend-openssl and backend-boring are mutually exclusive
    When a consumer enables both backend features
    Then Cargo reports a configuration error before compilation continues

  Scenario: Enabling no backend after disabling defaults fails fast
    Given backend selection is required once default features are disabled
    When a consumer builds without backend-openssl or backend-boring
    Then Cargo reports a configuration error before compilation continues