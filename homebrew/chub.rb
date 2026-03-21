class Chub < Formula
  desc "Fast curated docs for AI coding agents — team-first, git-tracked, built in Rust"
  homepage "https://chub.nrl.ai"
  version "0.1.3"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/vietanhdev/chub/releases/download/v#{version}/chub-darwin-arm64"
      sha256 "PLACEHOLDER_DARWIN_ARM64"
    else
      url "https://github.com/vietanhdev/chub/releases/download/v#{version}/chub-darwin-x64"
      sha256 "PLACEHOLDER_DARWIN_X64"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/vietanhdev/chub/releases/download/v#{version}/chub-linux-arm64"
      sha256 "PLACEHOLDER_LINUX_ARM64"
    else
      url "https://github.com/vietanhdev/chub/releases/download/v#{version}/chub-linux-x64"
      sha256 "PLACEHOLDER_LINUX_X64"
    end
  end

  def install
    binary = Dir["chub-*"].first || "chub"
    bin.install binary => "chub"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/chub --version")
  end
end
