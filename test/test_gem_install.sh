#!/usr/bin/env bash

set -euxo pipefail

if [ "$1" == "precompiled" ]; then
  gem_platform="$(ruby -e 'puts [Gem::Platform.local.cpu, Gem::Platform.local.os].join("-")')"

  if [ "$gem_platform" == "x64-mingw" ]; then
    gem_platform="$(ruby -e 'puts RUBY_PLATFORM')"
  fi

  gem_pkg="$(ls pkg/*-$gem_platform.gem)"

  if [ -z "$gem_pkg" ]; then
    echo "ERROR: No precompiled gem found for $gem_platform"
    exit 1
  fi
else
  gem_pkg="$(ls $1)"
fi

gem install --verbose "$gem_pkg"

echo "Running tests..."

if ruby -rlz4_flex -e 'exit(Lz4Flex.decompress(Lz4Flex.compress("shop")) == "shop")'; then
  echo "✅ Tests passed!"
else
  echo "❌ Tests failed!"
  exit 1
fi
