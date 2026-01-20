# Contributing to lz4-flex-rb

Thank you for your interest in contributing to `lz4-flex-rb`! This document provides guidelines for development, testing, and releasing this Ruby gem.

## Table of Contents

1. [Development Setup](#development-setup)
2. [Code Standards](#code-standards)
3. [Testing](#testing)
4. [Submitting Changes](#submitting-changes)
5. [Release Process](#release-process)
   - [Prerequisites](#prerequisites)
   - [Release Checklist](#release-checklist)
   - [Step-by-Step Release Instructions](#step-by-step-release-instructions)
   - [What Happens During Release](#what-happens-during-release)
   - [Troubleshooting](#troubleshooting)

## Development Setup

**Prerequisites:**

- Ruby 3.2 or higher
- Rust 1.81.0 or higher
- Bundler

**Installation:**

```bash
# Install dependencies
bundle install

# Compile the native extension
bundle exec rake compile
```

For detailed usage examples, see the [README.md](README.md).

## Code Standards

This project uses automated linting tools to maintain code quality:

**Ruby:** Uses `rubocop-shopify` standards

```bash
bundle exec rake rubocop
```

**Rust:** Uses `clippy` and `rustfmt`

```bash
# Check formatting
cargo fmt --check

# Run linter
cargo clippy -- -D warnings
```

**Note:** CI automatically enforces these standards. All checks must pass before merging.

## Testing

**Run all checks (compile + test + rubocop):**

```bash
bundle exec rake
```

**Run only tests:**

```bash
bundle exec rake test
```

## Submitting Changes

We welcome contributions from the community! To submit changes:

1. **Fork the repository** on GitHub
2. **Create a feature branch:**
   ```bash
   git checkout -b my-feature-branch
   ```
3. **Make your changes** with clear, descriptive commit messages
4. **Push to your fork:**
   ```bash
   git push origin my-feature-branch
   ```
5. **Open a Pull Request** to the `main` branch

**Important:** All contributors must sign the [Shopify CLA](https://cla.shopify.com/).

**PR Checklist:**

- [ ] Tests pass locally (`bundle exec rake`)
- [ ] Rubocop and Clippy checks pass
- [ ] Clear description of changes in PR
- [ ] CLA signed

Tag `@Shopify/ruby-infra` for review.

## Release Process

This section documents the complete release process for maintainers with write access to the repository.

### Prerequisites

**Permissions Required:**

- Write access to [Shopify/lz4-flex-rb](https://github.com/Shopify/lz4-flex-rb) repository
- Shipit access: https://shipit.shopify.io/shopify/lz4-flex-rb

**Tools Required:**

- `gh` CLI installed and authenticated
  ```bash
  gh auth status
  ```
- Git configured with GitHub credentials
- Bundler installed

**State Requirements:**

- Clean working directory (no uncommitted changes)
- All PRs merged to `main` branch
- CI passing on `main`

### Release Checklist

Before starting a release, verify:

- [ ] All PRs merged to main
- [ ] CI passing on main branch
- [ ] New version number decided (use [semantic versioning](https://semver.org/))
- [ ] Clean working directory (`git status`)
- [ ] gh CLI authenticated (`gh auth status`)
- [ ] Shipit access confirmed

### Step-by-Step Release Instructions

#### Step 1: Start the Release

```bash
bundle exec rake release
```

The task will:

- Check for uncommitted changes (aborts if found)
- Prompt for new version number

#### Step 2: Enter New Version

When prompted, enter the new version number **without** the `v` prefix:

```
Enter new version (current is 1.0.1): 1.0.2
```

⚠️ **Important:** Enter `1.0.2` (not `v1.0.2`). The task will automatically create the `v1.0.2` git tag.

#### Step 3: Review Changes

The task will display a `git diff` showing the changes to `lib/lz4_flex/version.rb`:

```diff
-  VERSION = "1.0.1"
+  VERSION = "1.0.2"
```

Confirm with `y` if correct. Any other input aborts the release.

#### Step 4: Automated Steps Execute

Once confirmed, the rake task automatically:

1. Updates `lib/lz4_flex/version.rb`
2. Runs `bundle install` to update `Gemfile.lock`
3. Creates git commit: `"Bump version to v1.0.2"`
4. Creates git tag: `v1.0.2`
5. Pushes commit: `git push`
6. Pushes tag: `git push --tags`
7. Watches the GitHub Actions release workflow

#### Step 5: Wait for GitHub Actions

The `.github/workflows/release.yml` workflow starts automatically when the tag is pushed.

**What the workflow does:**

1. **Builds source gem** on ubuntu-latest
2. **Cross-compiles native gems** for platforms:
   - `x86_64-linux`, `x86_64-linux-musl`
   - `aarch64-linux`, `aarch64-linux-musl`
   - `x86_64-darwin`, `arm64-darwin`
3. **Tests gem installations** on ubuntu-latest and macos-latest
4. **Creates a DRAFT GitHub release** (not published)
5. **Uploads all gem artifacts** to the release

⚠️ **Important:** The GitHub release is created as a **DRAFT**. It will NOT be published automatically.

**Monitoring:**

- The rake task will watch the workflow run automatically
- You can safely `Ctrl-C` to exit the watch
- Check progress at: https://github.com/Shopify/lz4-flex-rb/actions

**Expected duration:** 10-15 minutes

#### Step 6: Publish GitHub Release (Manual)

Once the GitHub Actions workflow completes:

1. Go to: https://github.com/Shopify/lz4-flex-rb/releases
2. Find the **DRAFT** release for your version (e.g., `v1.0.2`)
3. Review the auto-generated release notes
4. Edit release notes if needed:
   - Add highlights or important changes
   - Note any breaking changes
   - Include migration instructions if applicable
5. Click **"Publish release"**

#### Step 7: Deploy via Shipit (Manual)

After publishing the GitHub release:

1. Go to: https://shipit.shopify.io/shopify/lz4-flex-rb/release
2. Find your version in the deployment list
3. Click the **"Deploy"** button

**What Shipit does:**

- **Pre-check:** Runs `./scripts/pre_release`
  - Validates that a git tag exists on HEAD
- **Deploy:** Runs `./scripts/release`
  - Downloads gem artifacts from the GitHub release
  - Pushes all gems to rubygems.org (with `SHIPIT=1` environment variable)

#### Step 8: Verify Release

After Shipit deployment completes:

```bash
# Check RubyGems.org
open https://rubygems.org/gems/lz4_flex

# Test installation
gem install lz4_flex -v 1.0.2

# Verify it works
ruby -r lz4_flex -e 'puts Lz4Flex.compress("test")'
```

The new version should appear on RubyGems.org within a few minutes.
