# frozen_string_literal: true

require "csv"
require "json"
require "optparse"

options = {
  roots: [File.expand_path("..", __dir__)],
  labels: nil,
  min: 16,
  max: 4 * 1024 * 1024,
  steps_per_power: 2,
  min_seconds: 0.18,
  max_iterations: 1_000_000,
  patterns: ["text", "random"],
  out: nil,
}

OptionParser.new do |parser|
  parser.banner = "Usage: ruby --yjit benchmark/payload_size_sweep.rb [options]"

  parser.on("--root PATH", "Benchmark one root. Can be repeated. Default: repo root") do |path|
    options[:roots] = [] if options[:roots] == [File.expand_path("..", __dir__)]
    options[:roots] << File.expand_path(path)
  end

  parser.on("--label LABEL", "Label for a root. Can be repeated in --root order") do |label|
    options[:labels] ||= []
    options[:labels] << label
  end

  parser.on("--min BYTES", Integer, "Minimum payload size. Default: 16") { |v| options[:min] = v }
  parser.on("--max BYTES", Integer, "Maximum payload size. Default: 4MiB") { |v| options[:max] = v }
  parser.on("--steps N", Integer, "Samples per power of two. Default: 2") { |v| options[:steps_per_power] = v }
  parser.on("--seconds N", Float, "Minimum seconds per measurement. Default: 0.18") { |v| options[:min_seconds] = v }
  parser.on("--patterns LIST", "Comma-separated payload patterns. Default: text,random") do |v|
    options[:patterns] = v.split(",")
  end
  parser.on("--out PATH", "Write CSV output to path instead of stdout") { |v| options[:out] = v }
end.parse!

labels = options[:labels] || options[:roots].map { |root| File.basename(root) }
raise "--label count must match --root count" unless labels.length == options[:roots].length

SIZES = begin
  values = []
  exponent = Math.log2(options[:min]).floor
  while (base = 2**exponent) <= options[:max]
    options[:steps_per_power].times do |step|
      multiplier = 2.0**(step.to_f / options[:steps_per_power])
      size = (base * multiplier).round
      values << size if size >= options[:min] && size <= options[:max]
    end
    exponent += 1
  end
  values.uniq.sort.freeze
end

TEXT_CHUNK = "The quick brown fox jumps over the lazy dog. "

def payload(pattern, size)
  case pattern
  when "text"
    (TEXT_CHUNK * ((size / TEXT_CHUNK.bytesize) + 1)).byteslice(0, size)
  when "random"
    Random.new(size).bytes(size)
  when "repeat"
    ("abcdef0123456789" * ((size / 16) + 1)).byteslice(0, size)
  else
    raise "unknown payload pattern: #{pattern}"
  end
end

def timed
  started_at = Process.clock_gettime(Process::CLOCK_MONOTONIC)
  yield
  Process.clock_gettime(Process::CLOCK_MONOTONIC) - started_at
end

def measure(min_seconds:, max_iterations:, &block)
  iterations = 1
  elapsed = 0.0

  while elapsed < min_seconds && iterations < max_iterations
    iterations *= 2
    GC.start
    elapsed = timed do
      iterations.times(&block)
    end
  end

  [iterations, elapsed, iterations / elapsed]
end

rows = []

options[:roots].zip(labels).each do |root, label|
  $LOAD_PATH.unshift(File.join(root, "lib"))
  Object.send(:remove_const, :Lz4Flex) if Object.const_defined?(:Lz4Flex)
  Object.send(:remove_const, :Lz4FlexExt) if Object.const_defined?(:Lz4FlexExt)
  $LOADED_FEATURES.delete_if { |path| path.include?("lz4_flex") }
  require "lz4_flex"

  options[:patterns].each do |pattern|
    SIZES.each do |size|
      input = payload(pattern, size)
      compressed = Lz4Flex.compress(input)
      decompressed = Lz4Flex.decompress(compressed)
      raise "roundtrip failed for #{label}/#{pattern}/#{size}" unless decompressed == input

      compress_iterations, compress_seconds, compress_ips = measure(
        min_seconds: options[:min_seconds],
        max_iterations: options[:max_iterations],
      ) { Lz4Flex.compress(input) }

      decompress_iterations, decompress_seconds, decompress_ips = measure(
        min_seconds: options[:min_seconds],
        max_iterations: options[:max_iterations],
      ) { Lz4Flex.decompress(compressed) }

      rows << {
        label: label,
        pattern: pattern,
        size_bytes: size,
        compressed_bytes: compressed.bytesize,
        compression_ratio: compressed.bytesize.fdiv(size),
        operation: "compress",
        iterations: compress_iterations,
        seconds: compress_seconds,
        ips: compress_ips,
        mib_per_second: (size * compress_ips) / (1024.0 * 1024.0),
      }

      rows << {
        label: label,
        pattern: pattern,
        size_bytes: size,
        compressed_bytes: compressed.bytesize,
        compression_ratio: compressed.bytesize.fdiv(size),
        operation: "decompress",
        iterations: decompress_iterations,
        seconds: decompress_seconds,
        ips: decompress_ips,
        mib_per_second: (size * decompress_ips) / (1024.0 * 1024.0),
      }
    end
  end

  $LOAD_PATH.shift
end

csv = CSV.generate do |out|
  out << rows.first.keys
  rows.each { |row| out << row.values }
end

if options[:out]
  File.write(options[:out], csv)
  warn "wrote #{options[:out]}"
else
  print csv
end
