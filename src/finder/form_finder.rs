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

use crate::CaptchaFinder;

/// Find the captcha token and answer from the form
#[derive(Debug)]
pub struct CaptchaFormFinder {
    /// The form name of the captcha token
    ///
    /// Default: "captcha_token"
    pub token_name: String,

    /// The form name of the captcha answer
    ///
    /// Default: "captcha_answer"
    pub answer_name: String,
}

impl CaptchaFormFinder {
    /// Create a new CaptchaFormFinder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the token form name
    pub fn token_name(mut self, token_name: String) -> Self {
        self.token_name = token_name;
        self
    }

    /// Set the answer form name
    pub fn answer_name(mut self, answer_name: String) -> Self {
        self.answer_name = answer_name;
        self
    }
}

impl Default for CaptchaFormFinder {
    /// Create a default CaptchaFormFinder with:
    /// - token_name: "captcha_token"
    /// - answer_name: "captcha_answer"
    fn default() -> Self {
        Self {
            token_name: "captcha_token".to_string(),
            answer_name: "captcha_answer".to_string(),
        }
    }
}

impl CaptchaFinder for CaptchaFormFinder {
    async fn find_token(&self, req: &mut Request) -> Option<Option<String>> {
        req.form_data()
            .await
            .ok()
            .and_then(|form| form.fields.get(&self.token_name).cloned().map(Some))
    }

    async fn find_answer(&self, req: &mut Request) -> Option<Option<String>> {
        req.form_data()
            .await
            .ok()
            .and_then(|form| form.fields.get(&self.answer_name).cloned().map(Some))
    }
}

#[cfg(test)]
mod tests {
    use salvo_core::http::{header, HeaderValue, ReqBody};

    use super::*;

    #[tokio::test]
    #[rstest::rstest]
    #[case::not_found(
        None,
        None,
        None,
        None,
        "application/x-www-form-urlencoded",
        None,
        None
    )]
    #[case::not_found(None, None, None, None, "text/plain", None, None)]
    #[case::normal(
        None,
        None,
        Some(("captcha_token", "token")),
        Some(("captcha_answer", "answer")),
        "application/x-www-form-urlencoded",
        Some(Some("token")),
        Some(Some("answer"))
    )]
    #[case::custom_keys(
        Some("custom_token"),
        Some("custom_answer"),
        Some(("custom_token", "token")),
        Some(("custom_answer", "answer")),
        "application/x-www-form-urlencoded",
        Some(Some("token")),
        Some(Some("answer"))
    )]
    #[case::only_token(
        None,
        None,
        Some(("captcha_token", "token")),
        None,
        "application/x-www-form-urlencoded",
        Some(Some("token")),
        None
    )]
    #[case::only_answer(
        None,
        None,
        None,
        Some(("captcha_answer", "answer")),
        "application/x-www-form-urlencoded",
        None,
        Some(Some("answer"))
    )]
    #[case::custom_not_found(
        Some("custom_token"),
        Some("custom_answer"),
        None,
        None,
        "application/x-www-form-urlencoded",
        None,
        None
    )]
    #[case::custom_not_found_with_body(
        Some("custom_token"),
        Some("custom_answer"),
        Some(("captcha_token", "token")),
        Some(("captcha_answer", "answer")),
        "application/x-www-form-urlencoded",
        None,
        None
    )]
    #[case::invalid_type(
        None,
        None,
        Some(("captcha_token", "token")),
        Some(("captcha_answer", "answer")),
        "application/json",
        None,
        None
    )]
    async fn test_form_finder(
        #[case] custom_token_key: Option<&'static str>,
        #[case] custom_answer_key: Option<&'static str>,
        #[case] token_key_val: Option<(&'static str, &'static str)>,
        #[case] answer_key_val: Option<(&'static str, &'static str)>,
        #[case] content_type: &'static str,
        #[case] excepted_token: Option<Option<&'static str>>,
        #[case] excepted_answer: Option<Option<&'static str>>,
    ) {
        let mut req = Request::default();
        let mut finder = CaptchaFormFinder::new();
        if let Some(token_key) = custom_token_key {
            finder = finder.token_name(token_key.to_string())
        }
        if let Some(answer_key) = custom_answer_key {
            finder = finder.answer_name(answer_key.to_string())
        }

        let body = token_key_val
            .zip(answer_key_val)
            .map(|((t_k, t_v), (a_k, a_v))| format!("{t_k}={t_v}&{a_k}={a_v}"))
            .unwrap_or_else(|| {
                token_key_val
                    .or(answer_key_val)
                    .map(|(k, v)| format!("{k}={v}"))
                    .unwrap_or_default()
            });

        *req.body_mut() = ReqBody::Once(body.into());
        let headers = req.headers_mut();
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_str(content_type).unwrap(),
        );

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
