result = execute(<<~SQL, timeout: 5)
  SELECT id
  FROM events
SQL

logger.info(prefix, <<~MSG)
  first line
  second line
MSG

wrap(inner(<<~ONE), <<~TWO)
  one body
ONE
  two body
TWO

puts result
