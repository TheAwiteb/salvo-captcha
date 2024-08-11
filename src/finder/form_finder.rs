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
    use salvo_core::http::{header, headers::ContentType, HeaderValue, ReqBody};

    use super::*;

    #[tokio::test]
    async fn test_captcha_form_finder() {
        let finder = CaptchaFormFinder::new();
        let mut req = Request::default();
        *req.body_mut() = ReqBody::Once("captcha_token=token&captcha_answer=answer".into());
        let headers = req.headers_mut();
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_str(&ContentType::form_url_encoded().to_string()).unwrap(),
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
    async fn test_captcha_form_finder_customized() {
        let finder = CaptchaFormFinder::new()
            .token_name("token".to_string())
            .answer_name("answer".to_string());
        let mut req = Request::default();
        *req.body_mut() = ReqBody::Once("token=token&answer=answer".into());
        let headers = req.headers_mut();
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_str(&ContentType::form_url_encoded().to_string()).unwrap(),
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
    async fn test_captcha_form_finder_none() {
        let finder = CaptchaFormFinder::new();
        let mut req = Request::default();
        *req.body_mut() = ReqBody::Once("".into());
        let headers = req.headers_mut();
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_str(&ContentType::form_url_encoded().to_string()).unwrap(),
        );

        assert_eq!(finder.find_token(&mut req).await, Some(None));
        assert_eq!(finder.find_answer(&mut req).await, Some(None));
    }

    #[tokio::test]
    async fn test_captcha_form_finder_customized_none() {
        let finder = CaptchaFormFinder::new()
            .token_name("token".to_string())
            .answer_name("answer".to_string());
        let mut req = Request::default();
        *req.body_mut() = ReqBody::Once("".into());
        let headers = req.headers_mut();
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_str(&ContentType::form_url_encoded().to_string()).unwrap(),
        );

        assert_eq!(finder.find_token(&mut req).await, Some(None));
        assert_eq!(finder.find_answer(&mut req).await, Some(None));
    }

    #[tokio::test]
    async fn test_captcha_form_finder_invalid() {
        let finder = CaptchaFormFinder::new();
        let mut req = Request::default();
        *req.body_mut() = ReqBody::Once("captcha_token=token&captcha_answer=answer".into());
        let headers = req.headers_mut();
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_str(&ContentType::json().to_string()).unwrap(),
        );

        assert_eq!(finder.find_token(&mut req).await, None);
        assert_eq!(finder.find_answer(&mut req).await, None);
    }
}
