class BifrostBin < Formula
    version '0.1.1'
    desc "virtualization via containers"
    homepage "https://github.com/ericdeansanchez/bifrost"
  
    if OS.mac?
        url "https://github.com/ericdeansanchez/bifrost/releases/download/v0.1.0-beta/bifrost-#{version}-x86_64-apple-darwin18.6.0.tar.gz"
        sha256 "d889de9db31b25e72b72dadf6064c0f54c7aad04fc910efd3fed6cc9451fe17a"
    end

    def install
      bin.install "bifrost"
    end

    test do
        system "#{bin}/bifrost", "--version"
    end
end