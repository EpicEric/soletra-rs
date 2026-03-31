{
  system ? builtins.currentSystem,
  sources ? import ./npins,
  pkgs ? import sources.nixpkgs { inherit system; },
}:
let
  python3 = pkgs.python313.withPackages (ps: [
    ps.pydantic
    ps.tqdm
  ]);
in
pkgs.runCommand "soletra-rs-games.json" {
  src = pkgs.fetchFromGitHub {
    repo = "pt-br";
    owner = "fserb";
    rev = "93ba2a6f3b2f85262fba72df09d448c6bb2fa50a";
    hash = "sha256-HrBYUdUxWfLGMVOwhMoamWU92oqcIQ0UC6pwOZdQpKU=";
  };
  nativeBuildInputs = [ python3 ];
} "python3 ${./generate-games.py}"
