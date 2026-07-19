# frozen_string_literal: true

require 'kenshin'

module Kenshin
  module LSP
    class Formatter
      def self.format_edits(source)
        formatted = source.empty? ? "\n" : Kenshin.format(source)
        return [] if formatted == source

        [
          {
            range: full_document_range(source),
            newText: formatted
          }
        ]
      rescue Kenshin::Error
        []
      end

      def self.full_document_range(source)
        return { start: { line: 0, character: 0 }, end: { line: 0, character: 0 } } if source.empty?

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
