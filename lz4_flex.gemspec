# frozen_string_literal: true

require_relative "lib/lz4_flex/version"

Gem::Specification.new do |spec|
  spec.name          = "lz4_flex"
  spec.version       = Lz4Flex::VERSION
  spec.authors       = ["Shopify Engineering"]
  spec.email         = ["gems@shopify.com"]
  spec.summary       = "A modern LZ4 compression library for Ruby, leveraging the `lz4_flex` Rust crate."
  spec.description   = <<-DESC.strip.gsub(/\s+/, " ")
    A modern LZ4 compression library for Ruby, leveraging the power of the
    [`lz4_flex`](https://github.com/PSeitz/lz4_flex) Rust crate. This library
    provides a pure Rust implementation of the LZ4 algorithm, ensuring high
    performance and safety.
  DESC

  spec.license = "MIT"

  spec.homepage = "https://github.com/Shopify/lz4-flex-rb"
  spec.required_ruby_version = ">= 3.1"

  spec.metadata["allowed_push_host"] = "https://rubygems.org"

  spec.metadata["homepage_uri"] = spec.homepage
  spec.metadata["source_code_uri"] = spec.homepage
  spec.metadata["changelog_uri"] = "#{spec.homepage}/releases"

  spec.files = Dir["README.md", "LICENSE.md", "lib/**/*.rb", "Cargo.*", "ext/**/*.{rs,rb,toml}"]
  spec.bindir        = "exe"
  spec.executables   = spec.files.grep(%r{\Aexe/}) { |f| File.basename(f) }
  spec.require_paths = ["lib"]
  spec.extensions = ["ext/lz4_flex_ext/extconf.rb"]
  spec.add_dependency("rb_sys")
end
