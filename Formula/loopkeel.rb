class Loopkeel < Formula
  desc "Resilient CLI orchestrator for agentic work loops"
  homepage "https://github.com/hebertcisco/loopkeel"
  url "https://github.com/hebertcisco/loopkeel/archive/refs/tags/v0.1.0.tar.gz"
  version "0.1.0"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args(path: ".")
  end

  test do
    assert_match "Usage", shell_output("#{bin}/loop --help")
  end
end
