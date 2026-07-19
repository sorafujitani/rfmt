# frozen_string_literal: true

require 'spec_helper'
require 'ruby_lsp/kenshin/addon'

RSpec.describe RubyLsp::Kenshin::Addon do
  let(:addon) { described_class.new }

  describe '#name' do
    it "returns 'kenshin'" do
      expect(addon.name).to eq('kenshin')
    end
  end

  describe '#activate' do
    it 'registers kenshin formatter' do
      global_state = double('GlobalState')
      message_queue = []

      expect(global_state).to receive(:register_formatter)
        .with('kenshin', kind_of(RubyLsp::Kenshin::FormatterRunner))

      addon.activate(global_state, message_queue)
    end
  end

  describe '#deactivate' do
    it 'does not raise error' do
      expect { addon.deactivate }.not_to raise_error
    end
  end
end
