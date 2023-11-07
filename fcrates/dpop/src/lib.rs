//! This crate provides rust implementation of [DPoP](https://datatracker.ietf.org/doc/html/draft-ietf-oauth-dpop-14).
//!

#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(unused_qualifications)]

#[cfg(feature = "http-header")]
pub mod http_header;

pub mod proof;

// #[cfg(test)]
// #[allow(missing_docs)]
// mod test_files {
//     use picky::{key::PrivateKey, pem::Pem};

//     pub const RSA_2048_PK_1: &str = include_str!("../test_assets/private_keys/rsa-2048-pk_1.key");
//     pub const RSA_2048_PK_7: &str = include_str!("../test_assets/private_keys/rsa-2048-pk_7.key");
//     pub const RSA_4096_PK_3: &str = include_str!("../test_assets/private_keys/rsa-4096-pk_3.key");

//     pub fn get_private_key_1() -> PrivateKey {
//         let pk_pem = RSA_2048_PK_1.parse::<Pem>().unwrap();
//         PrivateKey::from_pkcs8(pk_pem.data()).unwrap()
//     }
// }
