#[cfg(test)]
extern crate fantoccini;
#[cfg(test)]
extern crate futures;
#[cfg(test)]
extern crate tokio;

extern crate krye_io;

#[test]
#[ignore]
fn loads_okay() {
	use fantoccini::{Client, Locator};
	use futures::Future;

	let c = Client::new("http://localhost:9515");

	let mut rt = tokio::runtime::Runtime::new().unwrap();

	let res = rt.block_on(
		c.map_err(|e| unimplemented!("failed to connect to WebDriver: {:?}", e))
			.and_then(|c| c.goto("http://localhost:8000/"))
			.and_then(|mut c| c.current_url().map(move |url| (c, url)))
			.and_then(|(mut c, url)| {
				assert_eq!(url.as_ref(), "http://localhost:8000/");
				c.find(Locator::LinkText("Check out my r\u{00e9}sum\u{00e9}"))
			})
			.and_then(|e| e.click())
			.and_then(|mut c| c.current_url())
			.and_then(|url| {
				assert_eq!(url.as_ref(), "http://localhost:8000/resume");
				Ok(())
			})
	);

	rt.shutdown_on_idle().wait().unwrap();

	assert!(res.is_ok(), format!("Could not complete checklist: {:?}", res.unwrap_err()));
}
