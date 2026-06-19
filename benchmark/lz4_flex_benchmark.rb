# frozen_string_literal: true

require "json"

$LOAD_PATH.unshift(File.expand_path("../lib", __dir__))
require "lz4_flex"
require "lz4_flex/version"

ITER_SCALE = (ENV["ITER_SCALE"] || "1").to_f

PAYLOADS = {
  small_text: "hello world " * 8,
  medium_text: ("The quick brown fox jumps over the lazy dog. " * 2_048),
  large_text: ("abcdef0123456789" * 262_144),
  medium_random: Random.new(1_234).bytes(128 * 1024),
}.freeze

BASE_ITERS = {
  small_text: 80_000,
  medium_text: 4_000,
  large_text: 160,
  medium_random: 1_000,
}.freeze

def timed(iterations, &block)
  GC.start
  started_at = Process.clock_gettime(Process::CLOCK_MONOTONIC)
  iterations.times(&block)
  Process.clock_gettime(Process::CLOCK_MONOTONIC) - started_at
end

def ips(iterations, elapsed)
  iterations / elapsed
end

results = {
  root: File.expand_path("..", __dir__),
  ruby: RUBY_DESCRIPTION,
  lz4_flex_version: Lz4Flex::VERSION,
  cases: {},
}

PAYLOADS.each do |name, input|
  iterations = [(BASE_ITERS.fetch(name) * ITER_SCALE).to_i, 1].max
  compressed = Lz4Flex.compress(input)
  raise "roundtrip failed for #{name}" unless Lz4Flex.decompress(compressed) == input

  compress_elapsed = timed(iterations) { Lz4Flex.compress(input) }
  decompress_elapsed = timed(iterations) { Lz4Flex.decompress(compressed) }

  thread_iterations = [iterations / 8, 1].max
  threaded_elapsed = timed(1) do
    4.times.map do
      Thread.new do
        thread_input = compressed.dup
        thread_iterations.times { Lz4Flex.decompress(thread_input) }
      end
    end.each(&:join)
  end

  results[:cases][name] = {
    input_bytes: input.bytesize,
    compressed_bytes: compressed.bytesize,
    iterations: iterations,
    compress_ips: ips(iterations, compress_elapsed),
    decompress_ips: ips(iterations, decompress_elapsed),
    threaded_decompress_ops: thread_iterations * 4,
    threaded_decompress_seconds: threaded_elapsed,
    threaded_decompress_ips: ips(thread_iterations * 4, threaded_elapsed),
  }
end

puts JSON.pretty_generate(results)
