expected = :ready

case payload
in [Integer => first, *rest]
  first + rest.sum
in { status: String => status, code: 200 | 201 }
  status
in ^expected
  :pinned
else
  :unmatched
end

begin
  risky!
rescue KeyError => e
  recover(e)
rescue StandardError
  retry_count += 1
  retry if retry_count < 3
else
  celebrate
ensure
  cleanup
end

counters = Hash.new(0)
counters[:hits] ||= 0
counters[:hits] += 1
@cache &&= @cache.compact
$verbose ||= false
CONFIG = { a: 3r, b: 3.14r, c: 2i, d: 2ri }

class << self
  def singleton_helper = :ok
end

alias aka singleton_helper
alias $err $stderr
undef :aka

flip = items.select { |l| (l == first)..(l == last) }

lines = ->(input) { input.each_line.to_a }
x, *middle, z = lines.call(<<~TXT)
  alpha
  beta
  gamma
TXT
puts x, middle, z
