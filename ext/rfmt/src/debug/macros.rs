/// デバッグ用マクロ
///
/// AST構造の詳細な出力を行うマクロ
/// TRACE レベルでログが有効な場合のみ実行される
#[macro_export]
macro_rules! debug_ast {
    ($ast:expr) => {
        if log::log_enabled!(log::Level::Trace) {
            log::trace!("AST structure:\n{:#?}", $ast);
        }
    };
    ($ast:expr, $msg:expr) => {
        if log::log_enabled!(log::Level::Trace) {
            log::trace!("{}: AST structure:\n{:#?}", $msg, $ast);
        }
    };
}

/// ノード情報をデバッグ出力するマクロ
/// DEBUG レベルでログが有効な場合のみ実行される
#[macro_export]
macro_rules! debug_node {
    ($node:expr) => {
        if log::log_enabled!(log::Level::Debug) {
            log::debug!(
                "Node: type={:?}, location={:?}",
                $node.node_type,
                $node.location
            );
        }
    };
    ($node:expr, $msg:expr) => {
        if log::log_enabled!(log::Level::Debug) {
            log::debug!(
                "{}: Node type={:?}, location={:?}",
                $msg,
                $node.node_type,
                $node.location
            );
        }
    };
}

/// フォーマット処理の開始を記録するマクロ
#[macro_export]
macro_rules! debug_format_start {
    ($msg:expr) => {
        if log::log_enabled!(log::Level::Debug) {
            log::debug!("Format start: {}", $msg);
        }
    };
}

/// フォーマット処理の完了を記録するマクロ
#[macro_export]
macro_rules! debug_format_end {
    ($msg:expr) => {
        if log::log_enabled!(log::Level::Debug) {
            log::debug!("Format end: {}", $msg);
        }
    };
}

/// パフォーマンス測定用マクロ
#[macro_export]
macro_rules! debug_time {
    ($label:expr, $block:expr) => {{
        let start = std::time::Instant::now();
        let result = $block;
        let elapsed = start.elapsed();
        if log::log_enabled!(log::Level::Debug) {
            log::debug!("{} took {:?}", $label, elapsed);
        }
        result
    }};
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_macros_compile() {
        // マクロが正しくコンパイルされることを確認
        let test_value = "test";
        debug_ast!(test_value);
        debug_ast!(test_value, "with message");

        struct TestNode {
            node_type: String,
            location: String,
        }

        let test_node = TestNode {
            node_type: "test".to_string(),
            location: "1:1".to_string(),
        };

        debug_node!(&test_node);
        debug_node!(&test_node, "with message");

        debug_format_start!("test");
        debug_format_end!("test");

        let _result = debug_time!("test operation", {
            let x = 1 + 1;
            x
        });
    }
}
