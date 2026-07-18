def !@
  true
end

def +@
  self
end

def []=(key, value)
  store(key, value)
end

def self.build(name)
  new(name)
end

obj = Object.new

def obj.run
  1
end

def bare arg
  arg
end

def none
  0
end

def empty_parens()
  nil
end
