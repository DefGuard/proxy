{mkCraneLib}: {
  config,
  lib,
  pkgs,
  ...
}: let
  inherit (lib) mkEnableOption mkIf mkOption optional optionalAttrs types;

  cfg = config.services.defguard-edge;
  package = pkgs.callPackage ./package.nix {
    craneLib = mkCraneLib pkgs;
  };
  stateDir = "/var/lib/defguard-edge";

  reservedConfigKeys = [
    "http_port"
    "grpc_port"
    "https_port"
    "http_bind_address"
    "grpc_bind_address"
    "log_level"
    "rate_limit_per_second"
    "rate_limit_burst"
    "cert_dir"
    "acme_staging"
    "adoption_timeout"
  ];
  invalidExtraConfig = lib.filter
    (key: builtins.elem key reservedConfigKeys)
    (builtins.attrNames cfg.extraConfig);
  privilegedPorts = lib.filter (port: port < 1024) [
    cfg.httpPort
    cfg.grpcPort
    cfg.httpsPort
  ];

  settings = {
    http_port = cfg.httpPort;
    grpc_port = cfg.grpcPort;
    https_port = cfg.httpsPort;
    log_level = cfg.logLevel;
    rate_limit_per_second = cfg.rateLimitPerSecond;
    rate_limit_burst = cfg.rateLimitBurst;
    cert_dir = "${stateDir}/certs";
    acme_staging = cfg.acmeStaging;
    adoption_timeout = cfg.adoptionTimeout;
  }
  // optionalAttrs (cfg.httpBindAddress != null) {
    http_bind_address = cfg.httpBindAddress;
  }
  // optionalAttrs (cfg.grpcBindAddress != null) {
    grpc_bind_address = cfg.grpcBindAddress;
  }
  // cfg.extraConfig;

  configFile = (pkgs.formats.toml {}).generate "defguard-edge.toml" settings;
  portCapability = optional cfg.allowPrivilegedPorts "CAP_NET_BIND_SERVICE";
in {
  options.services.defguard-edge = {
    enable = mkEnableOption "Defguard Edge service";

    package = mkOption {
      type = types.package;
      default = package;
      description = "Package that provides the defguard-proxy binary.";
    };

    httpPort = mkOption {
      type = types.port;
      default = 8080;
      description = "HTTP port for the Edge API and embedded web UI.";
    };

    grpcPort = mkOption {
      type = types.port;
      default = 50051;
      description = "gRPC port on which Edge accepts Core management connections; adoption is performed separately.";
    };

    httpsPort = mkOption {
      type = types.port;
      default = 8443;
      description = "HTTPS port used when Core sends TLS certificates to the Edge.";
    };

    httpBindAddress = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "Optional IP address on which the HTTP listener binds.";
    };

    grpcBindAddress = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "Optional IP address on which the Edge gRPC listener binds.";
    };

    logLevel = mkOption {
      type = types.enum ["off" "error" "warn" "info" "debug" "trace"];
      default = "info";
      description = "Edge log level.";
    };

    rateLimitPerSecond = mkOption {
      type = types.ints.unsigned;
      default = 0;
      description = "Interval in seconds between replenished requests; zero disables the limiter.";
    };

    rateLimitBurst = mkOption {
      type = types.ints.between 0 4294967295;
      default = 0;
      description = "Maximum burst size for the Edge rate limiter.";
    };

    adoptionTimeout = mkOption {
      type = types.ints.positive;
      default = 10;
      description = "Minutes for which the plaintext adoption listener accepts a new adoption.";
    };

    acmeStaging = mkOption {
      type = types.bool;
      default = false;
      description = "Use the Let's Encrypt staging endpoint for Edge-side ACME.";
    };

    allowPrivilegedPorts = mkOption {
      type = types.bool;
      default = false;
      description = ''
        Grant CAP_NET_BIND_SERVICE for configured ports below 1024 or the
        Edge's hard-coded ACME HTTP-01 listener on port 80. The default uses
        high ports and no capability; ACME HTTP-01 requires this opt-in.
      '';
    };

    extraConfig = mkOption {
      type = types.attrsOf (types.oneOf [
        types.bool
        types.int
        types.str
        (types.listOf types.str)
      ]);
      default = {};
      description = ''
        Additional TOML keys for Edge settings added by a future upstream
        release. Existing typed keys are rejected rather than overridden.
      '';
    };
  };

  config = mkIf cfg.enable {
    assertions = [
      {
        assertion = invalidExtraConfig == [];
        message = "services.defguard-edge.extraConfig cannot override typed keys: ${toString invalidExtraConfig}";
      }
      {
        assertion = cfg.allowPrivilegedPorts || privilegedPorts == [];
        message = "services.defguard-edge needs allowPrivilegedPorts = true for ports below 1024: ${toString privilegedPorts}";
      }
    ];

    environment.etc."defguard-edge/config.toml".source = configFile;

    systemd.services.defguard-edge = {
      description = "Defguard Edge service";
      documentation = ["https://docs.defguard.net/"];
      wantedBy = ["multi-user.target"];
      wants = ["network-online.target"];
      after = ["network-online.target"];

      serviceConfig = {
        ExecStart = "${cfg.package}/bin/defguard-proxy --config /etc/defguard-edge/config.toml";
        DynamicUser = true;
        StateDirectory = "defguard-edge";
        WorkingDirectory = "/var/empty";

        AmbientCapabilities = portCapability;
        CapabilityBoundingSet = portCapability;
        NoNewPrivileges = true;
        PrivateDevices = true;
        PrivateTmp = true;
        ProtectControlGroups = true;
        ProtectHome = true;
        ProtectKernelLogs = true;
        ProtectKernelModules = true;
        ProtectKernelTunables = true;
        ProtectSystem = "strict";
        ReadWritePaths = [stateDir];
        RestrictAddressFamilies = ["AF_INET" "AF_INET6" "AF_UNIX"];
        RestrictRealtime = true;
        RestrictSUIDSGID = true;
        LockPersonality = true;
        UMask = "0077";

        KillMode = "process";
        KillSignal = "SIGINT";
        LimitNOFILE = 65536;
        Restart = "on-failure";
        RestartSec = 2;
        TasksMax = "infinity";
      };
    };
  };
}
