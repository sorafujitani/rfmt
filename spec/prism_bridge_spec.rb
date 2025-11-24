# frozen_string_literal: true

require 'spec_helper'
require 'tempfile'

RSpec.describe Rfmt::PrismBridge do
  describe '.parse' do
    context 'with simple program' do
      let(:source) { "puts 'hello'" }

      it 'returns JSON string' do
        result = described_class.parse(source)
        expect(result).to be_a(String)
        expect { JSON.parse(result) }.not_to raise_error
      end

      it 'contains program node' do
        result = JSON.parse(described_class.parse(source))
        expect(result['ast']['node_type']).to eq('program_node')
      end

      it 'includes location information' do
        result = JSON.parse(described_class.parse(source))
        location = result['ast']['location']

        expect(location).to include(
          'start_line' => 1,
          'start_column' => 0,
          'end_line' => 1
        )
        expect(location['start_offset']).to be >= 0
        expect(location['end_offset']).to be > location['start_offset']
      end
    end

    context 'with class definition' do
      let(:source) do
        <<~RUBY
          class Foo
            def bar
              42
            end
          end
        RUBY
      end

      it 'parses successfully' do
        result = described_class.parse(source)
        expect(result).to be_a(String)
      end

      it 'contains class node in children' do
        result = JSON.parse(described_class.parse(source))
        class_node = result['ast']['children'].first

        expect(class_node['node_type']).to eq('class_node')
      end

      it 'extracts class name metadata' do
        result = JSON.parse(described_class.parse(source))
        class_node = result['ast']['children'].first

        expect(class_node['metadata']).to include('name' => 'Foo')
      end
    end

    context 'with method definition' do
      let(:source) do
        <<~RUBY
          def hello(name)
            puts name
          end
        RUBY
      end

      it 'parses method definition' do
        result = JSON.parse(described_class.parse(source))
        def_node = result['ast']['children'].first

        expect(def_node['node_type']).to eq('def_node')
      end

      it 'extracts method name' do
        result = JSON.parse(described_class.parse(source))
        def_node = result['ast']['children'].first

        expect(def_node['metadata']['name']).to eq('hello')
      end
    end

    context 'with literals' do
      it 'parses string literals' do
        result = JSON.parse(described_class.parse('"hello"'))
        string_node = result['ast']['children'].first

        expect(string_node['node_type']).to eq('string_node')
        expect(string_node['metadata']['content']).to eq('hello')
      end

      it 'parses integer literals' do
        result = JSON.parse(described_class.parse('42'))
        integer_node = result['ast']['children'].first

        expect(integer_node['node_type']).to eq('integer_node')
        expect(integer_node['metadata']['value']).to eq('42')
      end

      it 'parses float literals' do
        result = JSON.parse(described_class.parse('3.14'))
        float_node = result['ast']['children'].first

        expect(float_node['node_type']).to eq('float_node')
        expect(float_node['metadata']['value']).to eq('3.14')
      end
    end

    context 'with multiline detection' do
      it 'detects single-line code' do
        result = JSON.parse(described_class.parse("puts 'hello'"))
        formatting = result['ast']['formatting']

        expect(formatting['multiline']).to be false
      end

      it 'detects multi-line code' do
        source = <<~RUBY
          class Foo
          end
        RUBY
        result = JSON.parse(described_class.parse(source))
        class_node = result['ast']['children'].first
        formatting = class_node['formatting']

        expect(formatting['multiline']).to be true
      end
    end

    context 'with syntax errors' do
      let(:invalid_source) { "class Foo\n  def\nend" }

      it 'raises ParseError' do
        expect do
          described_class.parse(invalid_source)
        end.to raise_error(Rfmt::PrismBridge::ParseError, /Parse errors/)
      end

      it 'includes error location in message' do
        described_class.parse(invalid_source)
        raise 'Expected ParseError to be raised'
      rescue Rfmt::PrismBridge::ParseError => e
        expect(e.message).to match(/\d+:\d+:/)
      end
    end
  end

  describe '.parse_file' do
    let(:temp_file) { Tempfile.new(['test', '.rb']) }

    after { temp_file.unlink }

    it 'parses file successfully' do
      temp_file.write("puts 'hello'")
      temp_file.rewind

      result = described_class.parse_file(temp_file.path)
      expect(result).to be_a(String)

      parsed = JSON.parse(result)
      expect(parsed['ast']['node_type']).to eq('program_node')
    end

    it 'raises ParseError for non-existent file' do
      expect do
        described_class.parse_file('/non/existent/file.rb')
      end.to raise_error(Rfmt::PrismBridge::ParseError, /File not found/)
    end
  end

  describe 'node type conversion' do
    it 'converts ProgramNode to program_node' do
      result = JSON.parse(described_class.parse("puts 'hello'"))
      expect(result['ast']['node_type']).to eq('program_node')
    end

    it 'converts ClassNode to class_node' do
      result = JSON.parse(described_class.parse('class Foo; end'))
      class_node = result['ast']['children'].first
      expect(class_node['node_type']).to eq('class_node')
    end

    it 'converts DefNode to def_node' do
      result = JSON.parse(described_class.parse('def foo; end'))
      def_node = result['ast']['children'].first
      expect(def_node['node_type']).to eq('def_node')
    end
  end

  describe 'formatting information' do
    it 'includes all required formatting fields' do
      result = JSON.parse(described_class.parse("puts 'hello'"))
      formatting = result['ast']['formatting']

      expect(formatting).to include(
        'indent_level' => 0,
        'needs_blank_line_before' => false,
        'needs_blank_line_after' => false,
        'preserve_newlines' => false,
        'multiline' => false,
        'original_formatting' => nil
      )
    end
  end
end
