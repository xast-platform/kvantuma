{
  description = "kvantuma dev shell: embree + openimagedenoise for xastge-scene";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  # The `embree` Rust crate hardcodes `-lembree3` (Embree 3.x ABI), which current
  # nixpkgs no longer ships (embree is now 4.x, libembree4 only). Pin an older
  # revision for embree + the matching classic-ABI tbb (libtbb.so.2) it needs.
  inputs.nixpkgs-embree3.url = "github:NixOS/nixpkgs/nixos-23.05";

  outputs = { self, nixpkgs, nixpkgs-embree3 }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
      pkgsEmbree3 = nixpkgs-embree3.legacyPackages.${system};
    in {
      devShells.${system}.default = pkgs.mkShell {
        nativeBuildInputs = [ pkgs.pkg-config ];
        buildInputs = [
          pkgsEmbree3.embree
          pkgsEmbree3.tbb
          pkgs.openimagedenoise
        ];

        # embree's crate build.rs looks for EMBREE_DIR directly rather than pkg-config.
        EMBREE_DIR = "${pkgsEmbree3.embree}";
        # nixpkgs' openimagedenoise ships no .pc file (CMake config only), so oidn's
        # build.rs pkg-config probe can't work here either — use its OIDN_DIR fallback.
        OIDN_DIR = "${pkgs.openimagedenoise}";
        LD_LIBRARY_PATH = "${pkgs.openimagedenoise}/lib:${pkgsEmbree3.embree}/lib:${pkgsEmbree3.tbb}/lib";
      };
    };
}
