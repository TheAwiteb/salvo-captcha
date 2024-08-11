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

/// Find the captcha token and answer from the url query
#[derive(Debug)]
pub struct CaptchaQueryFinder {
    /// The query name of the captcha token
    ///
    /// Default: "c_t"
    pub token_name: String,

    /// The query name of the captcha answer
    ///
    /// Default: "c_a"
    pub answer_name: String,
}

impl CaptchaQueryFinder {
    /// Create a new [`CaptchaQueryFinder`]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the token query name
    pub fn token_name(mut self, token_name: String) -> Self {
        self.token_name = token_name;
        self
    }

    /// Set the answer query name
    pub fn answer_name(mut self, answer_name: String) -> Self {
        self.answer_name = answer_name;
        self
    }
}

impl Default for CaptchaQueryFinder {
    /// Create a default [`CaptchaQueryFinder`] with:
    /// - token_name: "c_t"
    /// - answer_name: "c_a"
    fn default() -> Self {
        Self {
            token_name: "c_t".to_string(),
            answer_name: "c_a".to_string(),
        }
    }
}

impl CaptchaFinder for CaptchaQueryFinder {
    async fn find_token(&self, req: &mut Request) -> Option<Option<String>> {
        req.queries()
            .get(&self.token_name)
            .map(|o| Some(o.to_owned()))
    }

    async fn find_answer(&self, req: &mut Request) -> Option<Option<String>> {
        req.queries()
            .get(&self.answer_name)
            .map(|o| Some(o.to_owned()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[rstest::rstest]
    #[case::not_found(None, None, None, None, None, None)]
    #[case::normal(
        None,
        None,
        Some(("c_t", "token")),
        Some(("c_a", "answer")),
        Some(Some("token")),
        Some(Some("answer"))
    )]
    #[case::custom_keys(
        Some("cc_t"),
        Some("cc_a"),
        Some(("cc_t", "token")),
        Some(("cc_a", "answer")),
        Some(Some("token")),
        Some(Some("answer"))
    )]
    #[case::only_token(
        None,
        None,
        Some(("c_t", "token")),
        None,
        Some(Some("token")),
        None
    )]
    #[case::only_answer(None, None, None, Some(("c_a", "ans")), None, Some(Some("ans")))]
    #[case::custom_not_found(Some("cc_t"), Some("cc_a"), None, None, None, None)]
    #[case::custom_not_found_with_query(
        Some("cc_t"),
        Some("cc_a"),
        Some(("c_t", "token")),
        Some(("c_a", "answer")),
        None,
        None
    )]
    async fn test_query_finder(
        #[case] custom_token_key: Option<&'static str>,
        #[case] custom_answer_key: Option<&'static str>,
        #[case] token_key_val: Option<(&'static str, &'static str)>,
        #[case] answer_key_val: Option<(&'static str, &'static str)>,
        #[case] excepted_token: Option<Option<&'static str>>,
        #[case] excepted_answer: Option<Option<&'static str>>,
    ) {
        let mut req = Request::default();
        let mut finder = CaptchaQueryFinder::new();
        if let Some(token_key) = custom_token_key {
            finder = finder.token_name(token_key.to_string())
        }
        if let Some(answer_key) = custom_answer_key {
            finder = finder.answer_name(answer_key.to_string())
        }

        let queries = req.queries_mut();

        if let Some((k, v)) = token_key_val {
            queries.insert(k.to_owned(), v.to_owned());
        }
        if let Some((k, v)) = answer_key_val {
            queries.insert(k.to_owned(), v.to_owned());
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
