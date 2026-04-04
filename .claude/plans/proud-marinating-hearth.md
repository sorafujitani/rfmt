# PBT 探索的パターン管理の実装プラン

## Context

PBT は **未知のバグを探索するための道具** であり、既知ケースの防御は E2E テストが担う。
現状の `spec/pbt_rails_format_spec.rb`(632行)は単一ファイルで全パターンが混在し、
「どの領域を探索済みか」「発見した問題は何か」が管理できない。

**目的**: パターン空間をグループ化して探索を計画的に進められるようにし、
発見したバグを E2E テストへスムーズに引き渡せる仕組みを作る。

## 設計思想

```
探索 PBT の役割:
  ┌────────────┐     発見     ┌────────────┐     移管     ┌────────────┐
  │ PBT 探索   │ ──────────▶ │ discoveries/ │ ──────────▶ │ E2E テスト │
  │ (ランダム) │             │ (一時保存)   │             │ (永続防御) │
  └────────────┘             └────────────┘             └────────────┘
       ↑
  カバレッジマップで未探索領域を特定
```

- PBT で見つかったバグ → `discoveries/` に一時保存 → E2E テスト作成後に削除
- PBT 自体にはデグレーションテストを持たない
- カバレッジマップで探索の偏りを可視化

## ファイル構成

```
spec/
├── support/
│   └── pbt/
│       ├── catalog.rb              # パターンカタログ (登録 + カバレッジマップ)
│       ├── primitives.rb           # 共通プリミティブジェネレータ
│       ├── properties.rb           # 4プロパティの検証ロジック
│       ├── discovery_reporter.rb   # バグ発見時のファイル保存 + コンソール出力
│       └── generators/
│           ├── controller.rb
│           ├── model.rb
│           ├── migration.rb
│           ├── concern.rb
│           ├── job.rb
│           ├── mailer.rb
│           ├── serializer.rb
│           ├── route.rb
│           ├── service.rb
│           └── config.rb
├── fixtures/
│   └── pbt/
│       ├── coverage.yml            # カバレッジマップ (探索履歴)
│       └── discoveries/            # 発見されたバグの一時保存 (E2E移管後に削除)
│           └── .gitkeep
├── pbt_rails_format_spec.rb        # メインテスト (カタログから動的生成)
└── spec_helper.rb                  # support/ 読み込み追加
```

## Step 1: パターンカタログ + カバレッジマップ

**ファイル**: `spec/support/pbt/catalog.rb`

パターンの登録と探索履歴の追跡を担う中心モジュール。

```ruby
module PBT
  PatternEntry = Data.define(:name, :group, :tags, :generator)

  module Catalog
    @entries = []

    # パターン登録
    def self.register(name:, group:, tags: [], &generator)
      @entries << PatternEntry.new(name:, group:, tags:, generator:)
    end

    # 検索
    def self.all = @entries
    def self.by_group(group) = @entries.select { |e| e.group == group }
    def self.by_tag(tag) = @entries.select { |e| e.tags.include?(tag) }
    def self.groups = @entries.map(&:group).uniq.sort

    # カバレッジマップ更新 (テスト実行後に呼ばれる)
    def self.record_run(group:, runs:, failures:)
      coverage = load_coverage
      key = group.to_s
      coverage[key] ||= { 'total_runs' => 0, 'total_failures' => 0, 'sessions' => [] }
      coverage[key]['total_runs'] += runs
      coverage[key]['total_failures'] += failures
      coverage[key]['sessions'] << {
        'date' => Time.now.strftime('%Y-%m-%d %H:%M'),
        'runs' => runs,
        'failures' => failures
      }
      save_coverage(coverage)
    end

    # カバレッジレポート出力
    def self.coverage_report
      coverage = load_coverage
      groups.each do |g|
        data = coverage[g.to_s]
        if data
          puts "  #{g}: #{data['total_runs']} runs, #{data['total_failures']} failures, last: #{data['sessions'].last&.dig('date') || 'never'}"
        else
          puts "  #{g}: ** UNEXPLORED **"
        end
      end
    end
  end
end
```

