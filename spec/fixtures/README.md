# Test Fixtures

This directory contains test fixtures for snapshot testing real-world Ruby code.

## Directory Structure

```
fixtures/
├── input/       # Unformatted Ruby files
├── expected/    # Expected formatted output
└── README.md    # This file
```

## Usage

### Running Snapshot Tests

```bash
# Run all snapshot tests
bundle exec rspec spec/snapshot_spec.rb

# Run with diff output on failures
SHOW_DIFF=1 bundle exec rspec spec/snapshot_spec.rb

# Run with verbose mode (always show diffs)
VERBOSE=1 bundle exec rspec spec/snapshot_spec.rb
```

### Adding New Test Cases

1. **Add input file** in `input/`:
   ```bash
   # Create new unformatted Ruby file
   cat > input/my_feature.rb <<'EOF'
   class MyFeature
   def my_method
   puts "hello"
   end
   end
   EOF
   ```

2. **Generate expected output**:
   ```bash
   # Format and save expected output
   bundle exec ruby -I lib -r kenshin -e \
     "print Kenshin.format(File.read('input/my_feature.rb'))" \
     > expected/my_feature.rb
   ```

3. **Run tests**:
   ```bash
   bundle exec rspec spec/snapshot_spec.rb
   ```

### Updating Expected Output

When the formatter behavior changes intentionally, update all expected outputs:

```bash
# Update all expected files
for file in input/*.rb; do
  basename=$(basename "$file")
  bundle exec ruby -I lib -r kenshin -e \
    "print Kenshin.format(File.read('$file'))" \
    > "expected/$basename"
done

# Verify changes
git diff expected/
```

## Test Categories

### Rails Model (`rails_model.rb`)
Tests formatting of:
- ActiveRecord model
- Associations (`has_many`)
- Validations
- Instance methods
- Scopes

### Rails Controller (`rails_controller.rb`)
Tests formatting of:
- Controller class
- Before actions
- CRUD actions
- Conditionals (if/else)
- Private methods
- Strong parameters

### Rails Service (`rails_service.rb`)
Tests formatting of:
- Module nesting
- Service object pattern
- Initializer
- Public methods
- Private methods
- Conditional logic

## Current Limitations

The current implementation (Phase 2) has these known limitations:

1. **If/Else indentation**: Contents of if/else blocks are extracted from source without reformatting
2. **Block syntax**: Lambda and block syntax preserved as-is
3. **Hash/Array formatting**: Not yet formatted, preserved from source
4. **Comments**: Not yet fully preserved
5. **String interpolation**: Preserved from source

These will be addressed in Phase 3.

## Test Structure

Each snapshot test includes:

1. **Formatting test**: Compares formatted output with expected
2. **Idempotency test**: Ensures formatting twice produces same result
3. **Structure preservation test**: Verifies AST structure is maintained

## Debugging Failed Tests

When a test fails:

```bash
# Show detailed diff
SHOW_DIFF=1 bundle exec rspec spec/snapshot_spec.rb

# Or manually compare
bundle exec ruby -I lib -r kenshin -e \
  "puts Kenshin.format(File.read('input/rails_model.rb'))" \
  | diff - expected/rails_model.rb
```

## Contributing

When adding new test cases:

1. Choose realistic, production-like code examples
2. Cover different Ruby/Rails patterns
3. Include edge cases
4. Keep files focused (one pattern per file)
5. Name files descriptively (e.g., `rails_concern.rb`, `rails_mailer.rb`)

## Future Test Cases

Suggested additions:
- `rails_concern.rb` - Module with ActiveSupport::Concern
- `rails_mailer.rb` - ActionMailer class
- `rails_job.rb` - ActiveJob class
- `rails_spec.rb` - RSpec test file
- `rails_routes.rb` - Routes DSL
- `plain_ruby_class.rb` - Non-Rails Ruby class
- `metaprogramming.rb` - define_method, class_eval, etc.
