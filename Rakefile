# frozen_string_literal: true

require "bundler/gem_tasks"
require "rake/testtask"
require "rb_sys/extensiontask"

Rake::TestTask.new(:test) do |t|
  t.ruby_opts = ["-W0", "-W:deprecated"]
  t.libs << "test"
  t.libs << "lib"
  t.test_files = FileList["test/**/*_test.rb"]
end

require "rubocop/rake_task"

RuboCop::RakeTask.new

GEMSPEC = Gem::Specification.load("lz4_flex.gemspec")

RbSys::ExtensionTask.new("lz4_flex_ext", GEMSPEC) do |ext|
  ext.lib_dir = "lib/lz4_flex"
end

task default: [:compile, :test, :rubocop]
