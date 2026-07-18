# frozen_string_literal: true

require 'spec_helper'

# Every construct whose rule emits verbatim source must mark the swept
# comments as emitted, or they duplicate at EOF and grow on every pass.
RSpec.describe Rfmt, 'comment emission invariant' do
  CASES = {
    'variable write with do..end block' => <<~RUBY,
      comments = result.comments.map do |comment|
        # inner note
        line = loc.end_line - 1
      end

      JSON.generate({ ast: 1 })
    RUBY
    'modifier if' => <<~RUBY,
      foo(
        # inner note
        bar
      ) if condition
    RUBY
    'postfix while' => <<~RUBY,
      process(
        # inner note
        item
      ) while queue.any?
    RUBY
    'ternary with multiline branch' => <<~RUBY
      x = cond ? foo(
        # inner note
        bar
      ) : baz
    RUBY
  }.freeze

  CASES.each do |name, source|
    context "with #{name}" do
      it 'emits each comment exactly once' do
        expect(Rfmt.format(source).scan('# inner note').size).to eq(1)
      end

      it 'is idempotent' do
        first = Rfmt.format(source)

        expect(Rfmt.format(first)).to eq(first)
      end
    end
  end
end
