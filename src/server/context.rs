#[derive(serde::Serialize, Hash, Eq, PartialEq, Debug)]
pub struct Context {
	pub version: Option<String>,
}

impl Default for Context {
	fn default() -> Self {
		Self {
			version: Some(env!("CARGO_PKG_VERSION").to_string()),
		}
	}
}