**カバレッジマップ (`coverage.yml`)** の例:
```yaml
controller:
  total_runs: 1500
  total_failures: 0
  sessions:
    - { date: "2026-02-23 14:30", runs: 500, failures: 0 }
    - { date: "2026-02-24 10:00", runs: 1000, failures: 0 }
model:
  total_runs: 500
  total_failures: 1
  sessions:
    - { date: "2026-02-23 14:30", runs: 500, failures: 1 }
```

## Step 2: ジェネレータ分割

**ファイル**: `spec/support/pbt/primitives.rb` + `spec/support/pbt/generators/*.rb`

現在の `pbt_rails_format_spec.rb` からジェネレータを抽出。

`primitives.rb`: 共通メソッド (`gen_class_name`, `gen_method_name` 等) を `PBT::Primitives` モジュールに集約。

各 `generators/*.rb`: カテゴリ別にクラスを定義し、Catalog に登録。

```ruby
# spec/support/pbt/generators/controller.rb
module PBT
  module Generators
    class Controller
      include PBT::Primitives

      def generate
        # 現在の gen_controller の内容
      end
    end
  end
end

PBT::Catalog.register(name: :controller, group: :controller, tags: [:rails_dsl]) {
  PBT::Generators::Controller.new.generate
}
```

**tags の設計** (パターンの性質を示すラベル):
- `:rails_dsl` — Rails DSL を使うパターン (controller, model, concern, job, mailer)
- `:data_def` — データ定義系 (migration, serializer)
- `:routing` — ルーティング系 (route)
- `:plain_ruby` — 純粋な Ruby コード (service, config)
- `:edge_case` — エッジケース (将来追加: deep_nesting, long_chain, heredoc 等)

## Step 3: プロパティ検証モジュール

**ファイル**: `spec/support/pbt/properties.rb`

4プロパティのチェックロジックを再利用可能なモジュールに。

```ruby
module PBT
  module Properties
    module_function

    def check_idempotent(source)
      once = Rfmt.format(source)
      twice = Rfmt.format(once)
      { pass: once == twice, once: once, twice: twice }
    end

    def check_syntax_preservation(source)
      formatted = Rfmt.format(source)
      errors = Prism.parse(formatted).errors
      { pass: errors.empty?, formatted: formatted, errors: errors }
    end

    def check_no_crash(source)
      Rfmt.format(source)
      { pass: true }
    rescue Rfmt::Error
      { pass: true }  # 構文エラーは許容
    rescue => e
      { pass: false, error: e }
    end

    def check_content_preservation(source, identifiers)
      formatted = Rfmt.format(source)
      missing = identifiers.reject { |id| formatted.include?(id) }
      { pass: missing.empty?, formatted: formatted, missing: missing }
    end
  end
end
```

## Step 4: DiscoveryReporter

**ファイル**: `spec/support/pbt/discovery_reporter.rb`

バグ発見時にコンソール詳細出力 + ファイル自動保存を行う。

```ruby
module PBT
  module DiscoveryReporter
    DISCOVERIES_DIR = File.expand_path('../../fixtures/pbt/discoveries', __dir__)

    module_function

    def report(group:, property:, source:, detail:)
      timestamp = Time.now.strftime('%Y%m%d_%H%M%S')
      id = "#{group}_#{property}_#{timestamp}"
      filepath = File.join(DISCOVERIES_DIR, "#{id}.yml")

      data = {
        'id' => id,
        'group' => group.to_s,
        'property' => property.to_s,
        'discovered' => Time.now.iso8601,
        'migrated_to_e2e' => false,
        'source' => source,
        'detail' => detail
      }

      File.write(filepath, data.to_yaml)

      # コンソール出力
      warn "\n#{'=' * 60}"
      warn "PBT DISCOVERY: #{id}"
      warn "Group: #{group}, Property: #{property}"
      warn "Source:\n#{source}"
      warn "Detail: #{detail}"
      warn "Saved to: #{filepath}"
      warn "#{'=' * 60}\n"
    end
  end
end
```

