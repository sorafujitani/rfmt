# frozen_string_literal: true

require 'ruby_lsp/addon'
require_relative 'formatter_runner'

module RubyLsp
  module Kenshin
    class Addon < ::RubyLsp::Addon
      def name
        'kenshin'
      end

      def activate(global_state, _message_queue)
        global_state.register_formatter('kenshin', FormatterRunner.new)
      end

      def deactivate; end
    end
  end
end
