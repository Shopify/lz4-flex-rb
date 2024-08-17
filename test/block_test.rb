# typed: true
# frozen_string_literal: true

require "test_helper"
require "lz4-ruby"

class BlockTest < Minitest::Test
  parallelize_me!

  def test_that_it_has_a_version_number
    refute_nil(::Lz4Flex::VERSION)
  end

  Dir["**/*"].each do |f|
    file_slug = f.gsub(/[^a-z0-9]+/i, "_").downcase

    define_method("test_roundtrip_compress_#{file_slug}") do
      next if File.directory?(f)

      input = rand(10) == 1 ? File.binread(f) : File.read(f)
      encoding = input.encoding
      compressed = with_gc_stress { Lz4Flex.compress(input) }
      decompressed = with_gc_stress { Lz4Flex.decompress(compressed) }

      assert_equal(Encoding::BINARY, compressed.encoding)
      assert_equal(input, decompressed)
      assert_equal(encoding, decompressed.encoding)
    end
  end

  def test_decompress_fail
    compressed = Lz4Flex.compress("foobarbaz")
    assert_raises(Lz4Flex::DecodeError) { Lz4Flex.decompress(compressed[0..8]) }
  end

  class VarIntTest < Minitest::Test
    def test_lz4_ruby_compatibility_decompress
      input = Random.bytes(1024)
      compressed = LZ4.compress(input)
      assert_equal(input, Lz4Flex::VarInt.decompress(compressed))
    end

    def test_lz4_ruby_compatibility_compress
      input = Random.bytes(1024)
      compressed = Lz4Flex::VarInt.compress(input)
      assert_equal(input, LZ4.decompress(compressed))
    end
  end
end
