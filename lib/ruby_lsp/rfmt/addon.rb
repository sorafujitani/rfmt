# frozen_string_literal: true

require "ruby_lsp/addon"
require_relative "formatter_runner"

module RubyLsp
  module Rfmt
    class Addon < ::RubyLsp::Addon
      def name
        "rfmt"
      end

      def activate(global_state, message_queue)
        global_state.register_formatter("rfmt", FormatterRunner.new)
      end

      def deactivate; end
    end
  end
end
