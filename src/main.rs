#![feature(decl_macro, proc_macro_hygiene)]

extern crate sentry;

#[macro_use]
extern crate rocket;
extern crate rocket_contrib;

use std::fs::File;
use std::path::Path;

use rocket_contrib::serve::StaticFiles;

/// Returns a fully-assembled Rocket, ready for ignition.
fn server() -> rocket::Rocket {
	let static_dir: &Path = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/public"));

	rocket::Rocket::ignite()
		.mount("/", StaticFiles::from(static_dir))
		.mount("/", routes![index, resume])
}

#[get("/")]
fn index() -> Result<File, std::io::Error> {
	File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/public/index.html"))
}

#[get("/resume")]
fn resume() -> Result<File, std::io::Error> {
	File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/public/resume.html"))
}

fn main() {
	server().launch();
}
