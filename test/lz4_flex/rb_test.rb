# typed: true
# frozen_string_literal: true

require "test_helper"

module Lz4Flex
  class RbTest < Minitest::Test
    def test_that_it_has_a_version_number
      refute_nil(::Lz4Flex::Rb::VERSION)
    end
  end
end
