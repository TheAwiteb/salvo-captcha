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
mod cacache_storage;
mod memory_storage;

#[cfg(feature = "cacache-storage")]
pub use cacache_storage::*;
pub use memory_storage::*;

/// Trait to store the captcha token and answer. is also clear the expired captcha.
///
/// The trait will be implemented for `Arc<T>` if `T` implements the trait.
///
/// The trait is thread safe, so the storage can be shared between threads.
pub trait CaptchaStorage: Send + Sync + 'static {
    /// The error type of the storage.
    type Error: std::error::Error + Send;

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
