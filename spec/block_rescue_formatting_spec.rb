# frozen_string_literal: true

require 'spec_helper'

RSpec.describe Rfmt, 'Block with Rescue Formatting' do
  describe 'do...end block with rescue' do
    it 'preserves block body and rescue clause' do
      source = <<~RUBY
        foo.each do |x|
          x
        rescue StandardError => e
          e
        end
      RUBY
      result = Rfmt.format(source)
      expect(result).to include('x')
      expect(result).to include('rescue StandardError => e')
    end

    it 'formats block with multiple rescue clauses' do
      source = <<~RUBY
        data.map do |d|
          transform(d)
        rescue TypeError => e
          handle_type_error(e)
        rescue StandardError => e
          handle_standard_error(e)
        end
      RUBY
      result = Rfmt.format(source)
      expect(result).to include('rescue TypeError')
      expect(result).to include('rescue StandardError')
    end

    it 'formats block with rescue and else' do
      source = <<~RUBY
        items.each do |item|
          process(item)
        rescue => e
          handle_error(e)
        else
          success
        end
      RUBY
      result = Rfmt.format(source)
      expect(result).to include('else')
      expect(result).to include('success')
    end

    it 'formats block with rescue, else, and ensure' do
      source = <<~RUBY
        file.each_line do |line|
          parse(line)
        rescue ParseError => e
          log_error(e)
        else
          mark_success
        ensure
          cleanup
        end
      RUBY
      result = Rfmt.format(source)
      expect(result).to include('rescue ParseError')
      expect(result).to include('else')
      expect(result).to include('ensure')
    end

    it 'formats block with only ensure' do
      source = <<~RUBY
        conn.transaction do |tx|
          execute(tx)
        ensure
          tx.close
        end
      RUBY
      result = Rfmt.format(source)
      expect(result).to include('ensure')
      expect(result).to include('tx.close')
    end
  end
end
