#[macro_use]
extern crate rocket;
extern crate rocket_contrib;

/// Returns a fully-assembled Rocket, ready for ignition.
fn server() -> rocket::Rocket {
	rocket::Rocket::ignite()
}

fn main() {
	println!("Hello world!");

	server().launch();
}
