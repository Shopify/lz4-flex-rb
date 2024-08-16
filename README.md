# `lz4-flex-rb`

## About this library

**Introduction:**
`lz4-flex-rb` is a modern LZ4 compression library for Ruby, leveraging the
power of the [`lz4_flex`](https://github.com/PSeitz/lz4_flex) Rust crate. This
library provides a pure Rust implementation of the LZ4 algorithm, ensuring high
performance and safety. One of the standout features of `lz4_flex-rb` is its
ability to conditionally unlock the Global VM Lock (GVL) for threaded web
servers, enhancing concurrency and performance in multi-threaded environments.

## How to install this library

### Requirements
- Ruby 3.0 or higher
- Rust (for building the native extension)

### Setup
Add this line to your application's Gemfile:

```ruby
gem 'lz4_flex', github: "Shopify/lz4-flex-rb"
```

And then execute:

```sh
bundle install
```

### Troubleshooting

If you encounter issues during installation, ensure that Rust is correctly installed and available in your PATH. You can install Rust from [rustup.rs](https://rustup.rs/).

## How to use this library

There are two methods provided, `LZ4Flex.compress` and `Lz4Flex.decompress`.
Both of these methods utilize the lz4 block format, with a custom 64 bit header
to keep track of the string's size and encoding. 

### Basic Usage

```ruby
require 'lz4_flex'

# Compress data
compressed = LZ4Flex.compress("Hello, World!")

# Decompress data
decompressed = LZ4Flex.decompress(compressed)

puts decompressed  # => "Hello, World!"
```

The header used in these methods will  not be recognizable from other lz4 block
parsers. If you need that, it's best to use the Frame API (which is currently a
WIP).


#### Header spec

The below describes the u64-sized header structure with its fields, in order:


- `version`: u8 (version of header format, expected value: 1)
- `encoding`: u8 (encoding of the string)
  - Utf8 = 0
  - Binary = 1
  - UsAscii = 2,
- `__reserved`: [u8; 2] (initialized to zero, reserved for later use in case needed)
- `content_sizes`: u32 (size of the content string)

### Running Tests
To run the tests, execute:

```sh
bundle exec rake
```

## Contribute to this library (optional)

1. Fork the repository.
2. Create a new branch (`git checkout -b my-feature-branch`).
3. Make your changes.
4. Commit your changes (`git commit -am 'Add new feature'`).
5. Push to the branch (`git push origin my-feature-branch`).
6. Create a new Pull Request.

Please ensure your code adheres to the project's coding standards and includes appropriate tests.
