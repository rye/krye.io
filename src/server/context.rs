#[derive(serde::Serialize, Hash, Eq, PartialEq, Debug)]
pub struct Context {
	pub version: Option<String>,
}
