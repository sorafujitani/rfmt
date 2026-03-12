# rfmt - Ruby Formatter

## Project Overview
A Ruby code formatter powered by a Rust native extension.

## Development Commands
```bash
# Run tests
bundle exec rspec

# Run the formatter
bundle exec rfmt <file>

# Build the Rust extension
bundle exec rake compile
```

## Claude Code Settings

### Plan File Location
Plan files created in plan mode should be saved to:
- **Preferred**: `.claude/plans/` (under the project root)
- Use the project-local `.claude/plans/`, not the global `~/.claude/plans/`

### Language
- Respond in Japanese
