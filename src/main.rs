#[macro_use]
extern crate rocket;
extern crate rocket_contrib;

use std::path::Path;

use rocket_contrib::serve::StaticFiles;

/// Returns a fully-assembled Rocket, ready for ignition.
fn server() -> rocket::Rocket {
	let static_dir: &Path = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/public"));

	rocket::Rocket::ignite()
		.mount("/", StaticFiles::from(static_dir))
}

fn main() {
	println!("Hello world!");

	server().launch();
}
