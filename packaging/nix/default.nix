# @task T023
# @epic T014
{ lib
, rustPlatform
, fetchFromGitHub
, stdenv
, darwin
, installShellFiles
, testers
, ferrous-forge
}:

rustPlatform.buildRustPackage rec {
  pname = "ferrous-forge";
  version = "1.7.6";

  src = fetchFromGitHub {
    owner = "kryptobaseddev";
    repo = "ferrous-forge";
    rev = "v${version}";
    hash = "sha256-PLACEHOLDER_HASH";
  };

  cargoHash = "sha256-PLACEHOLDER_CARGO_HASH";

  nativeBuildInputs = [ installShellFiles ];

  buildInputs = lib.optionals stdenv.isDarwin [
    darwin.apple_sdk.frameworks.Security
    darwin.apple_sdk.frameworks.SystemConfiguration
  ];

  # Enable all features for maximum functionality
  buildFeatures = [ "update-system" "telemetry" ];

  # Skip tests that may fail in sandboxed environments
  checkFlags = [
    "--skip=test_system_integration"
    "--skip=test_git_hooks"
  ];

  postInstall = ''
    # Install shell completions if available
    # installShellCompletion --cmd ferrous-forge \
    #   --bash <($out/bin/ferrous-forge completions bash) \
    #   --fish <($out/bin/ferrous-forge completions fish) \
    #   --zsh <($out/bin/ferrous-forge completions zsh)
  '';

  passthru.tests.version = testers.testVersion {
    package = ferrous-forge;
    command = "ferrous-forge --version";
  };

  meta = with lib; {
    description = "System-wide Rust development standards enforcer";
    longDescription = ''
      Ferrous Forge is a system-wide Rust development standards enforcer that
      automatically applies professional-grade coding standards to Rust projects.
      It integrates with cargo commands, installs git hooks, and provides a CLI
      for managing Rust versions, editions, safety checks, and more.
    '';
    homepage = "https://ferrous-forge.dev";
    changelog = "https://github.com/kryptobaseddev/ferrous-forge/blob/v${version}/CHANGELOG.md";
    license = with licenses; [ mit asl20 ];
    maintainers = with maintainers; [ ];
    mainProgram = "ferrous-forge";
  };
}
