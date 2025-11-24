# frozen_string_literal: true
module Users
  class RegistrationService
    def initialize(params)
      @params = params
    end
    def call
      user = User.new(@params)
      if user.save
        send_welcome_email(user)
        create_default_settings(user)
        { success: true, user: user }
      else
        { success: false, errors: user.errors }
      end
    end
    private
    def send_welcome_email(user)
      UserMailer.welcome(user).deliver_later
    end
    def create_default_settings(user)
      user.create_setting(theme: 'light', notifications: true)
    end
  end
end