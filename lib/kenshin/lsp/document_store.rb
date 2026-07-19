# frozen_string_literal: true

module Kenshin
  module LSP
    class DocumentStore
      def initialize
        @documents = {}
      end

      def open(uri, text)
        @documents[uri] = text
      end

      def change(uri, text)
        @documents[uri] = text
      end

      def close(uri)
        @documents.delete(uri)
      end

      def source_for(uri)
        @documents[uri]
      end
    end
  end
end
