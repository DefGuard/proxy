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

  # project.inlang/settings.json loads these plugins from a CDN at build time,
  # which the sandbox blocks. Each one is fetched at the exact version behind
  # the floating major URL in settings.json, which is then patched to point at it.
  inlangPluginUrl = name: version: "https://cdn.jsdelivr.net/npm/@inlang/${name}@${version}/dist/index.js";
  inlangPlugin = {
    name,
    settingsVersion,
    version,
    hash,
  }: {
    settingsUrl = inlangPluginUrl name settingsVersion;
    file = pkgs.fetchurl {
      url = inlangPluginUrl name version;
      inherit hash;
    };
  };
  inlangPlugins = map inlangPlugin [
    {
      name = "plugin-message-format";
      settingsVersion = "4";
      version = "4.4.5";
      hash = "sha256-siz2DrKLPIw84ftjAGEaBVLxLQ2ZXTfE3SyW462AxkU=";
    }
    {
      name = "plugin-m-function-matcher";
      settingsVersion = "2";
      version = "2.2.17";
      hash = "sha256-hYYvYwV5O1a/2a/lNosJbmP7Kuqzi3eZwFFRe+NJnAs=";
    }
  ];

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
      substituteInPlace project.inlang/settings.json ${lib.concatMapStringsSep " " (plugin:
        "--replace-fail '\"${plugin.settingsUrl}\"' '\"${plugin.file}\"'")
      inlangPlugins}
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

  commonCargoArgs = {
    inherit pname version;
    src = cargoSrc;
    cargoVendorDir = craneLib.vendorCargoDeps {
      src = rootSrc;
    };
    nativeBuildInputs = with pkgs; [
      cmake
      pkg-config
      protobuf
    ];
    buildInputs = with pkgs; [openssl];
    preBuild = ''
      mkdir -p web/dist
      cp -r ${webDist}/. web/dist/
    '';
    VERGEN_GIT_SHA = gitRev;
  };

  cargoArtifacts = craneLib.buildDepsOnly commonCargoArgs;
in
  craneLib.mkCargoDerivation (commonCargoArgs
    // {
      inherit cargoArtifacts;

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
    })
