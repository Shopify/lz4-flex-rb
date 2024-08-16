# frozen_string_literal: true

require_relative "lib/lz4_flex/rb/version"

Gem::Specification.new do |spec|
  spec.name          = "lz4_flex-rb"
  spec.version       = Lz4Flex::Rb::VERSION
  spec.authors       = ["Shopify Engineering"]
  spec.email         = ["gems@shopify.com"]

  spec.summary       = "Short summary of what your gem does"
  spec.description   = spec.summary
  spec.homepage      = "https://github.com/Shopify/lz4-flex-rb"
  spec.required_ruby_version = ">= 3.3"

  spec.metadata["allowed_push_host"] = "https://pkgs.shopify.io"

  spec.metadata["homepage_uri"] = spec.homepage
  spec.metadata["source_code_uri"] = spec.homepage
  spec.metadata["changelog_uri"] = "#{spec.homepage}/releases"

  # Specify which files should be added to the gem when it is released.
  # The `git ls-files -z` loads the files in the RubyGem that have been added into git.
  spec.files = Dir.chdir(File.expand_path(__dir__)) do
    %x(git ls-files -z).split("\x0").reject { |f| f.match(%r{\A(?:test|spec|features)/}) }
  end
  spec.bindir        = "exe"
  spec.executables   = spec.files.grep(%r{\Aexe/}) { |f| File.basename(f) }
  spec.require_paths = ["lib"]

  spec.add_dependency("sorbet-runtime")
end
