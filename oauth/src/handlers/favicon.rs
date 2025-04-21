use axum::{
    body::Full,
    http::HeaderValue,
    response::{IntoResponse, Response},
};
use http::header;

pub async fn go() -> impl IntoResponse {
    // one pixel favicon generated from https://png-pixel.com/
    let one_pixel_favicon = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mPk+89QDwADvgGOSHzRgAAAAABJRU5ErkJggg==";
    let pixel_favicon = base64::decode(one_pixel_favicon).unwrap();
    let mut res = Response::new(Full::from(pixel_favicon));
    res.headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_static("image/png"));
    res
}
