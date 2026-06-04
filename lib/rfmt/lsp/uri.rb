# frozen_string_literal: true

require 'uri'

module Rfmt
  module LSP
    module URI
      module_function

      def file_uri_to_path(uri)
        parsed = ::URI.parse(uri)
        return nil unless parsed.scheme == 'file'

        percent_decode(parsed.path)
      rescue ::URI::InvalidURIError
        nil
      end

      def path_to_file_uri(path)
        "file://#{percent_encode(File.expand_path(path))}"
      end

      def percent_decode(value)
        value.gsub(/%[0-9A-Fa-f]{2}/) { |match| [match[1..].to_i(16)].pack('C') }
      end

      def percent_encode(value)
        value.bytes.map do |byte|
          char = byte.chr
          char.match?(%r{[A-Za-z0-9._~/-]}) ? char : format('%%%02X', byte)
        end.join
      end
    end
  end
end
