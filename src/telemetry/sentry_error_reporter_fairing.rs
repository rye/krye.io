use rocket::fairing;

pub struct SentryErrorReporterFairing;

impl rocket::fairing::Fairing for SentryErrorReporterFairing {
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

impl Default for SentryErrorReporterFairing {
	fn default() -> Self {
		Self {}
	}
}
