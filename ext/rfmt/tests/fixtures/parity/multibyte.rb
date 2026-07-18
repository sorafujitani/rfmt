greeting = "こんにちは、世界"
farewell = 'さようなら'; short = "あ"

def label(polite: false)
  polite ? "です" : "だ"
end

mixed = "emoji 🎉 と 日本語"
message = <<~TEXT
  一行目の本文
  二行目の本文
TEXT

combined = "#{greeting} #{mixed}".freeze
