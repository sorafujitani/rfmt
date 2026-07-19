# frozen_string_literal: true

require 'pathname'

require_relative 'uri'

module Kenshin
  module LSP
    class Workspace
      def initialize
        @roots = []
      end

      def configure(params)
        @roots = workspace_folder_roots(params)
        root_uri = params['rootUri']
        @roots << URI.file_uri_to_path(root_uri) if root_uri
        @roots = @roots.compact.map { |root| File.expand_path(root) }.uniq
      end

      def root_for(uri)
        path = URI.file_uri_to_path(uri)
        return existing_root(@roots.first) unless path

        matching_root = @roots
                        .select { |root| path_inside?(path, root) }
                        .max_by(&:length)
        return existing_root(matching_root) if matching_root

        existing_root(File.dirname(path))
      end

      def with_root_for(uri, &block)
        root = root_for(uri)
        return block.call unless root

        Dir.chdir(root, &block)
      end

      private

      def workspace_folder_roots(params)
        Array(params['workspaceFolders']).filter_map do |folder|
          URI.file_uri_to_path(folder['uri'])
        end
      end

      def path_inside?(path, root)
        relative = Pathname.new(path).relative_path_from(Pathname.new(root)).to_s
        relative == '.' || !relative.start_with?('..')
      rescue ArgumentError
        false
      end

      def existing_root(root)
        return nil unless root && File.directory?(root)

        root
      end
    end
  end
end
