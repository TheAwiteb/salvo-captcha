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

#![allow(warnings)]

use std::{
    collections::HashMap,
    convert::Infallible,
    time::{Duration, SystemTime},
};
use tokio::sync::RwLock;

use crate::CaptchaStorage;

/// Captcha storage implementation using an in-memory [HashMap].
#[derive(Debug)]
pub struct MemoryStorage(RwLock<HashMap<String, (u64, String)>>);

impl MemoryStorage {
    /// Create a new instance of [`MemoryStorage`].
    pub fn new() -> Self {
        Self(RwLock::new(HashMap::new()))
    }
}

impl CaptchaStorage for MemoryStorage {
    /// This storage does not return any error.
    type Error = Infallible;

    async fn store_answer(&self, answer: String) -> Result<String, Self::Error> {
        let token = uuid::Uuid::new_v4().to_string();
        let mut write_lock = self.0.write().await;
        write_lock.insert(token.clone(), (now(), answer));

        Ok(token)
    }

    async fn get_answer(&self, token: &str) -> Result<Option<String>, Self::Error> {
        let reader = self.0.read().await;
        Ok(reader.get(token).map(|(_, answer)| answer.to_owned()))
    }

    async fn clear_expired(&self, expired_after: Duration) -> Result<(), Self::Error> {
        let expired_after = now() - expired_after.as_secs();

        let mut write_lock = self.0.write().await;
        write_lock.retain(|_, (timestamp, _)| *timestamp > expired_after);

        Ok(())
    }

    async fn clear_by_token(&self, token: &str) -> Result<(), Self::Error> {
        let mut write_lock = self.0.write().await;
        write_lock.retain(|c_token, (_, _)| c_token != token);
        Ok(())
    }
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("SystemTime before UNIX EPOCH!")
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn memory_store_captcha() {
        let storage = MemoryStorage::new();

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
    async fn memory_clear_expired() {
        let storage = MemoryStorage::new();

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
    async fn memory_clear_by_token() {
        let storage = MemoryStorage::new();

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
    async fn memory_is_token_exist() {
        let storage = MemoryStorage::new();

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
    async fn memory_clear_expired_with_expired_after() {
        let storage = MemoryStorage::new();

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
