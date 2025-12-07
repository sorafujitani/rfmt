# frozen_string_literal: true

source 'https://rubygems.org'

# Specify your gem's dependencies in rfmt.gemspec
gemspec

# Parser
gem 'prism', '~> 1.6.0'

# CLI
gem 'diff-lcs', '~> 1.5'
gem 'diffy', '~> 3.4'
gem 'parallel', '~> 1.24'
gem 'thor', '~> 1.3'

# Development
gem 'lefthook', '~> 1.5'
gem 'rake', '~> 13.0'
gem 'rake-compiler', '~> 1.2'
gem 'rspec', '~> 3.12'
gem 'rubocop', '~> 1.59'

# Ruby 3.1 compatibility
# These stdlib gems have versions that require Ruby 3.2+, so we constrain them
# to versions compatible with Ruby 3.1
gem 'irb', '< 1.15'        # 1.15+ requires rdoc which requires newer erb
gem 'rdoc', '< 6.7'        # 6.7+ requires erb that needs Ruby 3.2+

# Testing
group :test do
  gem 'rspec-benchmark', '~> 0.6'
  gem 'simplecov', '~> 0.22'
end

# Ruby LSP integration (optional)
group :development do
  gem 'ruby-lsp', '>= 0.17.0'
end
