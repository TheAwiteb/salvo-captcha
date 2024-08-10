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
    #[rstest::rstest]
    #[case::not_found(None, None, None, None, None, None)]
    #[case::normal(
         None,
         None,
         Some(("x-captcha-token", "token")),
         Some(("x-captcha-answer", "answer")),
         Some(Some("token")),
         Some(Some("answer"))
    )]
    #[case::custom_headers(
         Some("custom-token"),
         Some("custom-answer"),
         Some(("custom-token", "token")),
         Some(("custom-answer", "answer")),
         Some(Some("token")),
         Some(Some("answer"))
    )]
    #[case::custom_not_found(Some("custom-token"), Some("custom-answer"), None, None, None, None)]
    #[case::custom_not_found_with_headers(
         Some("custom-token"),
         Some("custom-answer"),
         Some(("x-captcha-token", "token")),
         Some(("x-captcha-answer", "answer")),
         None,
         None
    )]
    async fn test_header_finder(
        #[case] custom_token_header: Option<&'static str>,
        #[case] custom_answer_header: Option<&'static str>,
        #[case] token_header_name_value: Option<(&'static str, &'static str)>,
        #[case] answer_header_name_value: Option<(&'static str, &'static str)>,
        #[case] excepted_token: Option<Option<&'static str>>,
        #[case] excepted_answer: Option<Option<&'static str>>,
    ) {
        let mut finder = CaptchaHeaderFinder::new();
        if let Some(custom_token) = custom_token_header {
            finder = finder.token_header(HeaderName::from_static(custom_token));
        }
        if let Some(custom_answer) = custom_answer_header {
            finder = finder.answer_header(HeaderName::from_static(custom_answer));
        }

        let mut req = Request::default();
        let headers = req.headers_mut();
        if let Some((token_header_name, token_header_value)) = token_header_name_value {
            headers.insert(
                HeaderName::from_static(token_header_name),
                HeaderValue::from_static(token_header_value),
            );
        }
        if let Some((answer_header_name, answer_header_value)) = answer_header_name_value {
            headers.insert(
                HeaderName::from_static(answer_header_name),
                HeaderValue::from_static(answer_header_value),
            );
        }

        assert_eq!(
            finder.find_token(&mut req).await,
            excepted_token.map(|o| o.map(ToOwned::to_owned))
        );
        assert_eq!(
            finder.find_answer(&mut req).await,
            excepted_answer.map(|o| o.map(ToOwned::to_owned))
        );
    }
}
