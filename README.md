# soletra-rs

TUI version of the game Soletra/Spelling Bee. Currently playable in:

- Portuguese
- English

## Requisites

- A terminal with a NerdFont.

## Controls

Language selection screen:

- Ctrl+C: Quit
- Left/Right arrow: Change language
- Enter: Select language

Game screen:

- Ctrl+C: Quit
- Ctrl+L: Return to language selection
- Left/Right arrow: Scroll through guesses
- [: Previous game
- ]: Next game
- Character keys: Type guess
- Enter: Submit guess
- Backspace: Erase last character from guess

## Installation

### Cargo

```bash
cargo install --locked soletra-rs
```

### Nix flakes

```nix
{
  inputs = {
    # ...
    soletra-rs.url = "git+https://git.eric.dev.br/EpicEric/soletra-rs.git?ref=main";
  };

  outputs =
    { nixpkgs, soletra-rs, ... }:
    {
      nixosConfigurations.your-hostname = nixpkgs.lib.nixosSystem {
        # ...
        modules = [
          # ...
          ({ pkgs, ... }: {
            environment.systemPackages = [
              soletra-rs.packages.${pkgs.stdenv.hostPlatform.system}.pt
            ];
          })
        ];
      };
      # ...
    }
}
```

### npins

```bash
npins add git --forge forgejo --name soletra-rs --branch main https://git.eric.dev.br/EpicEric/soletra-rs.git
```

```nix
let
  sources = import ./npins;
in
{
  environment.systemPackages = [
    # ...
    (import sources.soletra-rs { })
  ];
}
```
