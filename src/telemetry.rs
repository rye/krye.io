pub trait ErrorReporter {
	fn report();
}

mod sentry_error_reporter_fairing;
pub use sentry_error_reporter_fairing::*;
