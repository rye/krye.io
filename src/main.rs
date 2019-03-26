#![feature(decl_macro, proc_macro_hygiene, test)]

#[macro_use]
extern crate rocket;
extern crate rocket_contrib;

extern crate sentry;
extern crate serde;

extern crate test;

extern crate krye_io;
use krye_io::*;

use std::env;
use std::mem;
use std::path::Path;

use rocket_contrib::serve::StaticFiles;
use rocket_contrib::templates::Template;

/// Returns a fully-assembled Rocket, ready for ignition.
fn server() -> rocket::Rocket {
	let static_dir: &Path = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/public"));

	rocket::Rocket::ignite()
		.mount("/", StaticFiles::from(static_dir))
		.mount("/", routes![index, resume])
		.attach(Template::fairing())
		.attach(rocket::fairing::AdHoc::on_attach(
			"Sentry Client creator",
			|rocket| {
				use sentry::internals::Dsn;

				let dsn: Option<Dsn> = env::var("SENTRY_DSN")
					.ok()
					.or_else(|| match rocket.config().get_str("sentry_dsn") {
						Ok(s) => Some(String::from(s)),
						Err(_) => None,
					})
					.map(|dsn: String| -> Dsn {
						dsn.parse::<Dsn>().unwrap()
					});

				let env = format!("{:?}", rocket.config().environment);
				let release = format!("{}@{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

				mem::forget(sentry::init(sentry::ClientOptions {
					dsn,
					environment: Some(env.into()),
					release: Some(release.into()),
					..Default::default()
				}));

				sentry::integrations::panic::register_panic_handler();

				Ok(rocket)
			},
		))
		.attach(SentryErrorReporterFairing::default())
}

#[get("/")]
fn index() -> Template {
	let context: Context = Default::default();

	Template::render("index", context)
}

#[get("/resume")]
fn resume() -> Template {
	let context: Context = Default::default();
	Template::render("resume", context)
}

fn main() {
	server().launch();
}

#[cfg(test)]
mod benches {
	#[bench]
	fn index(b: &mut test::Bencher) {
		b.iter(|| super::index())
	}

	#[bench]
	fn resume(b: &mut test::Bencher) {
		b.iter(|| super::resume())
	}
}
