{pkgs ? import <nixpkgs> {}}:
with pkgs; mkShell {
  buildInputs = [
      gnuplot
      pre-commit
  ];

}
