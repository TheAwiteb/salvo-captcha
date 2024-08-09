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

use salvo_core::{
    handler::{none_skipper, Skipper},
    Depot, FlowCtrl, Handler, Request, Response,
};
pub use {captcha_gen::*, finder::*, storage::*};

// Exports from other crates
pub use captcha::{CaptchaName, Difficulty as CaptchaDifficulty};

/// Key used to insert the captcha state into the depot
pub const CAPTCHA_STATE_KEY: &str = "::salvo_captcha::captcha_state";

/// Captcha struct, contains the token and answer.
#[non_exhaustive]
#[allow(clippy::type_complexity)]
pub struct Captcha<S, F>
where
    S: CaptchaStorage,
    F: CaptchaFinder,
{
    /// The captcha finder, used to find the captcha token and answer from the request.
    finder: F,
    /// The storage of the captcha, used to store and get the captcha token and answer.
    storage: S,
    /// The skipper of the captcha, used to skip the captcha check.
    skipper: Box<dyn Skipper>,
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

impl<S, F> Captcha<S, F>
where
    S: CaptchaStorage,
    F: CaptchaFinder,
{
    /// Create a new Captcha
    pub fn new(storage: S, finder: F) -> Self {
        Self {
            finder,
            storage,
            skipper: Box::new(none_skipper),
        }
    }

    /// Returns the captcha storage
    pub fn storage(&self) -> &S {
        &self.storage
    }

    /// Set the captcha skipper, the skipper will be used to check if the captcha check should be skipped.
    pub fn skipper(mut self, skipper: impl Skipper) -> Self {
        self.skipper = Box::new(skipper);
        self
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
    F: CaptchaFinder + 'static, // why?
{
    async fn handle(
        &self,
        req: &mut Request,
        depot: &mut Depot,
        _: &mut Response,
        _: &mut FlowCtrl,
    ) {
        if self.skipper.skipped(req, depot) {
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
                if captch_answer == answer {
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
