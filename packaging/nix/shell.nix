# @task T023
# @epic T014
{ pkgs ? import <nixpkgs> { } }:

pkgs.callPackage ./default.nix { }
