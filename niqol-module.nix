{ config, lib, pkgs, ... }:

let
  cfg = config.services.niqol;
in
{
  options.services.niqol = {
    enable = lib.mkEnableOption "Niqol";

    package = lib.mkOption {
      type = lib.types.package;
      description = "Package containing the niqol executables";
    };

    environment = lib.mkOption {
      type = lib.types.attrsOf lib.types.str;
      default = { };
      description = "Environment variables for the Niqol daemon";
    };

  };

  config = lib.mkIf cfg.enable {
    home.packages = [
      cfg.package
    ];

    systemd.user.services.niqol = {
      Unit = {
        Description = "Niqol daemon";
      };

      Service = {
        ExecStart = "${cfg.package}/bin/niqol-daemon";
        Restart = "on-failure";

        Environment = lib.mapAttrsToList (
          name: value: "${name}=${value}"
        ) cfg.environment;
      };

      Install = {
        WantedBy = [ "default.target" ];
      };
    };
  };
}
