fn main() {
	let _ = core::mem::size_of::<*mut openssl_sys::SSL_CTX>();
}