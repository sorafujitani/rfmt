# frozen_string_literal: true

require 'spec_helper'

RSpec.describe 'Prism to Rust integration' do
  describe 'end-to-end parsing' do
    it 'parses simple Ruby code through the full pipeline' do
      source = "puts 'hello'"

      # Step 1: Ruby Prism parsing
      prism_json = Rfmt::PrismBridge.parse(source)
      expect(prism_json).to be_a(String)

      # Verify JSON structure
      parsed = JSON.parse(prism_json)
      expect(parsed['ast']['node_type']).to eq('program_node')
      expect(parsed).to have_key('ast')
      expect(parsed).to have_key('comments')
      expect(parsed['ast']).to have_key('location')
      expect(parsed['ast']).to have_key('children')
    end

    it 'handles class definitions' do
      source = <<~RUBY
        class User
          def name
            @name
          end
        end
      RUBY

      prism_json = Rfmt::PrismBridge.parse(source)
      parsed = JSON.parse(prism_json)

      expect(parsed['ast']['node_type']).to eq('program_node')

      class_node = parsed['ast']['children'].first
      expect(class_node['node_type']).to eq('class_node')
      expect(class_node['metadata']['name']).to eq('User')
    end

    it 'preserves location information' do
      source = <<~RUBY
        class Foo
          def bar
            42
          end
        end
      RUBY

      prism_json = Rfmt::PrismBridge.parse(source)
      parsed = JSON.parse(prism_json)

      class_node = parsed['ast']['children'].first
      expect(class_node['location']['start_line']).to eq(1)
      expect(class_node['location']['end_line']).to be > 1
      expect(class_node['formatting']['multiline']).to be true
    end

    it 'handles method definitions with parameters' do
      source = 'def hello(name, age) end'

      prism_json = Rfmt::PrismBridge.parse(source)
      parsed = JSON.parse(prism_json)

      def_node = parsed['ast']['children'].first
      expect(def_node['node_type']).to eq('def_node')
      expect(def_node['metadata']['name']).to eq('hello')
      # Parameters are in children
      expect(def_node['children'].length).to be > 0
    end

    it 'handles various literal types' do
      literals = {
        '"string"' => 'string_node',
        '42' => 'integer_node',
        '3.14' => 'float_node',
        '[1, 2, 3]' => 'array_node',
        '{ a: 1 }' => 'hash_node'
      }

      literals.each do |source, expected_type|
        prism_json = Rfmt::PrismBridge.parse(source)
        parsed = JSON.parse(prism_json)

        node = parsed['ast']['children'].first
        expect(node['node_type']).to eq(expected_type), "Expected #{source} to produce #{expected_type}"
      end
    end
  end
end
