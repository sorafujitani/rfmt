# frozen_string_literal: true

require 'language_server/protocol'
require 'kenshin/version'

require_relative 'document_store'
require_relative 'formatter'
require_relative 'message_io'
require_relative 'uri'
require_relative 'workspace'

module Kenshin
  module LSP
    class Server
      TEXT_DOCUMENT_SYNC_FULL = 1
      METHOD_NOT_FOUND = -32_601
      INTERNAL_ERROR = -32_603

      def initialize(input: $stdin, output: $stdout)
        @io = MessageIO.new(input: input, output: output)
        @documents = DocumentStore.new
        @workspace = Workspace.new
        @shutdown_requested = false
      end

      def run
        while (message = @io.read_message)
          return @shutdown_requested ? 0 : 1 if handle_message(message) == :exit
        end

        0
      end

      def handle_message(message)
        method = message['method']
        id = message['id']
        params = message['params'] || {}

        dispatch_message(method, id, params)
      rescue StandardError => e
        respond_error(id, INTERNAL_ERROR, e.message) if id
      end

      private

      def dispatch_message(method, id, params)
        if request_handler?(method)
          dispatch_request(method, id, params)
        else
          dispatch_notification(method, params) || respond_method_not_found(id, method)
        end
      end

      def request_handler?(method)
        %w[initialize textDocument/formatting shutdown].include?(method)
      end

      def dispatch_request(method, id, params)
        case method
        when 'initialize'
          handle_initialize(id, params)
        when 'textDocument/formatting'
          handle_formatting(id, params)
        when 'shutdown'
          handle_shutdown(id)
        end
      end

      def dispatch_notification(method, params)
        case method
        when 'initialized'
          true
        when 'textDocument/didOpen'
          handle_did_open(params)
          true
        when 'textDocument/didChange'
          handle_did_change(params)
          true
        when 'textDocument/didClose'
          handle_did_close(params)
          true
        when 'exit'
          :exit
        end
      end

      def respond_method_not_found(id, method)
        respond_error(id, METHOD_NOT_FOUND, "Method not found: #{method}") if id
      end

      def handle_initialize(id, params)
        @workspace.configure(params)

        respond(id, {
                  capabilities: {
                    documentFormattingProvider: true,
                    textDocumentSync: TEXT_DOCUMENT_SYNC_FULL
                  },
                  serverInfo: {
                    name: 'kenshin',
                    version: Kenshin::VERSION
                  }
                })
      end

      def handle_did_open(params)
        text_document = params.fetch('textDocument')
        @documents.open(text_document.fetch('uri'), text_document.fetch('text'))
      end

      def handle_did_change(params)
        uri = params.fetch('textDocument').fetch('uri')
        change = Array(params['contentChanges']).last
        return unless change&.key?('text')

        @documents.change(uri, change.fetch('text'))
      end

      def handle_did_close(params)
        uri = params.fetch('textDocument').fetch('uri')
        @documents.close(uri)
      end

      def handle_formatting(id, params)
        uri = params.fetch('textDocument').fetch('uri')
        source = @documents.source_for(uri) || read_file_source(uri)
        edits = if source
                  @workspace.with_root_for(uri) { Formatter.format_edits(source) }
                else
                  []
                end

        respond(id, edits)
      end

      def handle_shutdown(id)
        @shutdown_requested = true
        respond(id, nil)
      end

      def read_file_source(uri)
        path = URI.file_uri_to_path(uri)
        return nil unless path && File.file?(path)

        File.read(path)
      end

      def respond(id, result)
        @io.write_message({
                            jsonrpc: '2.0',
                            id: id,
                            result: result
                          })
      end

      def respond_error(id, code, message)
        @io.write_message({
                            jsonrpc: '2.0',
                            id: id,
                            error: {
                              code: code,
                              message: message
                            }
                          })
      end
    end
  end
end
