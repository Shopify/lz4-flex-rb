# frozen_string_literal: true

system "bundle exec rake compile:release"

require "bundler/setup"
require "benchmark"
require "lz4_flex"
require "lz4-ruby"
require "benchmark/ips"
require "net/http"
require "tempfile"
require "json"
require "csv"

ADAPTERS = {
  "Lz4Flex" => Lz4Flex,
  "LZ4" => LZ4,
}.freeze

def download_and_unzip(url)
  puts "Downloading #{url}"
  bz2_tempfile = Tempfile.new(["webster", ".bz2"])
  data = Net::HTTP.get(URI(url))
  bz2_tempfile.write(data)
  bz2_tempfile.close
  puts "Unzipping #{bz2_tempfile.path}"
  system("bzip2 -dk #{bz2_tempfile.path}")
  content = File.read(bz2_tempfile.path.gsub(/\.bz2$/, ""))
  puts "Read #{content.bytesize / 1024 / 1024} MiB of data"

  content
ensure
  bz2_tempfile&.unlink
end

def sample_content(content, sizes)
  results = {}
  content_size = content.bytesize

  sizes.each do |size_bytes|
    raise "Requested sample size exceeds file size." if size_bytes > content_size

    start_index = rand(0..content_size - size_bytes)
    results[size_bytes] = content[start_index, size_bytes]
  end

  results
end

def benchmark(tag, data_hash, iterations, time, warmup, num_threads) # rubocop:disable Metrics/ParameterLists
  results = {}
  benchmark_data = Benchmark.ips do |x|
    x.config(time: time, warmup: warmup)

    ADAPTERS.each do |name, adapter|
      data = data_hash.fetch(name)

      x.report("#{name}.#{tag}") do
        if num_threads > 1
          threads = Array.new(num_threads) do
            Thread.new do
              thread_data = data.dup
              iterations.times { yield(adapter, thread_data) }
            end
          end
          threads.each(&:join)
        else
          iterations.times { yield(adapter, data) }
        end
      end
    end

    x.compare!
  end.data

  benchmark_data.each do |entry|
    results[entry[:name]] = entry[:central_tendency]
  end

  results
end

def format_csv_row(size_bytes, num_threads, results_compress, results_decompress)
  lz4_compress = results_compress.fetch("LZ4.compress")
  lz4flex_compress = results_compress.fetch("Lz4Flex.compress")
  speedup_compress = lz4flex_compress / lz4_compress

  lz4_decompress = results_decompress.fetch("LZ4.decompress")
  lz4flex_decompress = results_decompress.fetch("Lz4Flex.decompress")
  speedup_decompress = lz4flex_decompress / lz4_decompress

  [
    RUBY_PLATFORM,
    size_bytes,
    num_threads,
    lz4_compress,
    lz4flex_compress,
    speedup_compress,
    lz4_decompress,
    lz4flex_decompress,
    speedup_decompress,
  ]
end

def run_benchmarks(url:, sample_sizes:, iterations:, num_threads:, time:, warmup:, save: true)
  content = download_and_unzip(url)
  data_samples = sample_content(content, sample_sizes)
  file_name = "tmp/benchmarks_#{Time.now.to_i}.csv"
  puts "Writing results to #{file_name}"

  CSV.open(file_name, "wb") do |csv|
    csv << [
      "platform",
      "size_bytes",
      "num_threads",
      "lz4_compress",
      "lz4flex_compress",
      "speedup_compress",
      "lz4_decompress",
      "lz4flex_decompress",
      "speedup_decompress",
    ]

    data_samples.each do |size_bytes, data|
      puts "Benchmarking for size: #{size_bytes / 1024} KiB"

      num_threads = num_threads.respond_to?(:call) ? num_threads.call : num_threads
      compressed_data_lz4_flex = Lz4Flex.compress(data)
      compressed_data_lz4 = LZ4.compress(data)

      results_compress = benchmark(
        "compress",
        { "LZ4" => data, "Lz4Flex" => data },
        iterations,
        time,
        warmup,
        num_threads,
      ) { |adapter, data| adapter.compress(data) }

      results_decompress = benchmark(
        "decompress",
        { "LZ4" => compressed_data_lz4, "Lz4Flex" => compressed_data_lz4_flex },
        iterations,
        time,
        warmup,
        num_threads,
      ) { |adapter, data| adapter.decompress(data) }

      csv << format_csv_row(
        size_bytes,
        num_threads,
        results_compress,
        results_decompress,
      )
    end
  rescue SignalException
    puts "Interrupted! Cleaning up..."
  end

  if save
    puts "Done! Results written to #{file_name}"
  else
    puts "Warmup done! Cleaning up..."
    FileUtils.rm(file_name)
    puts "=========================================================="
    sleep(1)
  end
end

# Warmup
run_benchmarks(
  url: "https://sun.aei.polsl.pl/~sdeor/corpus/webster.bz2",
  sample_sizes: [1024, 1024 * 1024, 10 * 1024 * 1024],
  iterations: 10,
  num_threads: -> { rand(1..10) },
  time: 1,
  warmup: 1,
  save: false,
)

run_benchmarks(
  url: "https://sun.aei.polsl.pl/~sdeor/corpus/webster.bz2",
  sample_sizes: 10000.times.map { rand(1..(20 * 1024 * 1024)) },
  iterations: 100,
  num_threads: -> { rand(1..10) },
  time: 3,
  warmup: 0,
)
