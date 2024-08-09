// Copyright (c) 2024, Awiteb <a@4rs.nl>
//     A captcha middleware for Salvo framework.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

use std::{sync::Arc, time::Duration};

#[cfg(feature = "cacache-storage")]
use std::{
    path::{Path, PathBuf},
    time::SystemTime,
};

/// Trait to store the captcha token and answer. is also clear the expired captcha.
///
/// The trait will be implemented for `Arc<T>` if `T` implements the trait.
///
/// The trait is thread safe, so the storage can be shared between threads.
pub trait CaptchaStorage: Send + Sync + 'static
where
    Self: Clone + std::fmt::Debug,
{
    /// The error type of the storage.
    type Error: std::fmt::Display + std::fmt::Debug + Send;

    /// Store the captcha token and answer.
    fn store_answer(
        &self,
        answer: String,
    ) -> impl std::future::Future<Output = Result<String, Self::Error>> + Send;

    /// Returns the answer of the captcha token. This method will return None if the token is not exist.
    fn get_answer(
        &self,
        token: &str,
    ) -> impl std::future::Future<Output = Result<Option<String>, Self::Error>> + Send;

    /// Clear the expired captcha.
    fn clear_expired(
        &self,
        expired_after: Duration,
    ) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send;

    /// Clear the captcha by token.
    fn clear_by_token(
        &self,
        token: &str,
    ) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send;
}

/// The [`cacache`] storage.
///
/// [`cacache`]: https://github.com/zkat/cacache-rs
#[cfg(feature = "cacache-storage")]
#[derive(Debug, Clone)]
pub struct CacacheStorage {
    /// The cacache cache directory.
    cache_dir: PathBuf,
}

#[cfg(feature = "cacache-storage")]
impl CacacheStorage {
    /// Create a new CacacheStorage
    pub fn new(cache_dir: impl Into<PathBuf>) -> Self {
        Self {
            cache_dir: cache_dir.into(),
        }
    }

    /// Get the cacache cache directory.
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }
}

#[cfg(feature = "cacache-storage")]
impl CaptchaStorage for CacacheStorage {
    type Error = cacache::Error;

    async fn store_answer(&self, answer: String) -> Result<String, Self::Error> {
        let token = uuid::Uuid::new_v4();
        log::info!("Storing captcha answer to cacache for token: {token}");
        cacache::write(&self.cache_dir, token.to_string(), answer.as_bytes()).await?;
        Ok(token.to_string())
    }

    async fn get_answer(&self, token: &str) -> Result<Option<String>, Self::Error> {
        log::info!("Getting captcha answer from cacache for token: {token}");
        match cacache::read(&self.cache_dir, token).await {
            Ok(answer) => {
                log::info!("Captcha answer is exist in cacache for token: {token}");
                Ok(Some(
                    String::from_utf8(answer)
                        .expect("All the stored captcha answer should be utf8"),
                ))
            }
            Err(cacache::Error::EntryNotFound(_, _)) => {
                log::info!("Captcha answer is not exist in cacache for token: {token}");
                Ok(None)
            }
            Err(err) => {
                log::error!("Failed to get captcha answer from cacache for token: {token}");
                Err(err)
            }
        }
    }

    async fn clear_expired(&self, expired_after: Duration) -> Result<(), Self::Error> {
        let now = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("SystemTime before UNIX EPOCH!")
            .as_millis();
        let expired_after = expired_after.as_millis();

        let expr_keys = cacache::index::ls(&self.cache_dir).filter_map(|meta| {
            if let Ok(meta) = meta {
                if now >= (meta.time + expired_after) {
                    return Some(meta.key);
                }
            }
            None
        });

        for key in expr_keys {
            cacache::RemoveOpts::new()
                .remove_fully(true)
                .remove(&self.cache_dir, &key)
                .await
                .ok();
        }
        Ok(())
    }

    async fn clear_by_token(&self, token: &str) -> Result<(), Self::Error> {
        log::info!("Clearing captcha token from cacache: {token}");
        let remove_opts = cacache::RemoveOpts::new().remove_fully(true);
        remove_opts.remove(&self.cache_dir, token).await
    }
}

