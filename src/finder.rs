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

use std::marker::PhantomData;

use salvo_core::http::header::HeaderName;
use salvo_core::http::Request;

/// Trait to find the captcha token and answer from the request.
pub trait CaptchaFinder: Send + Sync {
    /// The token type
    type Token: TryFrom<String> + Sync + Send;
    /// The answer type
    type Answer: TryFrom<String> + Sync + Send;

    /// The token error type
    type TError: std::fmt::Debug + Send;
    /// The answer error type
    type AError: std::fmt::Debug + Send;

    /// Find the captcha token from the request.
    ///
    /// Return [`None`] if the request does not contain a captcha token. An error is returned if the token is invalid format.
    fn find_token(
        &self,
        req: &mut Request,
    ) -> impl std::future::Future<Output = Result<Option<Self::Token>, Self::TError>> + std::marker::Send;

    /// Find the captcha answer from the request.
    ///
    /// Return [`None`] if the request does not contain a captcha answer. An error is returned if the answer is invalid format.
    fn find_answer(
        &self,
        req: &mut Request,
    ) -> impl std::future::Future<Output = Result<Option<Self::Answer>, Self::AError>> + std::marker::Send;
}

/// Find the captcha token and answer from the header
#[derive(Debug)]
pub struct CaptchaHeaderFinder<T, A>
where
    T: TryFrom<String> + Sync + Send,
    A: TryFrom<String> + Sync + Send,
{
    phantom: PhantomData<(T, A)>,

    /// The header name of the captcha token
    ///
    /// Default: "x-captcha-token"
    pub token_header: HeaderName,

    /// The header name of the captcha answer
    ///
    /// Default: "x-captcha-answer"
    pub answer_header: HeaderName,
}

/// Find the captcha token and answer from the form
#[derive(Debug)]
pub struct CaptchaFormFinder<T, A>
where
    T: TryFrom<String> + Sync + Send,
    A: TryFrom<String> + Sync + Send,
{
    phantom: PhantomData<(T, A)>,

    /// The form name of the captcha token
    ///
    /// Default: "captcha_token"
    pub token_name: String,

    /// The form name of the captcha answer
    ///
    /// Default: "captcha_answer"
    pub answer_name: String,
}

