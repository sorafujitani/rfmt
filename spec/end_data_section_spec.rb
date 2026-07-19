# frozen_string_literal: true

require 'spec_helper'

RSpec.describe Rfmt, '__END__ data section preservation' do
  it 'keeps the __END__ line and everything after it' do
    source = <<~RUBY
      puts DATA.read

      __END__
      hello
      world
    RUBY

    formatted = Rfmt.format(source)

    expect(formatted).to end_with("__END__\nhello\nworld\n")
  end

  it 'preserves data content verbatim, including blank lines and indentation' do
    source = "x = 1\n__END__\n  indented\n\nspaced  \n"

    formatted = Rfmt.format(source)

    expect(formatted[/^__END__\n(.*)/m, 1]).to eq("  indented\n\nspaced  \n")
  end

  it 'stays idempotent with a data section present' do
    source = <<~RUBY
      class SeedLoader
        def load!
          DATA.each_line { |line| Tag.create!(name: line.strip) }
        end
      end

      __END__
      ruby
      rails
    RUBY

    first = Rfmt.format(source)
    expect(Rfmt.format(first)).to eq(first)
  end

  it 'does not invent a data section when none exists' do
    expect(Rfmt.format("x = 1\n")).not_to include('__END__')
  end
end
