#![feature(decl_macro, proc_macro_hygiene)]

extern crate sentry;

#[macro_use]
extern crate rocket;
extern crate rocket_contrib;

use std::env;
use std::fs::File;
use std::mem;
use std::path::Path;

use rocket::fairing;
use rocket_contrib::serve::StaticFiles;

pub struct ErrorReporter {}

impl fairing::Fairing for ErrorReporter {
	fn info(&self) -> fairing::Info {
		fairing::Info {
			name: "Sentry Error Reporter",
			kind: fairing::Kind::Request | fairing::Kind::Response,
		}
	}

	fn on_response(&self, request: &rocket::Request, response: &mut rocket::Response) {
		use rocket::http::Status;
		use sentry::protocol::{value::Map, Event, Level, Value};
		use std::collections::BTreeMap;
		use std::iter::FromIterator;

		match response.status() {
			Status::NotFound | Status::InternalServerError => {
				let request_headers = Map::from_iter(
					request
						.headers()
						.iter()
						.map(|header| (String::from(header.name()), header.value().into())),
				);

				let response_headers = Map::from_iter(
					response
						.headers()
						.iter()
						.map(|header| (String::from(header.name()), header.value().into())),
				);

				let mut extra = BTreeMap::<String, Value>::new();
				extra.insert("request.headers".to_string(), request_headers.into());
				extra.insert("response.headers".to_string(), response_headers.into());

				sentry::capture_event(Event {
					message: Some(format!("Error: {}", response.status().to_string())),
					level: Level::Error,
					user: Some(sentry::User {
						ip_address: Some(sentry::protocol::IpAddress::Exact(
							request.client_ip().unwrap(),
						)),
						..Default::default()
					}),
					extra,
					..Default::default()
				});
			}
			_ => {}
		}
	}
}

impl std::default::Default for ErrorReporter {
	fn default() -> ErrorReporter {
		ErrorReporter {}
	}
}

/// Returns a fully-assembled Rocket, ready for ignition.
fn server() -> rocket::Rocket {
	let static_dir: &Path = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/public"));

	rocket::Rocket::ignite()
		.mount("/", StaticFiles::from(static_dir))
		.mount("/", routes![index, resume])
		.attach(fairing::AdHoc::on_attach(
			"Sentry Client creator",
			|rocket| {
				let dsn: Option<sentry::internals::Dsn> = env::var("SENTRY_DSN")
					.ok()
					.or_else(|| match rocket.config().get_str("sentry_dsn") {
						Ok(s) => Some(String::from(s)),
						Err(_) => None,
					})
					.map(|dsn: String| -> sentry::internals::Dsn {
						dsn.parse::<sentry::internals::Dsn>().unwrap()
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
		.attach(ErrorReporter::default())
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
