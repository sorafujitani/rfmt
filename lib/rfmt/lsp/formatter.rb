# frozen_string_literal: true

require 'rfmt'

module Rfmt
  module LSP
    class Formatter
      def self.format_edits(source)
        formatted = source.empty? ? "\n" : Rfmt.format(source)
        return [] if formatted == source

        [
          {
            range: full_document_range(source),
            newText: formatted
          }
        ]
      rescue Rfmt::Error
        []
      end

      def self.full_document_range(source)
        lines = source.split("\n", -1)

        {
          start: { line: 0, character: 0 },
          end: {
            line: lines.length - 1,
            character: lines.last.length
          }
        }
      end
    end
  end
end
