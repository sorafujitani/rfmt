# frozen_string_literal: true

require 'spec_helper'

RSpec.describe Rfmt, 'chain indent idempotency on misindented input' do
  it 'indents chain continuations correctly in a single pass' do
    source = <<~RUBY
      def index
      @posts = Post.published
      .includes(:author)
      .order(created_at: :desc)
      end
    RUBY

    expected = <<~RUBY
      def index
        @posts = Post.published
          .includes(:author)
          .order(created_at: :desc)
      end
    RUBY

    expect(Rfmt.format(source)).to eq(expected)
  end

  it 'is idempotent when the input statement is deeper than its output position' do
    source = <<~RUBY
      def index
            @posts = Post.published
              .includes(:author)
              .order(created_at: :desc)
      end
    RUBY

    first = Rfmt.format(source)
    expect(Rfmt.format(first)).to eq(first), "non-idempotent output:\n#{first}"
  end

  it 'is idempotent for a nested misindented chain' do
    source = <<~RUBY
      module Api
      class PostsController
      def index
      @posts = Post.published
      .includes(:author)
      .page(params[:page])
      end
      end
      end
    RUBY

    first = Rfmt.format(source)
    expect(Rfmt.format(first)).to eq(first), "non-idempotent output:\n#{first}"
  end
end
