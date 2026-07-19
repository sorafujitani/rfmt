class Greeter
  attr_reader :name

  def initialize(name)
    @name = name
  end

  def greet(loud: false)
    message = "hello, #{name}"
    if loud
      message.upcase
    else
      message
    end
  end
end

module Registry
  DEFAULT = Greeter.new("world")

  def self.lookup(key)
    entries.fetch(key) { DEFAULT }
  end
end

items = [1, 2.5, :three, nil, true]
totals = { "sum" => items.compact.size, count: items.length }
items.each_with_index do |item, index|
  puts "#{index}: #{item}" unless item.nil?
end
result = Registry.lookup(:default)&.greet(loud: totals[:count] > 3)
