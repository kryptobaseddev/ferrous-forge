# typed: false
# frozen_string_literal: true

# Ferrous Forge - System-wide Rust development standards enforcer
# @task T023
# @epic T014
class FerrousForge < Formula
  desc "System-wide Rust development standards enforcer"
  homepage "https://ferrous-forge.dev"
  version "1.7.6"
  license "MIT OR Apache-2.0"

  stable do
    if OS.mac? && Hardware::CPU.arm?
      url "https://github.com/kryptobaseddev/ferrous-forge/releases/download/v1.7.6/ferrous-forge-macos-aarch64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_MACOS_AARCH64"
    elsif OS.mac?
      url "https://github.com/kryptobaseddev/ferrous-forge/releases/download/v1.7.6/ferrous-forge-macos-x86_64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_MACOS_X86_64"
    elsif OS.linux?
      url "https://github.com/kryptobaseddev/ferrous-forge/releases/download/v1.7.6/ferrous-forge-linux-x86_64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_LINUX_X86_64"
    end
  end

  head do
    url "https://github.com/kryptobaseddev/ferrous-forge.git", branch: "main"
    depends_on "rust" => :build
  end

  depends_on "rust" => :recommended

  conflicts_with "ferrous-forge-bin", because: "both install ferrous-forge binary"

  def install
    if build.head?
      system "cargo", "install", *std_cargo_args
    else
      bin.install "ferrous-forge"
    end
  end

  def post_install
    ohai "Ferrous Forge installed successfully!"
    ohai "Run 'ferrous-forge init' to set up system-wide standards"
    ohai "Run 'ferrous-forge --help' to see available commands"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/ferrous-forge --version")
    assert_match "System-wide Rust development standards enforcer", shell_output("#{bin}/ferrous-forge --help")
  end
end
