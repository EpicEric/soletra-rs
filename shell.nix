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
craneLib.devShell {
  packages = [
    (pkgs.python313.withPackages (ps: [
      ps.pydantic
      ps.tqdm
    ]))
  ];

  SOLETRA_RS_LANGUAGE = "pt";
}
