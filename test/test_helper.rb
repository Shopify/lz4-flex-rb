# typed: strict
# frozen_string_literal: true

$LOAD_PATH.unshift(File.expand_path("../lib", __dir__))

require "lz4_flex"
require "minitest/autorun"
require "nicetest"

module Minitest
  class Test
    def with_gc_stress
      return yield if ENV["NO_GC_STRESS"]
      return yield unless rand(10) == 1 # Stress GC 10% of the time

      old = [GC.stress, GC.auto_compact]
      GC.stress = true
      GC.auto_compact = true
      yield
    ensure
      old_stress, old_auto_compact = old
      GC.stress = old_stress
      GC.auto_compact = old_auto_compact
    end
  end
end
