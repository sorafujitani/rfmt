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

    expected = <<~RUBY
      def index
        @posts = Post.published
          .includes(:author)
          .order(created_at: :desc)
      end
    RUBY

    first = Rfmt.format(source)
    expect(first).to eq(expected)
    expect(Rfmt.format(first)).to eq(first), "non-idempotent output:\n#{first}"
  end

  it 'keeps plain heredoc body content byte-identical inside a reformatted chain' do
    require 'prism'
    source = "result = base\n      .where(<<SQL)\n        a = 1\nSQL\n      .order(:id)\n"

    heredoc_contents = lambda do |code|
      contents = []
      collect = lambda do |node|
        contents << node.unescaped if node.is_a?(Prism::StringNode)
        node.compact_child_nodes.each { |child| collect.call(child) }
      end
      collect.call(Prism.parse(code).value)
      contents
    end

    first = Rfmt.format(source)
    expect(heredoc_contents.call(first)).to eq(heredoc_contents.call(source))
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

    expected = <<~RUBY
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
    expect(first).to eq(expected)
    expect(Rfmt.format(first)).to eq(first), "non-idempotent output:\n#{first}"
  end
end
