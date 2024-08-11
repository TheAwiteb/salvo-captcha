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

use salvo_core::http::Request;

mod form_finder;
mod header_finder;
mod query_finder;

pub use form_finder::*;
pub use header_finder::*;
pub use query_finder::*;

/// Trait to find the captcha token and answer from the request.
pub trait CaptchaFinder: Send + Sync {
    /// Find the captcha token from the request.
    ///
    /// ### Returns
    /// - None: If the token is not found
    /// - Some(None): If the token is found but is invalid (e.g. not a valid string)
    /// - Some(Some(token)): If the token is found
    fn find_token(
        &self,
        req: &mut Request,
    ) -> impl std::future::Future<Output = Option<Option<String>>> + std::marker::Send;

    /// Find the captcha answer from the request.
    ///
    /// ### Returns
    /// - None: If the answer is not found
    /// - Some(None): If the answer is found but is invalid (e.g. not a valid string)
    /// - Some(Some(answer)): If the answer is found
    fn find_answer(
        &self,
        req: &mut Request,
    ) -> impl std::future::Future<Output = Option<Option<String>>> + std::marker::Send;
}
