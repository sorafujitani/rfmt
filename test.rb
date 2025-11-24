# frozen_string_literal: true

require 'rfmt'

puts "rfmt version: #{Rfmt::VERSION}"

code = <<~RUBY
  class MyClass
  def hello
  puts "Hello, World!"
  end
  end
RUBY

formatted = Rfmt.format(code)
puts 'Formatted code:'
puts formatted
