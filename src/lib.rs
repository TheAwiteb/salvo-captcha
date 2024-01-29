// Copyright (c) 2024, Awiteb <awiteb@hotmail.com>
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
    F: CaptchaFinder<Token = S::Token, Answer = S::Answer>,
{
    /// The captcha finder, used to find the captcha token and answer from the request.
    finder: F,
    /// The storage of the captcha, used to store and get the captcha token and answer.
    storage: S,
    /// The skipper of the captcha, used to skip the captcha check.
    skipper: Box<dyn Skipper>,
}

/// The captcha states of the request
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptchaState {
    /// The captcha is checked and passed. If the captcha is passed, it will be cleared from the storage.
    Passed,
    /// The captcha check is skipped. This depends on the skipper.
    Skipped,
    /// Can't find the captcha token in the request
    TokenNotFound,
    /// Can't find the captcha answer in the request
    AnswerNotFound,
    /// The captcha token is wrong, can't find the captcha in the storage.
    /// Maybe the captcha token entered by the user is wrong, or the captcha is expired, because the storage has been cleared.
    WrongToken,
    /// The captcha answer is wrong. This will not clear the captcha from the storage.
    WrongAnswer,
    /// Storage error
    StorageError,
}

impl<S, F> Captcha<S, F>
where
    S: CaptchaStorage,
    F: CaptchaFinder<Token = S::Token, Answer = S::Answer>,
{
    /// Create a new Captcha
    pub fn new(storage: impl Into<S>, finder: impl Into<F>) -> Self {
        Self {
            finder: finder.into(),
            storage: storage.into(),
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
#[easy_ext::ext(CaptchaDepotExt)]
impl Depot {
    /// Get the captcha state from the depot
    pub fn get_captcha_state(&self) -> Option<&CaptchaState> {
        self.get(CAPTCHA_STATE_KEY).ok()
    }
}

#[async_trait::async_trait]
impl<S, F> Handler for Captcha<S, F>
where
    S: CaptchaStorage,
    F: CaptchaFinder<Token = S::Token, Answer = S::Answer> + 'static,
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
            Ok(Some(token)) => token,
            Ok(None) => {
                log::info!("Captcha token is not found in request");
                depot.insert(CAPTCHA_STATE_KEY, CaptchaState::TokenNotFound);
                return;
            }
            Err(err) => {
                log::error!("Failed to find captcha token from request: {err:?}");
                depot.insert(CAPTCHA_STATE_KEY, CaptchaState::WrongToken);
                return;
            }
        };

        let answer = match self.finder.find_answer(req).await {
            Ok(Some(answer)) => answer,
            Ok(None) => {
                log::info!("Captcha answer is not found in request");
                depot.insert(CAPTCHA_STATE_KEY, CaptchaState::AnswerNotFound);
                return;
            }
            Err(err) => {
                log::error!("Failed to find captcha answer from request: {err:?}");
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