impl<T> CaptchaStorage for Arc<T>
where
    T: CaptchaStorage,
{
    type Error = T::Error;

    fn store_answer(
        &self,
        answer: String,
    ) -> impl std::future::Future<Output = Result<String, Self::Error>> + Send {
        self.as_ref().store_answer(answer)
    }

    fn get_answer(
        &self,
        token: &str,
    ) -> impl std::future::Future<Output = Result<Option<String>, Self::Error>> + Send {
        self.as_ref().get_answer(token)
    }

    fn clear_expired(
        &self,
        expired_after: Duration,
    ) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send {
        self.as_ref().clear_expired(expired_after)
    }

    fn clear_by_token(
        &self,
        token: &str,
    ) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send {
        self.as_ref().clear_by_token(token)
    }
}

#[cfg(test)]
#[cfg(feature = "cacache-storage")]
mod tests {
    use super::*;

    #[tokio::test]
    async fn cacache_store_captcha() {
        let storage = CacacheStorage::new(
            tempfile::tempdir()
                .expect("failed to create temp file")
                .path()
                .to_owned(),
        );

        let token = storage
            .store_answer("answer".to_owned())
            .await
            .expect("failed to store captcha");
        assert_eq!(
            storage
                .get_answer(&token)
                .await
                .expect("failed to get captcha answer"),
            Some("answer".to_owned())
        );
    }

    #[tokio::test]
    async fn cacache_clear_expired() {
        let storage = CacacheStorage::new(
            tempfile::tempdir()
                .expect("failed to create temp file")
                .path()
                .to_owned(),
        );

        let token = storage
            .store_answer("answer".to_owned())
            .await
            .expect("failed to store captcha");
        storage
            .clear_expired(Duration::from_secs(0))
            .await
            .expect("failed to clear expired captcha");
        assert!(storage
            .get_answer(&token)
            .await
            .expect("failed to get captcha answer")
            .is_none());
    }

    #[tokio::test]
    async fn cacache_clear_by_token() {
        let storage = CacacheStorage::new(
            tempfile::tempdir()
                .expect("failed to create temp file")
                .path()
                .to_owned(),
        );

        let token = storage
            .store_answer("answer".to_owned())
            .await
            .expect("failed to store captcha");
        storage
            .clear_by_token(&token)
            .await
            .expect("failed to clear captcha by token");
        assert!(storage
            .get_answer(&token)
            .await
            .expect("failed to get captcha answer")
            .is_none());
    }

    #[tokio::test]
    async fn cacache_is_token_exist() {
        let storage = CacacheStorage::new(
            tempfile::tempdir()
                .expect("failed to create temp file")
                .path()
                .to_owned(),
        );

        let token = storage
            .store_answer("answer".to_owned())
            .await
            .expect("failed to store captcha");
        assert!(storage
            .get_answer(&token)
            .await
            .expect("failed to check if token is exist")
            .is_some());
        assert!(storage
            .get_answer("token")
            .await
            .expect("failed to check if token is exist")
            .is_none());
    }

    #[tokio::test]
    async fn cacache_get_answer() {
        let storage = CacacheStorage::new(
            tempfile::tempdir()
                .expect("failed to create temp file")
                .path()
                .to_owned(),
        );

        let token = storage
            .store_answer("answer".to_owned())
            .await
            .expect("failed to store captcha");
        assert_eq!(
            storage
                .get_answer(&token)
                .await
                .expect("failed to get captcha answer"),
            Some("answer".to_owned())
        );
        assert!(storage
            .get_answer("token")
            .await
            .expect("failed to get captcha answer")
            .is_none());
    }

    #[tokio::test]
    async fn cacache_cache_dir() {
        let cache_dir = tempfile::tempdir()
            .expect("failed to create temp file")
            .path()
            .to_owned();
        let storage = CacacheStorage::new(cache_dir.clone());
        assert_eq!(storage.cache_dir(), &cache_dir);
    }

    #[tokio::test]
    async fn cacache_clear_expired_with_expired_after() {
        let storage = CacacheStorage::new(
            tempfile::tempdir()
                .expect("failed to create temp file")
                .path()
                .to_owned(),
        );

        let token = storage
            .store_answer("answer".to_owned())
            .await
            .expect("failed to store captcha");
        storage
            .clear_expired(Duration::from_secs(1))
            .await
            .expect("failed to clear expired captcha");
        assert_eq!(
            storage
                .get_answer(&token)
                .await
                .expect("failed to get captcha answer"),
            Some("answer".to_owned())
        );
        tokio::time::sleep(Duration::from_secs(1)).await;
        storage
            .clear_expired(Duration::from_secs(1))
            .await
            .expect("failed to clear expired captcha");
        assert!(storage
            .get_answer(&token)
            .await
            .expect("failed to get captcha answer")
            .is_none());
    }
}
