# frozen_string_literal: true

require 'json'

module Rfmt
  module LSP
    # Reads and writes LSP JSON-RPC messages over an IO pair.
    class MessageIO
      HEADER_SEPARATOR = "\r\n\r\n"
      CONTENT_LENGTH = /\AContent-Length:\s*(\d+)\z/i

      def initialize(input: $stdin, output: $stdout)
        @input = input
        @output = output
      end

      def read_message
        headers = read_headers
        return nil if headers.nil?

        content_length = headers.fetch('content-length').to_i
        JSON.parse(@input.read(content_length))
      end

      def write_message(payload)
        body = JSON.generate(payload)
        @output.write("Content-Length: #{body.bytesize}#{HEADER_SEPARATOR}#{body}")
        @output.flush if @output.respond_to?(:flush)
      end

      private

      def read_headers
        headers = {}

        loop do
          line = @input.gets
          return nil if line.nil?

          line = line.chomp
          line = line.delete_suffix("\r")
          break if line.empty?

          match = CONTENT_LENGTH.match(line)
          headers['content-length'] = match[1] if match
        end

        headers.fetch('content-length')
        headers
      rescue KeyError
        nil
      end
    end
  end
end