impl<T, A> CaptchaHeaderFinder<T, A>
where
    T: TryFrom<String> + Sync + Send,
    A: TryFrom<String> + Sync + Send,
{
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

impl<T, A> CaptchaFormFinder<T, A>
where
    T: TryFrom<String> + Sync + Send,
    A: TryFrom<String> + Sync + Send,
{
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

impl<T, A> Default for CaptchaHeaderFinder<T, A>
where
    T: TryFrom<String> + Sync + Send,
    A: TryFrom<String> + Sync + Send,
{
    /// Create a default CaptchaHeaderFinder with:
    /// - token_header: "x-captcha-token"
    /// - answer_header: "x-captcha-answer"
    fn default() -> Self {
        Self {
            phantom: PhantomData,
            token_header: HeaderName::from_static("x-captcha-token"),
            answer_header: HeaderName::from_static("x-captcha-answer"),
        }
    }
}

impl<T, A> Default for CaptchaFormFinder<T, A>
where
    T: TryFrom<String> + Sync + Send,
    A: TryFrom<String> + Sync + Send,
{
    /// Create a default CaptchaFormFinder with:
    /// - token_name: "captcha_token"
    /// - answer_name: "captcha_answer"
    fn default() -> Self {
        Self {
            phantom: PhantomData,
            token_name: "captcha_token".to_string(),
            answer_name: "captcha_answer".to_string(),
        }
    }
}

impl<T, A> CaptchaFinder for CaptchaHeaderFinder<T, A>
where
    T: TryFrom<String> + Sync + Send,
    A: TryFrom<String> + Sync + Send,
    <T as TryFrom<String>>::Error: Send,
    <T as TryFrom<String>>::Error: std::fmt::Debug,
    <A as TryFrom<String>>::Error: Send,
    <A as TryFrom<String>>::Error: std::fmt::Debug,
{
    type Token = T;
    type Answer = A;

    type TError = <T as TryFrom<String>>::Error;
    type AError = <A as TryFrom<String>>::Error;

    async fn find_token(&self, req: &mut Request) -> Result<Option<Self::Token>, Self::TError> {
        req.headers()
            .get(&self.token_header)
            .and_then(|t| t.to_str().ok())
            .map(|t| Self::Token::try_from(t.to_string()))
            .transpose()
    }

    async fn find_answer(&self, req: &mut Request) -> Result<Option<Self::Answer>, Self::AError> {
        req.headers()
            .get(&self.answer_header)
            .and_then(|a| a.to_str().ok())
            .map(|a| Self::Answer::try_from(a.to_string()))
            .transpose()
    }
}

impl<T, A> CaptchaFinder for CaptchaFormFinder<T, A>
where
    T: TryFrom<String> + Sync + Send,
    A: TryFrom<String> + Sync + Send,
    <T as TryFrom<String>>::Error: Send,
    <T as TryFrom<String>>::Error: std::fmt::Debug,
    <A as TryFrom<String>>::Error: Send,
    <A as TryFrom<String>>::Error: std::fmt::Debug,
{
    type Token = T;
    type Answer = A;

    type TError = <T as TryFrom<String>>::Error;
    type AError = <A as TryFrom<String>>::Error;

    async fn find_token(&self, req: &mut Request) -> Result<Option<Self::Token>, Self::TError> {
        req.form::<String>(&self.token_name)
            .await
            .map(|t| Self::Token::try_from(t.to_string()))
            .transpose()
    }

    async fn find_answer(&self, req: &mut Request) -> Result<Option<Self::Answer>, Self::AError> {
        req.form::<String>(&self.answer_name)
            .await
            .map(|a| Self::Answer::try_from(a.to_string()))
            .transpose()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use salvo_core::http::headers::ContentType;
    use salvo_core::http::Request;
    use salvo_core::http::{header::*, ReqBody};

    #[tokio::test]
    async fn test_captcha_header_finder() {
        let finder = CaptchaHeaderFinder::<String, String>::new();
        let mut req = Request::default();
        let headers = req.headers_mut();
        let token = uuid::Uuid::new_v4();
        headers.insert(
            HeaderName::from_static("x-captcha-token"),
            HeaderValue::from_str(&token.to_string()).unwrap(),
        );
        headers.insert(
            HeaderName::from_static("x-captcha-answer"),
            HeaderValue::from_static("answer"),
        );
        assert_eq!(
            finder.find_token(&mut req).await,
            Ok(Some(token.to_string()))
        );
        assert!(matches!(
            finder.find_answer(&mut req).await,
            Ok(Some(a)) if a == *"answer"
        ));
    }

    #[tokio::test]
    async fn test_captcha_header_finder_customized() {
        let finder = CaptchaHeaderFinder::<String, String>::new()
            .token_header(HeaderName::from_static("token"))
            .answer_header(HeaderName::from_static("answer"));
        let mut req = Request::default();
        let headers = req.headers_mut();
        let token = uuid::Uuid::new_v4();
        headers.insert(
            HeaderName::from_static("token"),
            HeaderValue::from_str(&token.to_string()).unwrap(),
        );
        headers.insert(
            HeaderName::from_static("answer"),
            HeaderValue::from_static("answer"),
        );
        assert_eq!(
            finder.find_token(&mut req).await,
            Ok(Some(token.to_string()))
        );
        assert!(matches!(
            finder.find_answer(&mut req).await,
            Ok(Some(a)) if a == *"answer"
        ));
    }

    #[tokio::test]
    async fn test_captcha_header_finder_none() {
        let finder = CaptchaHeaderFinder::<String, String>::new();
        let mut req = Request::default();

        assert_eq!(finder.find_token(&mut req).await, Ok(None));
        assert_eq!(finder.find_answer(&mut req).await, Ok(None));
    }

    #[tokio::test]
    async fn test_captcha_header_finder_customized_none() {
        let finder = CaptchaHeaderFinder::<String, String>::new()
            .token_header(HeaderName::from_static("token"))
            .answer_header(HeaderName::from_static("answer"));
        let mut req = Request::default();

        assert_eq!(finder.find_token(&mut req).await, Ok(None));
        assert_eq!(finder.find_answer(&mut req).await, Ok(None));
    }

    #[tokio::test]
    async fn test_captcha_form_finder() {
        let finder = CaptchaFormFinder::<String, String>::new();
        let mut req = Request::default();
        *req.body_mut() = ReqBody::Once("captcha_token=token&captcha_answer=answer".into());
        let headers = req.headers_mut();
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_str(&ContentType::form_url_encoded().to_string()).unwrap(),
        );

        assert_eq!(
            finder.find_token(&mut req).await,
            Ok(Some("token".to_string()))
        );
        assert!(matches!(
            finder.find_answer(&mut req).await,
            Ok(Some(a)) if a == *"answer"
        ));
    }

    #[tokio::test]
    async fn test_captcha_form_finder_customized() {
        let finder = CaptchaFormFinder::<String, String>::new()
            .token_name("token".to_string())
            .answer_name("answer".to_string());
        let mut req = Request::default();
        *req.body_mut() = ReqBody::Once("token=token&answer=answer".into());
        let headers = req.headers_mut();
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_str(&ContentType::form_url_encoded().to_string()).unwrap(),
        );

        assert_eq!(
            finder.find_token(&mut req).await,
            Ok(Some("token".to_string()))
        );
        assert!(matches!(
            finder.find_answer(&mut req).await,
            Ok(Some(a)) if a == *"answer"
        ));
    }

    #[tokio::test]
    async fn test_captcha_form_finder_none() {
        let finder = CaptchaFormFinder::<String, String>::new();
        let mut req = Request::default();
        *req.body_mut() = ReqBody::Once("".into());
        let headers = req.headers_mut();
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_str(&ContentType::form_url_encoded().to_string()).unwrap(),
        );

        assert_eq!(finder.find_token(&mut req).await, Ok(None));
        assert_eq!(finder.find_answer(&mut req).await, Ok(None));
    }

    #[tokio::test]
    async fn test_captcha_form_finder_customized_none() {
        let finder = CaptchaFormFinder::<String, String>::new()
            .token_name("token".to_string())
            .answer_name("answer".to_string());
        let mut req = Request::default();
        *req.body_mut() = ReqBody::Once("".into());
        let headers = req.headers_mut();
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_str(&ContentType::form_url_encoded().to_string()).unwrap(),
        );

        assert_eq!(finder.find_token(&mut req).await, Ok(None));
        assert_eq!(finder.find_answer(&mut req).await, Ok(None));
    }

    #[tokio::test]
    async fn test_captcha_form_finder_invalid() {
        let finder = CaptchaFormFinder::<String, String>::new();
        let mut req = Request::default();
        *req.body_mut() = ReqBody::Once("captcha_token=token&captcha_answer=answer".into());
        let headers = req.headers_mut();
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_str(&ContentType::json().to_string()).unwrap(),
        );

        assert_eq!(finder.find_token(&mut req).await, Ok(None));
        assert_eq!(finder.find_answer(&mut req).await, Ok(None));
    }
}
