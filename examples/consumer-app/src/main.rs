use std::mem::size_of;

use openssl::{
	error::ErrorStack,
	ssl::{SslConnector, SslMethod, SslVerifyMode},
};
use tokio::io::DuplexStream;

type AsyncNativeTlsStream = tokio_native_tls::TlsStream<DuplexStream>;

const COMPAT_OPENSSL_BACKEND: &str = env!("COMPAT_OPENSSL_BACKEND");
const COMPAT_OPENSSL_INCLUDE: &str = env!("COMPAT_OPENSSL_INCLUDE");
const COMPAT_OPENSSL_VERSION_NUMBER: &str = env!("COMPAT_OPENSSL_VERSION_NUMBER");

fn build_openssl_connector() -> Result<SslConnector, ErrorStack> {
	let mut builder = SslConnector::builder(SslMethod::tls())?;
	builder.set_verify(SslVerifyMode::NONE);
	Ok(builder.build())
}

fn build_native_tls_connector() -> Result<native_tls::TlsConnector, native_tls::Error> {
	native_tls::TlsConnector::builder()
		.danger_accept_invalid_certs(true)
		.build()
}

fn build_tokio_native_tls_connector() -> Result<tokio_native_tls::TlsConnector, native_tls::Error>
{
	Ok(tokio_native_tls::TlsConnector::from(
		build_native_tls_connector()?,
	))
}

fn build_postgres_tls_connector(
) -> Result<postgres_native_tls::MakeTlsConnector, native_tls::Error> {
	Ok(postgres_native_tls::MakeTlsConnector::new(
		build_native_tls_connector()?,
	))
}

fn raw_ssl_ctx_pointer_size() -> usize {
	size_of::<*mut openssl_sys::SSL_CTX>()
}

fn build_reqwest_client_builder() -> reqwest::ClientBuilder {
	reqwest::Client::builder()
}

fn touch_async_stream_types(_native_tls_stream: Option<AsyncNativeTlsStream>) {}

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let _openssl_connector = build_openssl_connector()?;
	let _native_tls_connector = build_native_tls_connector()?;
	let _tokio_native_tls_connector = build_tokio_native_tls_connector()?;
	let _postgres_tls_connector = build_postgres_tls_connector()?;
	let _reqwest_client_builder = build_reqwest_client_builder();
	let _raw_ssl_ctx_pointer_size = raw_ssl_ctx_pointer_size();
	let _metadata_snapshot = (
		COMPAT_OPENSSL_BACKEND,
		COMPAT_OPENSSL_INCLUDE,
		COMPAT_OPENSSL_VERSION_NUMBER,
	);

	touch_async_stream_types(None);
	Ok(())
}