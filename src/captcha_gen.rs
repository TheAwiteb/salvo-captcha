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

use crate::{CaptchaDifficulty, CaptchaName, CaptchaStorage};

/// Captcha generator, used to generate a new captcha image. This trait are implemented for all [`CaptchaStorage`].
pub trait CaptchaGenerator: CaptchaStorage {
    /// Create a new captcha image and return the token and the image encoded as png. Will return None if the captcha crate failed to create the captcha.
    ///
    /// The returned captcha image is 220x110 pixels.
    ///
    /// For more information about the captcha name and difficulty, see the [`README.md`](https://github.com/TheAwiteb/salvo-captcha/#captcha-name-and-difficulty).
    fn new_captcha(
        &self,
        name: CaptchaName,
        difficulty: CaptchaDifficulty,
    ) -> impl std::future::Future<Output = Result<Option<(String, Vec<u8>)>, Self::Error>> + Send
    {
        async {
            let Some((captcha_answer, captcha_image)) =
                captcha::by_name(difficulty, name).as_tuple()
            else {
                return Ok(None);
            };

            let token = self.store_answer(captcha_answer).await?;
            Ok(Some((token, captcha_image)))
        }
    }
}

impl<T> CaptchaGenerator for T where T: CaptchaStorage {}
