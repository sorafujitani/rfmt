# frozen_string_literal: true

require 'rfmt'

# Save original directory to ensure we always return to it
ORIGINAL_TEST_DIR = Dir.pwd

RSpec.configure do |config|
  # Enable flags like --only-failures and --next-failure
  # Use absolute path to avoid issues when tests change directories
  config.example_status_persistence_file_path = File.expand_path('../.rspec_status', __dir__)

  # Disable RSpec exposing methods globally on `Module` and `main`
  config.disable_monkey_patching!

  config.expect_with :rspec do |c|
    c.syntax = :expect
  end

  # Ensure we're in the correct directory after all tests complete
  config.after(:suite) do
    Dir.chdir(ORIGINAL_TEST_DIR)
  rescue StandardError
    nil
  end
end
