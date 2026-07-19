# frozen_string_literal: true

require 'json'
require 'stringio'
require 'tmpdir'

require 'spec_helper'
require 'kenshin/lsp/server'

RSpec.describe Kenshin::LSP::Server do
  def build_server
    output = StringIO.new
    server = described_class.new(input: StringIO.new, output: output)
    [server, output]
  end

  def read_messages(output)
    io = StringIO.new(output.string)
    messages = []

    loop do
      header = io.gets
      break unless header

      length = header[/Content-Length:\s*(\d+)/i, 1].to_i
      io.gets
      messages << JSON.parse(io.read(length))
    end

    messages
  end

  def file_uri(path)
    Kenshin::LSP::URI.path_to_file_uri(path)
  end

  describe '#handle_message' do
    it 'returns formatting capabilities on initialize' do
      server, output = build_server

      server.handle_message({
                              'jsonrpc' => '2.0',
                              'id' => 1,
                              'method' => 'initialize',
                              'params' => {}
                            })

      response = read_messages(output).first
      expect(response['id']).to eq(1)
      expect(response.dig('result', 'capabilities', 'documentFormattingProvider')).to eq(true)
      expect(response.dig('result', 'capabilities', 'textDocumentSync')).to eq(1)
      expect(response.dig('result', 'serverInfo', 'name')).to eq('kenshin')
    end

    it 'formats an opened document with a full-document text edit' do
      server, output = build_server
      uri = file_uri('/tmp/test.rb')

      server.handle_message({
                              'jsonrpc' => '2.0',
                              'method' => 'textDocument/didOpen',
                              'params' => {
                                'textDocument' => {
                                  'uri' => uri,
                                  'text' => "class Foo\ndef bar\n42\nend\nend\n"
                                }
                              }
                            })
      server.handle_message({
                              'jsonrpc' => '2.0',
                              'id' => 2,
                              'method' => 'textDocument/formatting',
                              'params' => {
                                'textDocument' => { 'uri' => uri },
                                'options' => {}
                              }
                            })

      response = read_messages(output).last
      edit = response['result'].first
      expect(edit['range']).to eq({
                                    'start' => { 'line' => 0, 'character' => 0 },
                                    'end' => { 'line' => 5, 'character' => 0 }
                                  })
      expect(edit['newText']).to include("  def bar\n")
      expect(edit['newText']).to include("    42\n")
    end

    it 'returns no edits when formatting fails' do
      server, output = build_server
      uri = file_uri('/tmp/broken.rb')

      server.handle_message({
                              'jsonrpc' => '2.0',
                              'method' => 'textDocument/didOpen',
                              'params' => {
                                'textDocument' => {
                                  'uri' => uri,
                                  'text' => 'def foo('
                                }
                              }
                            })
      server.handle_message({
                              'jsonrpc' => '2.0',
                              'id' => 3,
                              'method' => 'textDocument/formatting',
                              'params' => {
                                'textDocument' => { 'uri' => uri },
                                'options' => {}
                              }
                            })

      expect(read_messages(output).last['result']).to eq([])
    end

    it 'formats an empty document as a newline' do
      server, output = build_server
      uri = file_uri('/tmp/empty.rb')

      server.handle_message({
                              'jsonrpc' => '2.0',
                              'method' => 'textDocument/didOpen',
                              'params' => {
                                'textDocument' => {
                                  'uri' => uri,
                                  'text' => ''
                                }
                              }
                            })
      server.handle_message({
                              'jsonrpc' => '2.0',
                              'id' => 4,
                              'method' => 'textDocument/formatting',
                              'params' => {
                                'textDocument' => { 'uri' => uri },
                                'options' => {}
                              }
                            })

      edit = read_messages(output).last['result'].first
      expect(edit['range']).to eq({
                                    'start' => { 'line' => 0, 'character' => 0 },
                                    'end' => { 'line' => 0, 'character' => 0 }
                                  })
      expect(edit['newText']).to eq("\n")
    end

    it 'uses the workspace root when discovering kenshin config' do
      Dir.mktmpdir do |root|
        File.write(File.join(root, '.kenshin.yml'), <<~YAML)
          version: "1.0"
          formatting:
            indent_width: 4
        YAML
        path = File.join(root, 'test.rb')
        uri = file_uri(path)
        server, output = build_server

        server.handle_message({
                                'jsonrpc' => '2.0',
                                'id' => 1,
                                'method' => 'initialize',
                                'params' => { 'rootUri' => file_uri(root) }
                              })
        server.handle_message({
                                'jsonrpc' => '2.0',
                                'method' => 'textDocument/didOpen',
                                'params' => {
                                  'textDocument' => {
                                    'uri' => uri,
                                    'text' => "class Foo\ndef bar\n42\nend\nend\n"
                                  }
                                }
                              })
        server.handle_message({
                                'jsonrpc' => '2.0',
                                'id' => 5,
                                'method' => 'textDocument/formatting',
                                'params' => {
                                  'textDocument' => { 'uri' => uri },
                                  'options' => {}
                                }
                              })

        edit = read_messages(output).last['result'].first
        expect(edit['newText']).to include("    def bar\n")
        expect(edit['newText']).to include("        42\n")
      end
    end

    it 'handles shutdown and exit' do
      server, output = build_server

      server.handle_message({
                              'jsonrpc' => '2.0',
                              'id' => 6,
                              'method' => 'shutdown',
                              'params' => nil
                            })
      result = server.handle_message({
                                       'jsonrpc' => '2.0',
                                       'method' => 'exit',
                                       'params' => nil
                                     })

      response = read_messages(output).last
      expect(response['id']).to eq(6)
      expect(response['result']).to be_nil
      expect(result).to eq(:exit)
    end
  end
end
