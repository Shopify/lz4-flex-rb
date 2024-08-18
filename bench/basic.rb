# frozen_string_literal: true

require "bundler/setup"
require "benchmark"
require "lz4_flex"
require "lz4-ruby"
require "benchmark/ips"
require "open-uri"
require "tempfile"
require "json"
require "csv"

system "bundle exec rake compile:release"

ADAPTERS = {
  "Lz4Flex" => Lz4Flex,
  "LZ4" => LZ4,
}

def download_and_unzip(url)
  bz2_tempfile = Tempfile.new(["webster", ".bz2"])

  puts "Downloading..."
  URI.open(url) do |data|
    bz2_tempfile.write(data.read)
  end
  bz2_tempfile.close

  puts "Unzipping #{bz2_tempfile.path}"
  system("bzip2 -dk #{bz2_tempfile.path}")

  content = File.read(bz2_tempfile.path.gsub(/\.bz2$/, ""))
  puts "Read #{content.bytesize / 1024 / 1024} MiB of data"

  bz2_tempfile.unlink

  content
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

def benchmark_single_threaded(data, _iterations, time, warmup)
  results = {}
  benchmark_data = Benchmark.ips do |x|
    x.config(time: time, warmup: warmup)

    ADAPTERS.each do |name, adapter|
      x.report("#{name} single-threaded") do
        compressed = adapter.compress(data)
        decompressed = adapter.decompress(compressed)
      end
    end

    x.compare!
  end.data

  benchmark_data.each do |entry|
    results[entry[:name]] = entry[:central_tendency]
  end

  results
end

def benchmark_multi_threaded(data, iterations, num_threads, time, warmup)
  results = {}
  benchmark_data = Benchmark.ips do |x|
    x.config(time: time, warmup: warmup)

    ADAPTERS.each do |name, adapter|
      x.report("#{name} multi-threaded") do
        threads = []
        num_threads.times do
          threads << Thread.new do
            iterations.times do
              thread_data = data.dup
              compressed = adapter.compress(thread_data)
              decompressed = adapter.decompress(compressed)
            end
          end
        end
        threads.each(&:join)
      end
    end

    x.compare!
  end.data

  benchmark_data.each do |entry|
    results[entry[:name]] = entry[:central_tendency]
  end

  results
end

# Usage example
url = "https://sun.aei.polsl.pl/~sdeor/corpus/webster.bz2"
sample_sizes = [
  128,
  256,
  512,
  1024,
  2048,
  4096,
  8192,
  16384,
  32768,
  65536,
  128 * 1024,
  256 * 1024,
  512 * 1024,
  1024 * 1024,
]
content = download_and_unzip(url)
data_samples = sample_content(content, sample_sizes)
iterations = 100
num_threads = 10
time = 10
warmup = 3
csv_data = []

data_samples.each do |size_bytes, data|
  puts "Benchmarking for size: #{size_bytes / 1024} KiB"

  single_threaded_results = benchmark_single_threaded(data, iterations, time, warmup)
  multi_threaded_results = benchmark_multi_threaded(data, iterations, num_threads, time, warmup)

  lz4_single = single_threaded_results["LZ4 single-threaded"]
  lz4flex_single = single_threaded_results["Lz4Flex single-threaded"]
  lz4_multi = multi_threaded_results["LZ4 multi-threaded"]
  lz4flex_multi = multi_threaded_results["Lz4Flex multi-threaded"]

  single_threaded_speedup = lz4flex_single / lz4_single
  multi_threaded_speedup = lz4flex_multi / lz4_multi

  csv_data << [
    size_bytes,
    lz4_single,
    lz4flex_single,
    single_threaded_speedup,
    lz4_multi,
    lz4flex_multi,
    multi_threaded_speedup,
  ].map do |v|
    v.respond_to?(:round) ? v.round(2) : v
  end
end

csv_file = "tmp/benchmarks-#{Time.now.strftime("%Y%m%d-%H%M%S")}.csv"
CSV.open(csv_file, "wb") do |csv|
  csv << [
    "bytesize",
    "lz4_single_threaded",
    "lz4_flex_single_threaded",
    "single_threaded_speedup",
    "lz4_multi_threaded",
    "lz4_flex_multi_threaded",
    "multi_threaded_speedup",
  ]
  csv_data.each do |row|
    csv << row
  end
end

puts "Wrote CSV to disk #{csv_file}"
