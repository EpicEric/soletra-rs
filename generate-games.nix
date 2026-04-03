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
  lista-de-palavras = pkgs.fetchurl {
    url = "https://web.archive.org/web/20260403013752/http://200.17.137.109:8081/novobsi/Members/cicerog/disciplinas/introducao-a-programacao/arquivos-2015-2/algoritmos/Lista-de-Palavras.txt";
    hash = "sha256-Xn3mYiGbJEvmboyyxQb3d27GvnXKi1cCjOM3mkQGaEo=";
  };
  pt-br = pkgs.fetchFromGitHub {
    repo = "pt-br";
    owner = "fserb";
    rev = "93ba2a6f3b2f85262fba72df09d448c6bb2fa50a";
    hash = "sha256-HrBYUdUxWfLGMVOwhMoamWU92oqcIQ0UC6pwOZdQpKU=";
  };
  nativeBuildInputs = [ python3 ];
  LOCALE_ARCHIVE = "${pkgs.glibcLocales}/lib/locale/locale-archive";
} "python3 ${./generate-games.py}"
