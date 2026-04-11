#![deny(rustdoc::broken_intra_doc_links)]
//! OpenSSL passthrough shim for the compat-openssl Phase 1 baseline.

#[cfg(all(feature = "backend-boring", feature = "backend-openssl"))]
compile_error!("backend-openssl and backend-boring are mutually exclusive");

#[cfg(not(any(feature = "backend-boring", feature = "backend-openssl")))]
compile_error!("either backend-openssl or backend-boring must be enabled");

#[cfg(all(feature = "backend-boring", not(feature = "backend-openssl")))]
pub use boring::*;
#[cfg(all(feature = "backend-openssl", not(feature = "backend-boring")))]
pub use real_backend_openssl::*;
