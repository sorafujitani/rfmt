# frozen_string_literal: true

require 'spec_helper'

RSpec.describe Rfmt, '.format output validation' do
  # The invalid-output guard runs inside Rust since the phase-6 switchover,
  # so it cannot be triggered by stubbing from Ruby anymore. The guard itself
  # is covered by the Rust unit tests in ext/rfmt/src/validation.rs; here we
  # pin the public error surface it feeds.
  it 'exposes Rfmt::ValidationError as an Rfmt::Error' do
    expect(Rfmt::ValidationError.ancestors).to include(Rfmt::Error)
  end

  it 'formats valid code normally' do
    result = described_class.format('x = 1')

    expect(result).to include('x = 1')
  end
end
