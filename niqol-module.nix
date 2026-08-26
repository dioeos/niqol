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
      };

      Install = {
        WantedBy = [ "default.target" ];
      };
    };
  };
}
