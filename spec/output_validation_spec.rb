# frozen_string_literal: true

require 'spec_helper'

RSpec.describe Kenshin, '.format output validation' do
  # The invalid-output guard runs inside Rust since the phase-6 switchover,
  # so it cannot be triggered by stubbing from Ruby anymore. The guard itself
  # is covered by the Rust unit tests in ext/kenshin/src/validation.rs; here we
  # pin the public error surface it feeds.
  it 'exposes Kenshin::ValidationError as an Kenshin::Error' do
    expect(Kenshin::ValidationError.ancestors).to include(Kenshin::Error)
  end

  it 'formats valid code normally' do
    result = described_class.format('x = 1')

    expect(result).to include('x = 1')
  end
end
