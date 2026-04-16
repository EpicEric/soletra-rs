{
  description = "TUI version of the game Soletra/Spelling Bee";

  inputs = { };

  outputs =
    { ... }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];

      eachSystem =
        f:
        (builtins.foldl' (
          acc: system:
          let
            fSystem = f system;
          in
          builtins.foldl' (
            acc': attr:
            acc'
            // {
              ${attr} = (acc'.${attr} or { }) // fSystem.${attr};
            }
          ) acc (builtins.attrNames fSystem)
        ) { } systems);
    in
    eachSystem (system: {
      packages.${system} = {
        pt = import ./. {
          language = "pt";
          inherit system;
        };
        en = import ./. {
          language = "en";
          inherit system;
        };
      };

      devShells.${system}.default = import ./shell.nix { inherit system; };
    });
}
