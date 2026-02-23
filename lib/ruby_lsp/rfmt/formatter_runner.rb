# frozen_string_literal: true

require 'rfmt'

module RubyLsp
  module Rfmt
    class FormatterRunner
      # @param uri [URI::Generic] Document URI
      # @param document [RubyLsp::RubyDocument] Target document
      # @return [String, nil] Formatted text or nil on error
      def run_formatting(_uri, document)
        source = document.source
        return "\n" if source.empty?

        ::Rfmt.format(source)
      rescue ::Rfmt::Error
        nil
      end

      # @param uri [URI::Generic] Document URI
      # @param document [RubyLsp::RubyDocument] Target document
      # @return [Array<RubyLsp::Interface::Diagnostic>]
      def run_diagnostic(_uri, _document)
        []
      end
    end
  end
end
