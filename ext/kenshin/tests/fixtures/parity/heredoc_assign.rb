sql = <<~SQL
  SELECT *
  FROM users
  WHERE active = true
SQL

body = <<-TEXT
  indented terminator keeps its column
  TEXT

plain = <<HTML
<p>no squiggly</p>
HTML

after = sql.length + body.length
