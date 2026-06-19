# typed: strict
# frozen_string_literal: true

require_relative "lz4_flex/version"
require_relative "lz4_flex_ext"

module Lz4Flex
  class Error < StandardError; end
  class EncodeError < Error; end
  class DecodeError < Error; end

  ENCODING_TO_ID = {
    Encoding::UTF_8 => Lz4FlexExt::Encoding::UTF8,
    Encoding::BINARY => Lz4FlexExt::Encoding::BINARY,
    Encoding::US_ASCII => Lz4FlexExt::Encoding::US_ASCII,
  }.freeze

  ID_TO_ENCODING = {
    Lz4FlexExt::Encoding::UTF8 => Encoding::UTF_8,
    Lz4FlexExt::Encoding::BINARY => Encoding::BINARY,
    Lz4FlexExt::Encoding::US_ASCII => Encoding::US_ASCII,
  }.freeze

  extend self

  def compress(input)
    encoding_id = ENCODING_TO_ID.fetch(input.encoding) do
      raise Error, "unsupported encoding for string, please use Lz4Flex.compress_block instead"
    end

    Lz4FlexExt.compress(input, encoding_id).tap { |output| output.force_encoding(Encoding::BINARY) }
  rescue Error
    raise
  rescue StandardError => e
    raise EncodeError, e.message
  end

  def decompress(input)
    encoding_id = Lz4FlexExt.get_compressed_encoding(input)
    encoding = ID_TO_ENCODING[encoding_id]
    raise DecodeError, "failed to deserialize header" unless encoding

    expected_size = Lz4FlexExt.get_decompressed_size(input)
    raise DecodeError, "failed to deserialize header" if expected_size == 0xffffffff

    Lz4FlexExt.decompress(input).tap do |output|
      raise DecodeError, "failed to decompress block" if output.empty? && expected_size.positive?

      output.force_encoding(encoding)
    end
  rescue DecodeError
    raise
  rescue StandardError => e
    raise DecodeError, e.message
  end

  class << self
    alias_method :deflate, :compress
    alias_method :inflate, :decompress
  end

  module VarInt
    extend self

    def compress(input)
      Lz4FlexExt::VarInt.compress(input).tap { |output| output.force_encoding(Encoding::BINARY) }
    rescue StandardError => e
      raise EncodeError, e.message
    end

    def decompress(input)
      Lz4FlexExt::VarInt.decompress(input).tap { |output| output.force_encoding(Encoding::BINARY) }
    rescue StandardError => e
      raise DecodeError, e.message
    end
  end
end
