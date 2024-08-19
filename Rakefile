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

Rake::Task["release"].clear

desc "Trigger publishing of a new release"
task :release do
  abort("ERROR: uncommited changes") unless system("git diff --exit-code")

  old_version = GEMSPEC.version.to_s
  print "Enter new version (current is #{old_version}): "
  new_version = $stdin.gets.strip
  new_git_tag = "v#{new_version}"

  abort("ERROR: #{GEMSPEC.version} tag already exists") if system("git rev-parse #{new_git_tag}")

  old_version_rb = File.read("lib/lz4_flex/version.rb")
  new_version_rb = old_version_rb.gsub("VERSION = \"#{old_version}\"", "VERSION = \"#{new_version}\"")

  File.write("lib/lz4_flex/version.rb", new_version_rb)
  diff = %x(git diff)

  puts "Diff:\n#{diff}"
  print "Does this look good? (y/n): "

  if $stdin.gets.strip == "y"
    sh "bundle"
    sh "git commit -am \"Bump version to #{new_git_tag}\""
    sh "git tag #{new_git_tag}"
    sh "git push"
    sh "git push --tags"

    sleep 3

    runs = %x(gh run list -w release -e push -b #{new_git_tag} --json=databaseId --jq '.[].databaseId')
    runs = runs.strip.split("\n")

    if runs.empty?
      warn("WARN: no release runs found")
    elsif runs.length > 1
      warn("WARN: multiple release runs found")
    else
      puts "Watching release run #{runs.first}, safe to Ctrl-C..."
      sleep 3
      system("gh run watch #{runs.first}")
      shipit_link = "https://shipit.shopify.io/shopify/lz4-flex-rb/release"
      system("osascript -e 'display notification \"Release complete -> #{shipit_link}\" with title \"lz4_flex\"'")
      puts "Release complete, see #{shipit_link}"
    end
  else
    File.write("lib/lz4_flex/version.rb", old_version_rb)
    puts "Aborting release"
  end
end

task default: [:compile, :test, :rubocop]
