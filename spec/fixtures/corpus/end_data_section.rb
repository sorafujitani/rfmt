# frozen_string_literal: true

# Corpus sentinel: exercises the data-section preservation property on every
# corpus check run (the __END__ section lives outside the AST).
puts DATA.read

__END__
plain data line
  indented line	with a tab

trailing blank line above, no code below
