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

use crate::CaptchaGenerator;

use std::fmt::Display;

/// Supported captcha names
///
/// See [`README.md`](https://git.4rs.nl/awiteb/salvo-captcha/#captcha-name-and-difficulty) for more information.
#[derive(Debug, Clone, Copy)]
pub enum CaptchaName {
    /// Plain text, without any distortion
    Normal,
    /// Slightly twisted text
    SlightlyTwisted,
    /// Very twisted text
    VeryTwisted,
}

/// Supported captcha difficulties
///
/// See [`README.md`](https://git.4rs.nl/awiteb/salvo-captcha/#captcha-name-and-difficulty) for more information.
#[derive(Debug, Clone, Copy)]
pub enum CaptchaDifficulty {
    /// Easy to read text
    Easy,
    /// Medium difficulty text
    Medium,
    /// Hard to read text
    Hard,
}

impl From<CaptchaName> for captcha::CaptchaName {
    /// Function to convert the [`CaptchaName`] to the [`captcha::CaptchaName`]
    fn from(value: CaptchaName) -> Self {
        match value {
            CaptchaName::Normal => Self::Lucy,
            CaptchaName::SlightlyTwisted => Self::Amelia,
            CaptchaName::VeryTwisted => Self::Mila,
        }
    }
}

impl From<CaptchaDifficulty> for captcha::Difficulty {
    /// Function to convert the [`CaptchaDifficulty`] to the [`captcha::Difficulty`]
    fn from(value: CaptchaDifficulty) -> captcha::Difficulty {
        match value {
            CaptchaDifficulty::Easy => Self::Easy,
            CaptchaDifficulty::Medium => Self::Medium,
            CaptchaDifficulty::Hard => Self::Hard,
        }
    }
}

#[derive(Debug)]
/// Error type for the [`SimpleGenerator`]
pub enum SimpleGeneratorError {
    /// Failed to encode the captcha to png image
    FaildEncodedToPng,
}

impl Display for SimpleGeneratorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Faild to encode the captcha to png image")
    }
}

impl std::error::Error for SimpleGeneratorError {}

/// A simple captcha generator, using the [`captcha`](https://crates.io/crates/captcha) crate.
pub struct SimpleGenerator {
    name: CaptchaName,
    difficulty: CaptchaDifficulty,
}

impl SimpleGenerator {
    /// Create new [`SimpleGenerator`] instance
    pub const fn new(name: CaptchaName, difficulty: CaptchaDifficulty) -> Self {
        Self { name, difficulty }
    }
}

impl CaptchaGenerator for SimpleGenerator {
    type Error = SimpleGeneratorError;

    /// The returned captcha image is 220x110 pixels in png format.
    async fn new_captcha(&self) -> Result<(String, Vec<u8>), Self::Error> {
        let Some((captcha_answer, captcha_image)) =
            captcha::by_name(self.difficulty.into(), self.name.into()).as_tuple()
        else {
            return Err(SimpleGeneratorError::FaildEncodedToPng);
        };

        Ok((captcha_answer, captcha_image))
    }
}
