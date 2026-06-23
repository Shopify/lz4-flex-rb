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

  ID_TO_ENCODING = [
    Encoding::UTF_8,
    Encoding::BINARY,
    Encoding::US_ASCII,
  ].freeze

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
    metadata = Lz4FlexExt.get_decompression_metadata(input)
    raise DecodeError, "failed to deserialize header" if metadata == 0xffffffffffffffff

    encoding = ID_TO_ENCODING[metadata >> 56]
    raise DecodeError, "failed to deserialize header" unless encoding

    data_offset = (metadata >> 32) & 0x00ffffff
    expected_size = metadata & 0xffffffff

    output = if input.bytesize >= expected_size
      input.byteslice(0, expected_size)
    else
      "\0".b * expected_size
    end
    written = Lz4FlexExt.decompress_payload_into(input, data_offset, expected_size, output)
    raise DecodeError, "failed to decompress block" if written == 0xffffffff
    raise DecodeError, "unexpected decompressed size" if written != expected_size

    output.force_encoding(encoding)
    output
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
