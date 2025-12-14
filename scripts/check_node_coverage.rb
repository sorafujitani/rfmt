#!/usr/bin/env ruby
# frozen_string_literal: true

# Check rfmt node type coverage against actual Ruby code
#
# This script automatically detects supported node types from Rust source,
# then compares against node types found in scanned Ruby files.
#
# Usage:
#   ruby scripts/check_node_coverage.rb lib/
#   ruby scripts/check_node_coverage.rb /path/to/ruby/project

require 'prism'
require 'set'

def collect_node_types(node, types = Set.new)
  types << node.class.name.split('::').last
  node.child_nodes.compact.each { |child| collect_node_types(child, types) }
  types
end

def scan_directory(dir)
  types = Set.new
  files_scanned = 0
  Dir.glob(File.join(dir, '**/*.rb')).each do |file|
    result = Prism.parse_file(file)
    collect_node_types(result.value, types)
    files_scanned += 1
  rescue StandardError
    # Skip files that can't be parsed
  end
  [types, files_scanned]
end

def parse_supported_nodes(ast_file)
  content = File.read(ast_file)
  if content =~ /pub enum NodeType \{(.+?)Unknown\(String\)/m
    enum_body = Regexp.last_match(1)
    enum_body.scan(/^\s*([A-Z][a-zA-Z]+Node)\s*,?/m).flatten.to_set
  else
    Set.new
  end
end

def parse_explicit_emitters(emitter_file)
  content = File.read(emitter_file)
  if content =~ /fn emit_node\(.+?match &node\.node_type \{(.+?)_ =>/m
    match_body = Regexp.last_match(1)
    match_body.scan(/NodeType::([A-Z][a-zA-Z]+Node)/).flatten.to_set
  else
    Set.new
  end
end

def parse_structural_nodes(emitter_file)
  content = File.read(emitter_file)
  if content =~ /fn is_structural_node\(.+?matches!\s*\(\s*node_type\s*,(.+?)\)/m
    match_body = Regexp.last_match(1)
    match_body.scan(/NodeType::([A-Z][a-zA-Z]+Node)/).flatten.to_set
  else
    Set.new
  end
end

def find_project_root
  dir = Dir.pwd
  dir = File.dirname(dir) until File.exist?(File.join(dir, 'Cargo.toml')) || dir == '/'
  dir
end

if ARGV.empty?
  warn "Usage: #{$PROGRAM_NAME} <directory>"
  exit 1
end

dir = ARGV[0]
project_root = find_project_root

ast_file = File.join(project_root, 'ext/rfmt/src/ast/mod.rs')
emitter_file = File.join(project_root, 'ext/rfmt/src/emitter/mod.rs')

unless File.exist?(ast_file) && File.exist?(emitter_file)
  warn 'Error: Cannot find rfmt source files. Run from project root.'
  exit 1
end

known_nodes = parse_supported_nodes(ast_file)
explicit_emitters = parse_explicit_emitters(emitter_file)
structural_nodes = parse_structural_nodes(emitter_file)

used_types, files_scanned = scan_directory(dir)

puts '=== rfmt Node Type Coverage Report ==='
puts
puts "Scanned: #{dir} (#{files_scanned} files)"
puts "Total unique node types found: #{used_types.size}"
puts

explicitly_supported = used_types & explicit_emitters
structural = used_types & structural_nodes
known_fallback = used_types & (known_nodes - explicit_emitters - structural_nodes)
unknown_fallback = used_types - known_nodes

puts "Explicitly supported - has dedicated emitter (#{explicitly_supported.size}):"
explicitly_supported.sort.each { |t| puts "  ✓ #{t}" }
puts

puts "Structural - handled by parent (#{structural.size}):"
structural.sort.each { |t| puts "  • #{t}" }
puts

puts "Known but using emit_generic (#{known_fallback.size}):"
known_fallback.sort.each { |t| puts "  △ #{t}" }
puts

puts "Unknown - not in NodeType enum (#{unknown_fallback.size}):"
unknown_fallback.sort.each { |t| puts "  ⚠ #{t}" }

if unknown_fallback.any?
  puts
  puts 'Warning: Unknown nodes should be added to NodeType enum in ast/mod.rs'
end

exit unknown_fallback.empty? ? 0 : 1