## Step 5: メインテストファイルのリファクタリング

**ファイル**: `spec/pbt_rails_format_spec.rb`

Catalog から動的にテスト生成。RSpec タグでフィルタリング可能。
テスト完了後にカバレッジマップを自動更新。

```ruby
require 'spec_helper'
require 'prism'

NUM_RUNS = Integer(ENV.fetch('RAILS_FMT_PBT_RUNS', 100))

RSpec.describe 'PBT: Rails formatting' do
  PBT::Catalog.groups.each do |group|
    entries = PBT::Catalog.by_group(group)
    group_tags = entries.flat_map(&:tags).uniq.map { |t| [t, true] }.to_h

    describe group.to_s, **group_tags, group => true do
      entries.each do |entry|
        failure_count = 0

        it "FP1: idempotent (#{entry.name})" do
          NUM_RUNS.times { |i|
            source = entry.generator.call
            next if Prism.parse(source).errors.any?
            result = PBT::Properties.check_idempotent(source)
            unless result[:pass]
              failure_count += 1
              PBT::DiscoveryReporter.report(
                group: group, property: :idempotent,
                source: source, detail: PBT::Properties.diff(result[:once], result[:twice])
              )
            end
            expect(result[:pass]).to be(true), "Idempotency failed (run #{i+1})"
          }
        end

        # FP2, FP3, FP4 も同様のパターン ...

        after(:all) do
          PBT::Catalog.record_run(group: group, runs: NUM_RUNS, failures: failure_count)
        end
      end
    end
  end
end
```

## Step 6: spec_helper.rb の更新

`spec/support/pbt/` の自動読み込みを追加:

```ruby
Dir[File.join(__dir__, 'support', 'pbt', '**', '*.rb')].sort.each { |f| require f }
```

## 実行コマンド

```bash
# 全探索
bundle exec rspec spec/pbt_rails_format_spec.rb

# カテゴリ指定
bundle exec rspec spec/pbt_rails_format_spec.rb --tag controller
bundle exec rspec spec/pbt_rails_format_spec.rb --tag model

# タグ指定 (性質ベース)
bundle exec rspec spec/pbt_rails_format_spec.rb --tag rails_dsl
bundle exec rspec spec/pbt_rails_format_spec.rb --tag edge_case

# 回数指定
RAILS_FMT_PBT_RUNS=500 bundle exec rspec spec/pbt_rails_format_spec.rb --tag controller

# カバレッジマップ表示
bundle exec ruby -e "require_relative 'spec/support/pbt/catalog'; PBT::Catalog.coverage_report"
```

## 実装順序

1. `spec/support/pbt/catalog.rb` — カタログ + カバレッジマップ
2. `spec/support/pbt/primitives.rb` — 共通ジェネレータ抽出
3. `spec/support/pbt/properties.rb` — プロパティ検証モジュール
4. `spec/support/pbt/discovery_reporter.rb` — 発見レポーター
5. `spec/support/pbt/generators/*.rb` — 10カテゴリのジェネレータ分割・登録
6. `spec/fixtures/pbt/coverage.yml` — 空の初期カバレッジ
7. `spec/fixtures/pbt/discoveries/.gitkeep` — 発見ディレクトリ
8. `spec/pbt_rails_format_spec.rb` — カタログベースに書き換え
9. `spec/spec_helper.rb` — support/ 読み込み追加
10. 全テスト実行で動作確認

## 検証方法

```bash
# 1. 全 PBT テストが PASS すること
RAILS_FMT_PBT_RUNS=100 bundle exec rspec spec/pbt_rails_format_spec.rb --format documentation

# 2. タグフィルタリングが動作すること
bundle exec rspec spec/pbt_rails_format_spec.rb --tag controller --format documentation
bundle exec rspec spec/pbt_rails_format_spec.rb --tag rails_dsl --format documentation

# 3. カバレッジマップが更新されること
cat spec/fixtures/pbt/coverage.yml

# 4. 既存テストが壊れていないこと
bundle exec rspec spec/ --exclude-pattern 'spec/pbt_*'
```
