# frozen_string_literal: true

require 'spec_helper'

RSpec.describe Rfmt, 'Chain reformatting with multi-line arguments' do
  def idempotent(source)
    first = Rfmt.format(source)
    second = Rfmt.format(first)
    expect(second).to eq(first), "non-idempotent output:\n#{first}"
    first
  end

  it 'shifts multi-line args inside a chain call down with the chain itself' do
    source = <<~RUBY
      module Api
        class UsersController
          def ranking
            @users = User.left_joins(articles: :article_likes)
                         .group('users.id')
                         .select(
                           'users.*, ' \\
                           'COUNT(CASE WHEN articles.status = 1 THEN article_likes.id END) AS received_likes_count'
                         )
                         .having('received_likes_count > 0')
                         .order('received_likes_count DESC')
                         .limit(5)
          end
        end
      end
    RUBY
    formatted = idempotent(source)

    # After chain reformat, `.select(`, the `)`, and each chain method should
    # all sit at the same column (base_indent + indent_width = 8), and the
    # args inside `.select(...)` should be exactly one indent deeper (10).
    expect(formatted).to include("        .select(\n          'users.*, '")
    expect(formatted).to match(/^        \)\n {8}\.having/m)
    expect(formatted).to include("        .having('received_likes_count > 0')")
    expect(formatted).to include('        .limit(5)')
  end

  it 'does not shift non-chain lines when no chain reformat happens' do
    source = <<~RUBY
      foo(
        arg1,
        arg2
      )
    RUBY
    expect(idempotent(source)).to eq(source)
  end

  it 'preserves heredoc body indent when reformatting a tail chain' do
    source = <<~RUBY
      class Q
        def call
          query = <<~SQL
            SELECT *
          SQL
          query.squish
        end
      end
    RUBY
    formatted = idempotent(source)
    expect(formatted).to include("      SELECT *\n    SQL")
  end
end
