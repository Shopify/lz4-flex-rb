# typed: true
# frozen_string_literal: true

require "test_helper"
require "lz4-ruby"

class BlockTest < Minitest::Test
  parallelize_me!

  BIN_TEST_NAMES = Dir["test/data/*"].map { |f| File.basename(f).split(".", 2).first }

  def test_that_it_has_a_version_number
    refute_nil(::Lz4Flex::VERSION)
  end

  Dir["**/*"].each do |f|
    file_slug = f.gsub(/[^a-z0-9]+/i, "_").downcase

    define_method("test_roundtrip_compress_#{file_slug}") do
      next if File.directory?(f)

      input = rand(10) == 1 ? read_file(f) : read_file(f, false)
      encoding = input.encoding
      compressed = with_gc_stress { Lz4Flex.compress(input) }
      decompressed = with_gc_stress { Lz4Flex.decompress(compressed) }

      assert_equal(Encoding::BINARY, compressed.encoding)
      assert_equal(input, decompressed)
      assert_equal(encoding, decompressed.encoding)
    end
  end

  BIN_TEST_NAMES.each do |basename|
    define_method("test_binary_compat_#{basename}") do
      input = read_file("test/data/#{basename}.input")
      expected = read_file("test/data/#{basename}.expected")

      assert_equal(expected, Lz4Flex.decompress(input))
    end
  end

  def test_decompress_fail
    compressed = Lz4Flex.compress("foobarbaz")
    assert_raises(Lz4Flex::DecodeError) { Lz4Flex.decompress(compressed[0..8]) }
  end

  def read_file(file, bin = true, attempts = 3)
    if bin
      File.binread(file)
    else
      File.read(file)
    end
  rescue Errno::EFAULT
    # If we hit an EFAULT, retry a few times since it can be a transient error on ruby 3.2
    attempts -= 1

    if attempts > 0
      sleep(0.1)
      retry
    else
      raise
    end
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
