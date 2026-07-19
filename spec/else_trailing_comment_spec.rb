# frozen_string_literal: true

require 'spec_helper'

RSpec.describe Rfmt, 'trailing comment on else' do
  it 'keeps the comment on the else line' do
    source = <<~RUBY
      if big?
        :big
      else # inline on else
        :small
      end
    RUBY

    expect(Rfmt.format(source)).to include("else # inline on else\n")
  end

  it 'keeps else comments inline alongside if and end comments' do
    source = <<~RUBY
      result = if chain > 10 # inline on if
        :big
      else # inline on else
        :small
      end # inline on end
    RUBY

    formatted = Rfmt.format(source)

    expect(formatted).to include("# inline on if\n")
    expect(formatted).to include("else # inline on else\n")
    expect(formatted).to include("end # inline on end\n")
  end

  it 'keeps the comment on the else line inside unless' do
    source = <<~RUBY
      unless ready?
        wait
      else # fall through
        run
      end
    RUBY

    expect(Rfmt.format(source)).to include("else # fall through\n")
  end
end
