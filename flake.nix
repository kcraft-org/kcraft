{
  description = "A custom launcher for Minecraft that allows you to easily manage multiple installations of Minecraft at once (Fork of MultiMC)";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    flake-compat = { url = "github:edolstra/flake-compat"; flake = false; };
    libnbtplusplus = { url = "github:KCraft/libnbtplusplus"; flake = false; };
    tomlplusplus = { url = "github:marzer/tomlplusplus"; flake = false; };
  };

  outputs = { self, nixpkgs, libnbtplusplus, tomlplusplus, ... }:
    let
      # User-friendly version number.
      version = builtins.substring 0 8 self.lastModifiedDate;

      # Supported systems (qtbase is currently broken for "aarch64-darwin")
      supportedSystems = [ "x86_64-linux" "x86_64-darwin" "aarch64-linux" ];

      # Helper function to generate an attrset '{ x86_64-linux = f "x86_64-linux"; ... }'.
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;

      # Nixpkgs instantiated for supported systems.
      pkgs = forAllSystems (system: nixpkgs.legacyPackages.${system});

      packagesFn = pkgs: rec {
        kcraft-unwrapped = pkgs.qt6Packages.callPackage ./nix/unwrapped.nix { inherit version self libnbtplusplus tomlplusplus; };
        kcraft-qt5-unwrapped = pkgs.libsForQt5.callPackage ./nix/unwrapped.nix { inherit version self libnbtplusplus tomlplusplus; };
        kcraft = pkgs.qt6Packages.callPackage ./nix { inherit version self kcraft-unwrapped; };
        kcraft-qt5 = pkgs.libsForQt5.callPackage ./nix { inherit version self; kcraft-unwrapped = kcraft-qt5-unwrapped; };
        default = kcraft;
      };
    in
    {
      packages = forAllSystems (system:
        let packages = packagesFn pkgs.${system}; in
        packages // { default = packages.kcraft; }
      );

      overlay = final: prev: packagesFn final;
    };
}
