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

#![doc = include_str!("../README.md")]
#![deny(warnings)]
#![deny(missing_docs)]
#![deny(clippy::print_stdout)]

mod captcha_gen;
mod finder;
mod storage;

use std::{sync::Arc, time::Duration};

use salvo_core::{
    handler::{none_skipper, Skipper},
    Depot, FlowCtrl, Handler, Request, Response,
};
pub use {captcha_gen::*, finder::*, storage::*};

/// Key used to insert the captcha state into the depot
pub const CAPTCHA_STATE_KEY: &str = "::salvo_captcha::captcha_state";

/// The captcha middleware
///
/// The captcha middleware is used to check the captcha token and answer from
/// the request. You can use the [`CaptchaBuilder`] to create a new captcha
/// middleware.
///
/// ## Note
/// You need to generate the captcha token and answer before, then the captcha
/// middleware will check the token and answer from the request using the finder
/// and storage you provided. The captcha middleware will insert the
/// [`CaptchaState`] into the depot, you can get the captcha state from the
/// depot using the [`CaptchaDepotExt::get_captcha_state`] trait, which is
/// implemented for the [`Depot`].
/// 
/// Check the [`examples`](https://git.4rs.nl/awiteb/salvo-captcha/src/branch/master/examples) for more information.
#[non_exhaustive]
pub struct Captcha<S, F>
where
    S: CaptchaStorage,
    F: CaptchaFinder,
{
    /// The captcha finder, used to find the captcha token and answer from the request.
    finder: F,
    /// The storage of the captcha, used to store and get the captcha token and answer.
    storage: Arc<S>,
    /// The skipper of the captcha, used to skip the captcha check.
    skipper: Box<dyn Skipper>,
    /// The case sensitive of the captcha answer.
    case_sensitive: bool,
}

/// The captcha states of the request
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptchaState {
    /// The captcha check is skipped. This depends on the skipper.
    #[default]
    Skipped,
    /// The captcha is checked and passed. If the captcha is passed, it will be cleared from the storage.
    Passed,
    /// Can't find the captcha token in the request
    TokenNotFound,
    /// Can't find the captcha answer in the request
    AnswerNotFound,
    /// Can't find the captcha token in the storage or the token is wrong (not valid string)
    WrongToken,
    /// Can't find the captcha answer in the storage or the answer is wrong (not valid string)
    WrongAnswer,
    /// Storage error
    StorageError,
}

/// The [`Captcha`] builder
pub struct CaptchaBuilder<S, F>
where
    S: CaptchaStorage,
    F: CaptchaFinder,
{
    storage: S,
    finder: F,
    captcha_expired_after: Duration,
    clean_interval: Duration,
    skipper: Box<dyn Skipper>,
    case_sensitive: bool,
}

impl<S, F> CaptchaBuilder<Arc<S>, F>
where
    S: CaptchaStorage,
    F: CaptchaFinder,
{
    /// Create a new [`CaptchaBuilder`] with the given storage and finder.
    pub fn new(storage: Arc<S>, finder: F) -> Self {
        CaptchaBuilder {
            storage,
            finder,
            captcha_expired_after: Duration::from_secs(60 * 5),
            clean_interval: Duration::from_secs(60),
            skipper: Box::new(none_skipper),
            case_sensitive: true,
        }
    }

    /// Remove the case sensitive of the captcha, default is case sensitive.
    ///
    /// This will make the captcha case insensitive, for example, the answer "Hello" will be the same as "hello".
    pub fn case_insensitive(mut self) -> Self {
        self.case_sensitive = false;
        self
    }

    /// Set the duration after which the captcha will be expired, default is 5 minutes.
    ///
    /// After the captcha is expired, it will be removed from the storage, and the user needs to get a new captcha.
    pub fn expired_after(mut self, expired_after: impl Into<Duration>) -> Self {
        self.captcha_expired_after = expired_after.into();
        self
    }

    /// Set the interval to clean the expired captcha, default is 1 minute.
    ///
    /// The expired captcha will be removed from the storage every interval.
    pub fn clean_interval(mut self, interval: impl Into<Duration>) -> Self {
        self.clean_interval = interval.into();
        self
    }

    /// Set the skipper of the captcha, default without skipper.
    ///
    /// The skipper is used to skip the captcha check, for example, you can skip the captcha check for the admin user.
    pub fn skipper(mut self, skipper: impl Skipper) -> Self {
        self.skipper = Box::new(skipper);
        self
    }

    /// Build the [`Captcha`] with the given configuration.
    pub fn build(self) -> Captcha<S, F> {
        Captcha::new(
            self.storage,
            self.finder,
            self.captcha_expired_after,
            self.clean_interval,
            self.skipper,
            self.case_sensitive,
        )
    }
}

