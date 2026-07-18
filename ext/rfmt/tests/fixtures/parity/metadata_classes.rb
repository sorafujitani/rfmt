class Foo::Bar < Baz::Qux
  def change
    x = compute(1)
    @count = x
  end
end

class AddUsers < ActiveRecord::Migration[8.1]
  def change
    create_table :users
  end
end

class Simple < Base
end

class Plain
end

module Alpha::Beta
end

module Gamma
end

class ::TopScoped < ::Deep::Nested::Base
end

module Alpha::Beta::Gamma
end
