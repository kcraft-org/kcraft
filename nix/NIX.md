# How to import

To import with flakes use

```nix
{
  inputs = {
    kcraft.url = "github:KCraft/KCraft";
  };

...

  nixpkgs.overlays = [ inputs.kcraft.overlay ]; ## Within configuration.nix
  environment.systemPackages = with pkgs; [ kcraft ]; ##
}
```

To import without flakes use channels:

```sh
nix-channel --add https://github.com/KCraft/KCraft/archive/master.tar.gz kcraft
nix-channel --update kcraft
nix-env -iA kcraft
```

or alternatively you can use

```nix
{
  nixpkgs.overlays = [
    (import (builtins.fetchTarball "https://github.com/KCraft/KCraft/archive/develop.tar.gz")).overlay
  ];

  environment.systemPackages = with pkgs; [ kcraft ];
}
```
