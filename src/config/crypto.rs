//cryto
// deals with password hashing
// and Json WebTokens

use color_eyre::Result;
use eyre::eyre;
use std::sync::Arc;
use argonautica::Hasher;
use futures::compat::Future01CompatExt;
use tracing::instrument;

#[derive(Debug, Clone)]
pub struct CryptoService {
    pub key: Arc<String>
}

impl CryptoService {

    #[instrument(skip(self, password))]
    pub async fn hash_password(&self, password: String) -> Result<String> {
        Hasher::default()
            .with_secret_key(&*self.key)
            .with_password(password)
            .hash_non_blocking()
            .compat()//Future01CompatExt converts to current version of Future
            .await
            .map_err(|err| eyre!("Hashing error: {:?}", err))
    }
}

