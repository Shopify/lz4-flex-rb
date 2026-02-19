# typed: strict
# frozen_string_literal: true

# Tries to require the precompiled extension for the given Ruby version first
begin
  RUBY_VERSION =~ /(\d+\.\d+)/
  require "lz4_flex/#{Regexp.last_match(1)}/lz4_flex_ext"
rescue LoadError
  require "lz4_flex/lz4_flex_ext"
end
