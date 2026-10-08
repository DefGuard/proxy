{
  pkgs,
  lib,
  craneLib,
  fetchPnpmDeps,
  pnpmConfigHook,
  gitRev ? "unknown",
}: let
  pname = "defguard-proxy";
  version = (builtins.fromTOML (builtins.readFile ../Cargo.toml)).package.version;
  rootSrc = craneLib.path ../.;
  webSrc = "${rootSrc}/web";

  # Keep the proto sources in the Cargo derivations; the web tree is built
  # separately and copied into web/dist before rust-embed runs.
  cargoSrc = lib.cleanSourceWith {
    src = rootSrc;
    filter = path: type:
      (craneLib.filterCargoSources path type)
      || lib.hasInfix "/proto/" path;
  };

  messageFormatPlugin = pkgs.fetchurl {
    url = "https://cdn.jsdelivr.net/npm/@inlang/plugin-message-format@4/dist/index.js";
    hash = "sha256-siz2DrKLPIw84ftjAGEaBVLxLQ2ZXTfE3SyW462AxkU=";
  };
  mFunctionMatcherPlugin = pkgs.fetchurl {
    url = "https://cdn.jsdelivr.net/npm/@inlang/plugin-m-function-matcher@2/dist/index.js";
    hash = "sha256-hYYvYwV5O1a/2a/lNosJbmP7Kuqzi3eZwFFRe+NJnAs=";
  };

  webPnpmDeps = fetchPnpmDeps {
    pname = "${pname}-web";
    inherit version;
    src = webSrc;
    fetcherVersion = 4;
    hash = "sha256-G3ob1OJDRcHQkFz/bc9K3xmYRNGIwNd//vBtgxF8GY8=";
  };

  webDist = pkgs.stdenv.mkDerivation {
    pname = "${pname}-web";
    inherit version;
    src = webSrc;

    nativeBuildInputs = with pkgs; [
      nodejs_26
      pnpm_11
      pnpmConfigHook
    ];
    pnpmDeps = webPnpmDeps;

    postPatch = ''
      substituteInPlace project.inlang/settings.json \
        --replace-fail '"https://cdn.jsdelivr.net/npm/@inlang/plugin-message-format@4/dist/index.js"' '"${messageFormatPlugin}"' \
        --replace-fail '"https://cdn.jsdelivr.net/npm/@inlang/plugin-m-function-matcher@2/dist/index.js"' '"${mFunctionMatcherPlugin}"'
    '';

    buildPhase = ''
      runHook preBuild
      pnpm build
      runHook postBuild
    '';

    installPhase = ''
      mkdir -p "$out"
      cp -r dist/. "$out/"
    '';
  };

  cargoNativeBuildInputs = with pkgs; [
    cmake
    pkg-config
    protobuf
  ];
  cargoBuildInputs = with pkgs; [
    openssl
    systemd # provides libudev.pc required by hidapi
  ];

  cargoVendorDir = craneLib.vendorCargoDeps {
    src = rootSrc;
  };

  cargoEnv = {
    SQLX_OFFLINE = "true";
    VERGEN_GIT_SHA = gitRev;
  };

  cargoArtifacts = craneLib.buildDepsOnly ({
      inherit pname version cargoSrc cargoVendorDir;
      src = cargoSrc;
      nativeBuildInputs = cargoNativeBuildInputs;
      buildInputs = cargoBuildInputs;
      preBuild = ''
        mkdir -p web/dist
        cp -r ${webDist}/. web/dist/
      '';
    }
    // cargoEnv);
in
  craneLib.mkCargoDerivation ({
      inherit pname version cargoArtifacts cargoVendorDir;
      src = cargoSrc;
      nativeBuildInputs = cargoNativeBuildInputs;
      buildInputs = cargoBuildInputs;

      preBuild = ''
        mkdir -p web/dist
        cp -r ${webDist}/. web/dist/
      '';

      buildPhaseCargoCommand = "cargo build --release --locked";

      installPhase = ''
        install -Dm755 target/release/defguard-proxy "$out/bin/defguard-proxy"
      '';

      passthru = {inherit webPnpmDeps;};

      meta = with lib; {
        description = "Defguard Edge service";
        homepage = "https://github.com/DefGuard/proxy";
        license = licenses.asl20;
        mainProgram = "defguard-proxy";
        platforms = platforms.linux;
      };
    }
    // cargoEnv)
