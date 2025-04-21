use axum::{extract::Form, response::Html};
use serde::Deserialize;

//
// For Authenticated users
// Have the user complete additional information
//
#[allow(dead_code)]
pub async fn show() -> Html<&'static str> {
    Html(
        r#"
        <!doctype html>
        <html>
            <head></head>
            <body>
                <!-- ⬜ CONFIG to  match the route entry -->
                <form action="/auth/api/forms/user" method="post">
                <div>
                    <label for="name">
                        Enter your name:
                        <input type="text" name="name">
                    </label>
                </div>
                <div>
                    <label>
                        Enter your email:
                        <input type="text" name="email">
                    </label>
                </div>
                <div>
                    <input type="submit" value="Subscribe!">
                </div>
                </form>
            </body>
        </html>
        "#,
    )
}

#[derive(Deserialize, Debug)]
pub struct Input {
    pub name: String,
    pub email: String,
}

#[allow(dead_code)]
pub async fn accept(Form(input): Form<Input>) {
    dbg!(&input);
}
