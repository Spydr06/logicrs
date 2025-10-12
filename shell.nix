{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
    buildInputs = with pkgs.buildPackages; [
        pkgconf
        cairo
        curl
        gtk4
        libadwaita
        librsvg
    ];

    shellHook = ''
        export LD_LIBRARY_PATH="''${LD_LIBRARY_PATH}''${LD_LIBRARY_PATH:+:}${pkgs.libglvnd}/lib"
    '';
}

