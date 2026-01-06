# frozen_string_literal: true

require_relative 'lib/rfmt/version'

Gem::Specification.new do |spec|
  spec.name = 'rfmt'
  spec.version = Rfmt::VERSION
  spec.authors = ['fujitani sora']
  spec.email = ['fujitanisora0414@gmail.com']

  spec.summary = 'Ruby Formatter impl Rust lang.'
  spec.description = 'Write a longer description or delete this line.'
  spec.homepage = 'https://github.com/fs0414/rfmt'
  spec.license = 'MIT'
  spec.required_ruby_version = '>= 3.1.0'
  spec.required_rubygems_version = '>= 3.0.0'

  spec.metadata['allowed_push_host'] = 'https://rubygems.org'
  spec.metadata['homepage_uri'] = spec.homepage
  spec.metadata['source_code_uri'] = 'https://github.com/fs0414/rfmt'
  spec.metadata['changelog_uri'] = 'https://github.com/fs0414/rfmt/releases'
  spec.metadata['ruby_lsp_addon'] = 'true'

  # Specify which files should be added to the gem when it is released.
  # Explicitly list files to avoid including compiled artifacts (.bundle, .so)
  spec.files = Dir[
    'lib/**/*.rb',
    'ext/**/*.{rb,rs,toml}',
    'Cargo.toml',
    'Cargo.lock',
    'README.md',
    'LICENSE.txt',
    'CHANGELOG.md'
  ]
  spec.bindir = 'exe'
  spec.executables = ['rfmt']
  spec.require_paths = ['lib']
  spec.extensions = ['ext/rfmt/extconf.rb']

  spec.add_dependency 'rb_sys', '~> 0.9.91'
end
