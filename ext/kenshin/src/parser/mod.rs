use crate::ast::Node;
use crate::error::Result;

pub mod native_adapter;
pub mod prism_adapter;
pub use native_adapter::NativeAdapter;
pub use prism_adapter::PrismAdapter;

pub trait RubyParser: Send + Sync {
    fn parse(&self, source: &str) -> Result<Node>;
}
