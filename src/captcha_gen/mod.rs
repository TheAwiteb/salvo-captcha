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

#[cfg(feature = "simple-generator")]
mod simple_generator;

#[cfg_attr(docsrs, doc(cfg(feature = "simple-generator")))]
#[cfg(feature = "simple-generator")]
pub use simple_generator::*;

/// Captcha generator, used to generate a new captcha image and answer.
pub trait CaptchaGenerator: Send {
    /// The error type of the captcha generator
    type Error: std::error::Error;

    /// Create a new captcha image and return the answer and the image encoded as png
    fn new_captcha(
        &self,
    ) -> impl std::future::Future<Output = Result<(String, Vec<u8>), Self::Error>> + Send;
}
