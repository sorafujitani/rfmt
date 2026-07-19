# frozen_string_literal: true

require 'spec_helper'

RSpec.describe Kenshin, 'Comment duplication in do…end blocks (#112)' do
  let(:source) do
    <<~RUBY
      comments = result.comments.map do |comment|
        # first line of a long explanation
        # second line of the explanation
        line = loc.end_line - 1
        { text: loc.slice, position: "leading" } # trailing comment here
      end

      JSON.generate({ ast: 1 })
    RUBY
  end

  it 'emits each block-interior comment exactly once' do
    formatted = Kenshin.format(source)

    expect(formatted.scan('# first line of a long explanation').size).to eq(1)
    expect(formatted.scan('# second line of the explanation').size).to eq(1)
    expect(formatted.scan('# trailing comment here').size).to eq(1)
  end

  it 'is idempotent' do
    first = Kenshin.format(source)
    second = Kenshin.format(first)

    expect(second).to eq(first)
  end
end
