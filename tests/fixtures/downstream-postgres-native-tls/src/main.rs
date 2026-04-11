fn build_native_tls_connector() -> Result<native_tls::TlsConnector, native_tls::Error> {
	native_tls::TlsConnector::builder()
		.danger_accept_invalid_certs(true)
		.build()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let _connector = postgres_native_tls::MakeTlsConnector::new(build_native_tls_connector()?);
	Ok(())
}