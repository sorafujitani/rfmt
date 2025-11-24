# frozen_string_literal: true
class User < ApplicationRecord
  has_many :posts
  has_many :comments
  validates :email, presence: true, uniqueness: true
  validates :name, presence: true
  def full_name
    "#{first_name} #{last_name}"
  end
  def admin?
    role == 'admin'
  end
  scope :active, -> { where(active: true) }
  scope :recent, -> { order(created_at: :desc).limit(10) }
end