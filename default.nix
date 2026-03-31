{
  system ? builtins.currentSystem,
  sources ? import ./npins,
  pkgs ? import sources.nixpkgs {
    inherit system;
    overlays = [ (import sources.rust-overlay) ];
  },
  craneLib ? (import sources.crane { inherit pkgs; }).overrideToolchain (
    p: p.rust-bin.stable.latest.default
  ),
}:
let
  inherit (pkgs) lib;

  src =
    let
      soletra-rs-games = import ./generate-games.nix { inherit system sources pkgs; };
    in
    pkgs.stdenvNoCC.mkDerivation {
      name = "soletra-rs-source";
      src = lib.fileset.toSource {
        root = ./.;
        fileset = lib.fileset.unions [
          (craneLib.fileset.commonCargoSources ./.)
        ];
      };
      installPhase = ''
        runHook preInstall

        cp -r $src/ $out
        chmod -R u+w $out
        ln -s ${soletra-rs-games} $out/src/games.json

        runHook postInstall
      '';
    };

  commonArgs = {
    inherit src;
    strictDeps = true;
  };

  cargoArtifacts = craneLib.buildDepsOnly commonArgs;
in
craneLib.buildPackage (
  commonArgs
  // {
    inherit cargoArtifacts;
    doCheck = false;
    meta.mainProgram = "soletra-rs";
  }
)
