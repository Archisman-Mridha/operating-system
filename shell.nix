/* REFER : https://nixos.wiki/wiki/Development_environment_with_nix-shell */

{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
  nativeBuildInputs = with pkgs.buildPackages; [
		rustup llvm
		nasm

		qemu
		/* TODO : Add riscv64-elf-gdb. */
	];
}
