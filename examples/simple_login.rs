// Example of using the `salvo_captcha`, this example is a simple login page with a captcha.
// The page will be in <http://127.0.0.1:5800>
// You can see a video of this example here:
//
// Run the example with `cargo run --example simple_login --features cacache-storage`

use std::sync::Arc;

use base64::{engine::GeneralPurpose, Engine};
use salvo::prelude::*;
use salvo_captcha::*;

// To convert the image to base64, to show it in the browser
const BASE_64_ENGINE: GeneralPurpose = GeneralPurpose::new(
    &base64::alphabet::STANDARD,
    base64::engine::general_purpose::PAD,
);

const SIMPLE_GENERATOR: SimpleGenerator =
    SimpleGenerator::new(CaptchaName::Normal, CaptchaDifficulty::Medium);

#[handler]
async fn index(res: &mut Response, depot: &mut Depot) {
    // Get the captcha from the depot
    let captcha_storage = depot.obtain::<Arc<MemoryStorage>>().unwrap();

    // Create a new captcha
    let Ok((token, image)) = captcha_storage.new_captcha(SIMPLE_GENERATOR).await else {
        res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
        res.render(Text::Html(
            "<html><body><h1>Server Error 500</h1></body></html>",
        ));
        return;
    };

    // Convert the image to base64
    let image = BASE_64_ENGINE.encode(image);

    // Set the response content
    res.render(Text::Html(index_page(image, token)))
}

#[handler]
async fn auth(req: &mut Request, res: &mut Response, depot: &mut Depot) {
    // Get the captcha state from the depot, where we can know if the captcha is passed
    let captcha_state = depot.get_captcha_state();
    // Not important, just for demo
    let Some(username) = req.form::<String>("username").await else {
        res.status_code(StatusCode::BAD_REQUEST);
        return res.render(Text::Html("Invalid form submission"));
    };

    // Handle the captcha state, that's all
    let content = match captcha_state {
        CaptchaState::Passed => {
            format!("Welcome, {username}!")
        }
        CaptchaState::AnswerNotFound => "Captcha answer not found".to_string(),
        CaptchaState::TokenNotFound => "Captcha token not found".to_string(),
        CaptchaState::WrongAnswer => "Wrong captcha answer".to_string(),
        CaptchaState::WrongToken => "Wrong captcha token".to_string(),
        CaptchaState::Skipped => "Captcha skipped".to_string(),
        CaptchaState::StorageError => "Captcha storage error".to_string(),
    };

    res.render(Text::Html(captcha_result_page(content)))
}

#[tokio::main]
async fn main() {
    let captcha_storage = Arc::new(MemoryStorage::new());
    let captcha_middleware =
        CaptchaBuilder::new(Arc::clone(&captcha_storage), CaptchaFormFinder::new())
            // Skip the captcha if the request path is /skipped
            .skipper(|req: &mut Request, _: &Depot| req.uri().path() == "/skipped")
            .case_insensitive()
            .build();

    let router = Router::new()
        .hoop(affix::inject(captcha_storage))
        .push(Router::with_path("/").get(index))
        .push(
            Router::new()
                .hoop(captcha_middleware)
                .push(Router::with_path("/auth").post(auth))
                .push(Router::with_path("/skipped").post(auth)),
        );

    let acceptor = TcpListener::new(("127.0.0.1", 5800)).bind().await;
    Server::new(acceptor).serve(router).await;
}

fn index_page(captcha_image: String, captcha_token: String) -> String {
    format!(
        r#"
    <html>
        <head>
            <title>Salvo Captcha Example</title>
        </head>
        <style>
            body {{
                text-align: center;
            }}
            .captcha-img {{
                width: 220px;
                height: 110px;
                border: 5px solid black;
                padding: 5px;
                margin: 5px;
                box-shadow: 0 0 5px rgba(0, 0, 0, 0.3);
                border-radius: 5px;
            }}
            input {{
                margin: 5px;
                padding: 5px;
                border: 1px solid black;
                border-radius: 3px;
            }}
        </style>
        <body>
            <h1>Salvo Captcha Example</h1>
            <h2>Sign In</h2>
            <img class="captcha-img" src="data:image/png;base64,{captcha_image}" />
            <form action="/auth" method="post">
                <input type="hidden" name="captcha_token" value="{captcha_token}" />

                <input type="text" name="username" placeholder="Username" />
                <br/>
                <input type="password" name="password" placeholder="Password" />
                <br/>
                <input type="text" name="captcha_answer" placeholder="Captcha Answer" />
                <br/>
                <input type="submit" value="Submit" />
            </form>
            <srong>Or you can skip the captcha</strong>
            <form action="/skipped" method="post">
                <input type="text" name="username" placeholder="Username" />
                <br/>
                <input type="password" name="password" placeholder="Password" />
                <br/>
                <input type="submit" value="Skip Captcha" />
            </form>
            <a href="https://git.4rs.nl/awiteb/salvo-captcha">Source Code</a>
        </body>
    </html>
    "#
    )
}

fn captcha_result_page(captcha_result: String) -> String {
    format!(
        r#"
    <html>
        <head>
            <title>Salvo Captcha Example</title>
        </head>
        <style>
            body {{
                text-align: center;
            }}
        </style>
        <body>
            <h1>Salvo Captcha Example</h1>
            <h2>Result page</h2>
            <strong>{captcha_result}</strong>
            <br/>
            <a href="/">Go Back</a>
        </body>
    </html>
    "#
    )
}
