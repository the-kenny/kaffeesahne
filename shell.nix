{ pkgs ? import <nixpkgs> {} }:
 
let 
  stdenv = pkgs.stdenv;
  libPath = with pkgs; stdenv.lib.makeLibraryPath [ xlibs.libX11 xlibs.libXcursor xlibs.libXi xlibs.libXxf86vm ];
in 
stdenv.mkDerivation {
  name = "android-sdk-fhs-shell";
  buildInputs = with pkgs; [ cargo rustc ];
  shellHook = "export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:${libPath}";

  RUST_SRC_PATH="${pkgs.rustc.src}";
}
