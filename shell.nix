{ pkgs ? import <nixpkgs> {} }:
 
let 
  stdenv = pkgs.stdenv;
  libPath = with pkgs; stdenv.lib.makeLibraryPath [ xlibs.libX11 xlibs.libXcursor xlibs.libXi xlibs.libXxf86vm ];
in 
stdenv.mkDerivation {
  name = "rust-opengl";
  buildInputs = with pkgs; [ cargo rustc gdb ];
  shellHook = "export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:${libPath}";

  RUST_BACKTRACE=1;
  RUST_SRC_PATH="${pkgs.rustc.src}";
}
