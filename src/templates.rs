use crate::domain::ExplainResult;
use crate::routes::response::ExplainResultWithAnalysis;
use askama::Template;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};

#[derive(Template, Debug)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    // name: String,
    pub explains: Vec<ExplainResultWithAnalysis>,
}

#[derive(Template)]
#[template(path = "journals.html")]
pub struct JournalTemplate {
    // name: String,
    pub explains: Vec<ExplainResult>,
}

#[derive(Template)]
#[template(path = "txn_detail.html")]
pub struct TxnDetailTemplate {
    pub uuid: String,
    pub explains: Vec<ExplainResult>,
}

pub struct HtmlTemplate<T>(pub T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to render template. Error: {err}")).into_response(),
        }
    }
}
