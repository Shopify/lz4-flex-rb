# typed: true
# frozen_string_literal: true

require "test_helper"

class BlockTest < Minitest::Test
  parallelize_me!

  def test_that_it_has_a_version_number
    refute_nil(::Lz4Flex::VERSION)
  end

  Dir["**/*"].each do |f|
    file_slug = f.gsub(/[^a-z0-9]+/i, "_").downcase

    define_method("test_roundtrip_compress_#{file_slug}") do
      next if File.directory?(f)

      input = File.binread(f)

      compressed = with_gc_stress { Lz4Flex.compress_block(input) }
      assert_equal(input, with_gc_stress { Lz4Flex.decompress_block(compressed) })
    rescue => e
      e.message << " in file #{f}"
      raise
    end
  end
end
