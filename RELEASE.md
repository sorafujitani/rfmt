# Release Process

This document describes the release process for kenshin.

## Overview

kenshin uses an automated release process powered by GitHub Actions. The process includes:

1. Version bumping and changelog updates
2. Running tests
3. Building native gems for multiple platforms
4. Creating GitHub releases
5. Publishing to RubyGems

## Prerequisites

### For Maintainers

- Write access to the repository
- RubyGems account with API key (for publishing)
- Ruby 3.3+ installed locally

### GitHub Secrets

The following secrets should be configured in GitHub repository settings:

- `RUBYGEMS_API_KEY` - Your RubyGems API key for publishing gems

To get your RubyGems API key:
```bash
# Login to RubyGems
gem signin

# Create an API key
gem signin --key kenshin

# View your credentials
cat ~/.gem/credentials
```

Add the key to GitHub:
1. Go to repository Settings → Secrets and variables → Actions
2. Click "New repository secret"
3. Name: `RUBYGEMS_API_KEY`
4. Value: Your RubyGems API key

## Release Steps

### 1. Prepare the Release

Use the automated release script:

```bash
# Interactive mode (recommended)
bin/release

# Or specify version directly
bin/release 0.2.0

# Dry run to test without making changes
bin/release 0.2.0 --dry-run
```

The script will:
- ✅ Prompt for the new version (or suggest patch/minor/major)
- ✅ Update `lib/kenshin/version.rb`
- ✅ Update `CHANGELOG.md` with release date
- ✅ Run the test suite
- ✅ Commit changes
- ✅ Create a git tag

### 2. Review Changes

Before pushing, review the changes:

```bash
# Check the commit
git show

# Check the tag
git tag -l -n9 v0.2.0
```

If you need to make corrections:

```bash
# Undo the commit and tag
git reset --hard HEAD~1
git tag -d v0.2.0

# Make corrections and run bin/release again
```

### 3. Push to GitHub

Push both the commit and the tag:

```bash
# Push the commit
git push origin main

# Push the tag (this triggers the release workflow)
git push origin v0.2.0
```

⚠️ **Important**: Once you push the tag, the automated release process begins immediately.

### 4. Automated Build and Release

GitHub Actions will automatically:

1. **Validate the release** (.github/workflows/release.yml)
   - Verify version.rb matches the tag
   - Check CHANGELOG.md is updated

2. **Build native gems** for all platforms:
   - x86_64-linux
   - aarch64-linux
   - x86_64-darwin
   - arm64-darwin
   - x64-mingw-ucrt

3. **Create GitHub Release**
   - Creates a draft release
   - Uploads all platform-specific gems
   - Generates release notes from CHANGELOG.md

4. **Publish to RubyGems**
   - Automatically publishes the gem (if `RUBYGEMS_API_KEY` is configured)

### 5. Finalize the Release

1. Go to [GitHub Releases](https://github.com/sorafujitani/rfmt/releases)
2. Find the draft release
3. Review the release notes
4. Edit if needed (use `.github/release_template.md` as a reference)
5. Click "Publish release"

## Release Checklist

Before releasing, ensure:

- [ ] All tests pass (`bundle exec rspec`)
- [ ] CHANGELOG.md is up to date with all changes
- [ ] Version number follows [Semantic Versioning](https://semver.org/)
- [ ] Documentation is updated (if needed)
- [ ] No pending PRs that should be included

## Version Numbering

kenshin follows [Semantic Versioning](https://semver.org/):

- **MAJOR** (X.0.0): Breaking changes
- **MINOR** (0.X.0): New features, backwards compatible
- **PATCH** (0.0.X): Bug fixes, backwards compatible

## Troubleshooting

### Build fails for a specific platform

If a platform build fails:
1. Check the GitHub Actions logs
2. The workflow continues even if one platform fails
3. You can manually rebuild for that platform later

### Version mismatch error

If you see "Version mismatch" error:
```
Error: Version mismatch!
Tag: v0.2.0
lib/kenshin/version.rb: 0.1.0
```

Fix it by updating the version file:
```ruby
# lib/kenshin/version.rb
module Kenshin
  VERSION = "0.2.0"
end
```

### CHANGELOG not updated

If you see "CHANGELOG.md does not contain version" error:
```
Error: CHANGELOG.md does not contain version 0.2.0
```

Add the version entry to CHANGELOG.md:
```markdown
## [0.2.0] - 2025-11-24

- New feature or fix
```

### RubyGems publish fails

If gem push fails:
1. Check that `RUBYGEMS_API_KEY` secret is set
2. Verify the API key has publish permissions
3. Check the gem name is available on RubyGems

Manual publish:
```bash
# Build the gem
bundle exec rake build

# Push to RubyGems
gem push pkg/kenshin-0.2.0.gem
```

## Manual Release (Emergency)

If the automated process fails, you can release manually:

```bash
# 1. Update version and changelog (as above)

# 2. Build the gem
bundle exec rake build

# 3. Create and push tag
git tag -a v0.2.0 -m "Release v0.2.0"
git push origin v0.2.0

# 4. Create GitHub release manually
gh release create v0.2.0 \
  --title "v0.2.0" \
  --notes-file CHANGELOG.md \
  pkg/kenshin-0.2.0.gem

# 5. Push to RubyGems
gem push pkg/kenshin-0.2.0.gem
```

## Post-Release

After releasing:

1. Announce the release:
   - Post to relevant Ruby forums/communities
   - Tweet about new features
   - Update project website (if applicable)

2. Monitor for issues:
   - Watch GitHub issues for bug reports
   - Check RubyGems.org download stats

3. Prepare for next release:
   - Add `## [Unreleased]` section to CHANGELOG.md if not present
   - Plan next features/fixes

## Release Cadence

- **Patch releases**: As needed for critical bugs
- **Minor releases**: Monthly or when significant features are ready
- **Major releases**: When breaking changes are necessary

## Support Policy

- **Latest version**: Full support
- **Previous minor version**: Security fixes only
- **Older versions**: No support

## References

- [Semantic Versioning](https://semver.org/)
- [Keep a Changelog](https://keepachangelog.com/)
- [GitHub Releases](https://docs.github.com/en/repositories/releasing-projects-on-github)
- [RubyGems Publishing](https://guides.rubygems.org/publishing/)
