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

use salvo_core::http::{HeaderName, Request};

use crate::CaptchaFinder;

/// Find the captcha token and answer from the header
#[derive(Debug)]
pub struct CaptchaHeaderFinder {
    /// The header name of the captcha token
    ///
    /// Default: "x-captcha-token"
    pub token_header: HeaderName,

    /// The header name of the captcha answer
    ///
    /// Default: "x-captcha-answer"
    pub answer_header: HeaderName,
}

impl CaptchaHeaderFinder {
    /// Create a new CaptchaHeaderFinder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the token header name
    pub fn token_header(mut self, token_header: HeaderName) -> Self {
        self.token_header = token_header;
        self
    }

    /// Set the answer header name
    pub fn answer_header(mut self, answer_header: HeaderName) -> Self {
        self.answer_header = answer_header;
        self
    }
}

impl Default for CaptchaHeaderFinder {
    /// Create a default CaptchaHeaderFinder with:
    /// - token_header: "x-captcha-token"
    /// - answer_header: "x-captcha-answer"
    fn default() -> Self {
        Self {
            token_header: HeaderName::from_static("x-captcha-token"),
            answer_header: HeaderName::from_static("x-captcha-answer"),
        }
    }
}

impl CaptchaFinder for CaptchaHeaderFinder {
    async fn find_token(&self, req: &mut Request) -> Option<Option<String>> {
        req.headers()
            .get(&self.token_header)
            .map(|t| t.to_str().map(ToString::to_string).ok())
    }

    async fn find_answer(&self, req: &mut Request) -> Option<Option<String>> {
        req.headers()
            .get(&self.answer_header)
            .map(|a| a.to_str().map(ToString::to_string).ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use salvo_core::http::HeaderValue;

    #[tokio::test]
    async fn test_captcha_header_finder() {
        let finder = CaptchaHeaderFinder::new();
        let mut req = Request::default();
        let headers = req.headers_mut();

        headers.insert(
            HeaderName::from_static("x-captcha-token"),
            HeaderValue::from_str("token").unwrap(),
        );
        headers.insert(
            HeaderName::from_static("x-captcha-answer"),
            HeaderValue::from_static("answer"),
        );

        assert_eq!(
            finder.find_token(&mut req).await,
            Some(Some("token".to_owned()))
        );
        assert_eq!(
            finder.find_answer(&mut req).await,
            Some(Some("answer".to_owned()))
        );
    }

    #[tokio::test]
    async fn test_captcha_header_finder_customized() {
        let finder = CaptchaHeaderFinder::new()
            .token_header(HeaderName::from_static("token"))
            .answer_header(HeaderName::from_static("answer"));

        let mut req = Request::default();
        let headers = req.headers_mut();

        headers.insert(
            HeaderName::from_static("token"),
            HeaderValue::from_str("token").unwrap(),
        );
        headers.insert(
            HeaderName::from_static("answer"),
            HeaderValue::from_static("answer"),
        );

        assert_eq!(
            finder.find_token(&mut req).await,
            Some(Some("token".to_owned()))
        );
        assert_eq!(
            finder.find_answer(&mut req).await,
            Some(Some("answer".to_owned()))
        );
    }

    #[tokio::test]
    async fn test_captcha_header_finder_none() {
        let finder = CaptchaHeaderFinder::new();
        let mut req = Request::default();

        assert_eq!(finder.find_token(&mut req).await, None);
        assert_eq!(finder.find_answer(&mut req).await, None);
    }

    #[tokio::test]
    async fn test_captcha_header_finder_customized_none() {
        let finder = CaptchaHeaderFinder::new()
            .token_header(HeaderName::from_static("token"))
            .answer_header(HeaderName::from_static("answer"));
        let mut req = Request::default();
        let headers = req.headers_mut();

        headers.insert(
            HeaderName::from_static("x-captcha-token"),
            HeaderValue::from_str("token").unwrap(),
        );
        headers.insert(
            HeaderName::from_static("x-captcha-answer"),
            HeaderValue::from_static("answer"),
        );

        assert_eq!(finder.find_token(&mut req).await, None);
        assert_eq!(finder.find_answer(&mut req).await, None);
    }
}
