# frozen_string_literal: true

require 'spec_helper'

# Exploratory coverage: Rails-heavy Ruby idioms (valid Ruby; no Rails gem required).
# Each case must round-trip to parseable Ruby and must not leak formatter internals.
RSpec.describe Rfmt, 'Rails idioms (exploratory)' do
  def assert_well_formed(source, label = nil)
    result = Rfmt.format(source)
    expect(result).not_to include('Prism::'), label || 'no raw AST class names in output'
    errors = Prism.parse(result).errors
    expect(errors).to be_empty, lambda {
      msg = (label ? "[#{label}] " : '').to_s
      "#{msg}Prism errors after format: #{errors.map(&:message).join('; ')}"
    }
    result
  end

  it 'preserves versioned ActiveRecord::Migration superclass' do
    source = <<~RUBY
      class CreateWidgets < ActiveRecord::Migration[7.2]
        def change
          create_table :widgets do |t|
            t.string :name, null: false
            t.timestamps
          end
        end
      end
    RUBY
    out = assert_well_formed(source, 'migration')
    expect(out).to include('class CreateWidgets < ActiveRecord::Migration[7.2]')
    expect(out).to include('create_table :widgets')
  end

  it 'formats ActiveRecord model with associations and validations' do
    source = <<~RUBY
      class Post < ApplicationRecord
        belongs_to :user, inverse_of: :posts, optional: true
        has_many :comments, dependent: :destroy
        has_one :draft, class_name: 'PostVersion', foreign_key: :post_id
        validates :title, presence: true, length: { maximum: 255 }
        validates :slug, uniqueness: { scope: :user_id }
        enum :status, { draft: 0, published: 1, archived: 2 }
        scope :recent, -> { order(created_at: :desc) }
        delegate :email, to: :user, prefix: true, allow_nil: true
      end
    RUBY
    assert_well_formed(source, 'ar model')
  end

  it 'formats ApplicationController-style filters and strong params' do
    source = <<~RUBY
      class PostsController < ApplicationController
        before_action :set_post, only: %i[show edit update destroy]
        skip_before_action :verify_authenticity_token, if: :api_request?

        def create
          @post = Post.new(post_params)
          respond_to do |format|
            format.html { redirect_to @post }
            format.json { render json: @post, status: :created, location: @post }
          end
        end

        private

        def post_params
          params.require(:post).permit(:title, :body, tag_ids: [])
        end
      end
    RUBY
    assert_well_formed(source, 'controller')
  end

  it 'formats routes.rb-style DSL' do
    source = <<~RUBY
      Rails.application.routes.draw do
        root to: 'home#index'
        resources :posts do
          resources :comments, shallow: true
        end
        namespace :admin do
          resources :users, only: %i[index show]
        end
        get 'up' => 'rails/health#show', as: :rails_health_check
      end
    RUBY
    assert_well_formed(source, 'routes')
  end

  it 'formats ActiveJob perform and ApplicationMailer' do
    source = <<~RUBY
      class NotifyUserJob < ApplicationJob
        queue_as :mailers

        def perform(user_id, template:)
          user = User.find(user_id)
          UserMailer.with(user: user).notify(template).deliver_later
        end
      end

      class UserMailer < ApplicationMailer
        default from: 'app@example.com'

        def notify(template)
          @user = params[:user]
          mail to: @user.email, subject: template
        end
      end
    RUBY
    assert_well_formed(source, 'job mailer')
  end

  it 'formats ActiveSupport::Concern and included block' do
    source = <<~RUBY
      module Trackable
        extend ActiveSupport::Concern

        included do
          before_save :touch_tracked_at
        end

        class_methods do
          def tracked
            where.not(tracked_at: nil)
          end
        end
      end
    RUBY
    assert_well_formed(source, 'concern')
  end

  it 'formats initializers and callbacks with blocks' do
    source = <<~RUBY
      ActiveSupport.on_load(:active_record) do
        self.default_timezone = :utc
      end

      Rails.application.config.after_initialize do
        Sidekiq.configure_server { |c| c.redis = { url: ENV.fetch('REDIS_URL') } }
      end
    RUBY
    assert_well_formed(source, 'initializers')
  end

  it 'formats STI and abstract model patterns' do
    source = <<~RUBY
      class Animal < ApplicationRecord
        self.abstract_class = true
      end

      class Dog < Animal
        self.table_name = 'creatures'
      end
    RUBY
    assert_well_formed(source, 'sti')
  end

  it 'formats serializer-ish hash and JSON column defaults' do
    source = <<~RUBY
      class Setting < ApplicationRecord
        store_accessor :metadata, :theme, :locale
        attribute :config, default: -> { {} }
      end
    RUBY
    assert_well_formed(source, 'store')
  end
end