impl<S, F> Captcha<S, F>
where
    S: CaptchaStorage,
    F: CaptchaFinder,
{
    /// Create a new Captcha
    fn new(
        storage: Arc<S>,
        finder: F,
        captcha_expired_after: Duration,
        clean_interval: Duration,
        skipper: Box<dyn Skipper>,
        case_sensitive: bool,
    ) -> Self {
        let task_storage = Arc::clone(&storage);

        tokio::spawn(async move {
            loop {
                if let Err(err) = task_storage.clear_expired(captcha_expired_after).await {
                    log::error!("Captcha storage error: {err}")
                }
                tokio::time::sleep(clean_interval).await;
            }
        });

        Self {
            finder,
            storage,
            skipper,
            case_sensitive,
        }
    }
}

/// The captcha extension of the depot.
/// Used to get the captcha info from the depot.
pub trait CaptchaDepotExt {
    /// Get the captcha state from the depot
    fn get_captcha_state(&self) -> CaptchaState;
}

impl CaptchaDepotExt for Depot {
    fn get_captcha_state(&self) -> CaptchaState {
        self.get(CAPTCHA_STATE_KEY).cloned().unwrap_or_default()
    }
}

#[salvo_core::async_trait]
impl<S, F> Handler for Captcha<S, F>
where
    S: CaptchaStorage,
    F: CaptchaFinder,
{
    async fn handle(
        &self,
        req: &mut Request,
        depot: &mut Depot,
        _: &mut Response,
        _: &mut FlowCtrl,
    ) {
        if self.skipper.as_ref().skipped(req, depot) {
            log::info!("Captcha check is skipped");
            depot.insert(CAPTCHA_STATE_KEY, CaptchaState::Skipped);
            return;
        }

        let token = match self.finder.find_token(req).await {
            Some(Some(token)) => token,
            Some(None) => {
                log::info!("Captcha token is not found in request");
                depot.insert(CAPTCHA_STATE_KEY, CaptchaState::TokenNotFound);
                return;
            }
            None => {
                log::error!("Invalid token found in request");
                depot.insert(CAPTCHA_STATE_KEY, CaptchaState::WrongToken);
                return;
            }
        };

        let answer = match self.finder.find_answer(req).await {
            Some(Some(answer)) => answer,
            Some(None) => {
                log::info!("Captcha answer is not found in request");
                depot.insert(CAPTCHA_STATE_KEY, CaptchaState::AnswerNotFound);
                return;
            }
            None => {
                log::error!("Invalid answer found in request");
                depot.insert(CAPTCHA_STATE_KEY, CaptchaState::WrongAnswer);
                return;
            }
        };

        match self.storage.get_answer(&token).await {
            Ok(Some(captch_answer)) => {
                log::info!("Captcha answer is exist in storage for token: {token}");
                if (captch_answer == answer && self.case_sensitive)
                    || captch_answer.eq_ignore_ascii_case(&answer)
                {
                    log::info!("Captcha answer is correct for token: {token}");
                    self.storage.clear_by_token(&token).await.ok();
                    depot.insert(CAPTCHA_STATE_KEY, CaptchaState::Passed);
                } else {
                    log::info!("Captcha answer is wrong for token: {token}");
                    depot.insert(CAPTCHA_STATE_KEY, CaptchaState::WrongAnswer);
                }
            }
            Ok(None) => {
                log::info!("Captcha answer is not exist in storage for token: {token}");
                depot.insert(CAPTCHA_STATE_KEY, CaptchaState::WrongToken);
            }
            Err(err) => {
                log::error!("Failed to get captcha answer from storage: {err}");
                depot.insert(CAPTCHA_STATE_KEY, CaptchaState::StorageError);
            }
        };
    }
}
