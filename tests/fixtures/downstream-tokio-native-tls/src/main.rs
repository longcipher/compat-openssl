use tokio::io::DuplexStream;

type AsyncTlsStream = tokio_native_tls::TlsStream<DuplexStream>;

fn build_native_tls_connector() -> Result<native_tls::TlsConnector, native_tls::Error> {
	native_tls::TlsConnector::builder()
		.danger_accept_invalid_certs(true)
		.build()
}

fn build_tokio_tls_connector() -> Result<tokio_native_tls::TlsConnector, native_tls::Error> {
	Ok(tokio_native_tls::TlsConnector::from(
		build_native_tls_connector()?,
	))
}

fn touch_async_stream_type(_stream: Option<AsyncTlsStream>) {}

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let _connector = build_native_tls_connector()?;
	let _tokio_connector = build_tokio_tls_connector()?;
	touch_async_stream_type(None);
	Ok(())
}