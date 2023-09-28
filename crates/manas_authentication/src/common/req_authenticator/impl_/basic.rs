use std::marker::PhantomData;

use crate::common::{credentials::RequestCredentials, req_authenticator::RequestAuthenticator};

/// A basic implementation of [`RequestAuthenticator`], that
/// sets credentials as an typed extension in request.
#[derive(Debug)]
pub struct BasicRequestAuthenticator<C> {
    _phantom: PhantomData<fn(C)>,
}

impl<C: RequestCredentials> RequestAuthenticator for BasicRequestAuthenticator<C> {
    type Credentials = C;
}
