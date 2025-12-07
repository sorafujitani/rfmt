# frozen_string_literal: true

require "spec_helper"
require "ruby_lsp/rfmt/addon"

RSpec.describe RubyLsp::Rfmt::Addon do
  let(:addon) { described_class.new }

  describe "#name" do
    it "returns 'rfmt'" do
      expect(addon.name).to eq("rfmt")
    end
  end

  describe "#activate" do
    it "registers rfmt formatter" do
      global_state = double("GlobalState")
      message_queue = []

      expect(global_state).to receive(:register_formatter)
        .with("rfmt", kind_of(RubyLsp::Rfmt::FormatterRunner))

      addon.activate(global_state, message_queue)
    end
  end

  describe "#deactivate" do
    it "does not raise error" do
      expect { addon.deactivate }.not_to raise_error
    end
  end
end
