# frozen_string_literal: true

require 'spec_helper'

RSpec.describe Rfmt, 'Orphan comments inside do…end blocks' do
  def idempotent(source)
    first = Rfmt.format(source)
    second = Rfmt.format(first)
    expect(second).to eq(first), "non-idempotent output:\n#{first}"
    first
  end

  it 'keeps comments inside the block instead of concatenating them to `end`' do
    source = <<~RUBY
      RSpec.configure do |config|
        config.expect_with :rspec do |expectations|
          expectations.include_chain_clauses_in_custom_matcher_descriptions = true
        end

        # The settings below are suggested to provide a good initial experience
        # with RSpec, but feel free to customize to your heart's content.
        #   config.filter_run_when_matching :focus
      end
    RUBY
    formatted = idempotent(source)

    # The block's `end` must sit alone — no comment concatenated onto it.
    expect(formatted).not_to match(/\bend#/)
    expect(formatted).to match(/^end$/m)

    # The comments must remain *inside* the block, indented one level.
    expect(formatted).to include('  # The settings below')
    expect(formatted).to include('  #   config.filter_run_when_matching :focus')
  end

  it 'preserves the blank line between the last body statement and a trailing comment' do
    source = <<~RUBY
      class Foo
        def bar
          1
        end

        # trailing annotation
      end
    RUBY
    expect(idempotent(source)).to eq(source)
  end

  it 'does not add extra blank lines to file-level trailing comments on successive passes' do
    source = <<~RUBY
      class Main
      end

      # End comment
    RUBY
    expect(idempotent(source)).to eq(source)
  end
end
