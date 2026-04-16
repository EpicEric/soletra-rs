# soletra-rs

TUI version of the game Soletra/Spelling Bee.

## Nix installation

Nix automates all the setup for you (games generation, compilation).

### With npins

```sh
npins add github EpicEric soletra-rs -b main
```

The following packages are available:

```nix
let
  sources = import ./npins;
in
[
  (import sources.soletra-rs { language = "pt"; })  # Português
  (import sources.soletra-rs { language = "en"; })  # English
]
```

### With Nix Flakes

```nix
{
  inputs = {
    soletra-rs.url = "github:EpicEric/soletra-rs";
  };

  outputs =
    { ... }@inputs:
    {
      # ...
    }
}
```

The following packages are available:

```nix
[
  inputs.soletra-rs.packages.${pkgs.stdenv.hostPlatform.system}.pt  # Português
  inputs.soletra-rs.packages.${pkgs.stdenv.hostPlatform.system}.en  # English
]
```
