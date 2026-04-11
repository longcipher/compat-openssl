use openssl as _;

fn main() {
    let _ = native_tls::TlsConnector::builder();
}