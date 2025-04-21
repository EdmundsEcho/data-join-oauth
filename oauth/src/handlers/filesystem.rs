use axum::extract::{Extension, Path, Query, TypedHeader};
use axum::http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use axum::Json;
use core::str::FromStr;
use http::Version;
use oauth2::AccessToken;
use serde::Deserialize;
// use serde::{Deserialize, Serialize};

use crate::errors::AuthError;
use crate::models::drive_clients;
use crate::models::drive_clients::{DriveClient, DriveClients};
use crate::models::drive_provider::DriveProvider;
use crate::models::files::{
    drop_box, google, ms_graph, Files, FilesBuilder, RawFileDropBox, RawFileGoogle, RawFileMSGraph,
    RawFiles,
};
use crate::models::project_id::ProjectId;

#[derive(Debug, Deserialize)]
pub struct AuthDriveToken {
    access_token: String,
}
impl From<AuthDriveToken> for AccessToken {
    fn from(AuthDriveToken { access_token }: AuthDriveToken) -> AccessToken {
        AccessToken::new(access_token)
    }
}
///
/// 🔗 filesystem endpoint
/// Use the auth code to retrieve the token.  This is a trusted, machine to machine exchange.
/// Then go ahead and retrieve the resource (user email)
///
pub(crate) async fn handle<'a>(
    Path((drive_provider, _project_id)): Path<(DriveProvider, ProjectId)>,
    Query(token): Query<AuthDriveToken>,
    TypedHeader(_cookies): TypedHeader<headers::Cookie>,
    Extension(clients): Extension<DriveClients>,
) -> Result<Json<Files>, AuthError> {
    if let Some(DriveClient { files_request, .. }) = clients.get(&drive_provider) {
        let drive_clients::FilesRequest {
            method,
            drive_server,
            endpoint,
            query_ls,
            // json_body_ls,
            ..
        } = files_request;
        /* ------------------------------------------------------------------------- */
        // 🚧 Check for a valid token; hit the session endpoint
        /* ------------------------------------------------------------------------- */
        // let token_response: BasicTokenResponse = todo!();
        /* ------------------------------------------------------------------------- */

        /* ------------------------------------------------------------------------- */
        // Fetch user data from the shared drive (protected resource)
        /* ------------------------------------------------------------------------- */
        tracing::debug!(
            "\n👉 Protected resource:\n{host}{endpoint}\n",
            host = &drive_server,
            endpoint = &endpoint,
        );
        /* ------------------------------------------------------------------------- */
        // debug
        tracing::debug!(
            "\n🔗 🦀 list files: copy/paste with Bearer\n{host}{endpoint}{query}\n",
            host = &drive_server,
            endpoint = &endpoint,
            query = &query_ls
        );

        /*
        // dropbox WIP
        let json = r#"
            {
               "path": "",
               "include_non_downloadable_files": false
            }
        "#;

        #[derive(Deserialize, Serialize)]
        struct ReqBody {
            path: String,
            include_non_downloadable_files: bool,
        }
        let req_body = serde_json::from_str::<ReqBody>(json)
            .map_err(|err| AuthError::JsonParsingError(err.into()))?;
        */

        let access_token: AccessToken = token.into();

        let client = reqwest::Client::new();
        let request = client
            .request(
                http::Method::from_str(method).unwrap(),
                format!(
                    "{host}{endpoint}{query}",
                    host = &drive_server,
                    endpoint = &endpoint,
                    query = &query_ls
                ),
            )
            .header(AUTHORIZATION, format!("Bearer {}", access_token.secret()))
            .header(CONTENT_TYPE, "application/json")
            .header(ACCEPT, "application/json")
            .version(Version::HTTP_11);
        // .json(&req_body);
        tracing::debug!("\n📬 request:\n{:#?}\n", &request);

        let response = request
            .send() // convert Request to a Future
            .await
            .map_err(|err| AuthError::InvalidResponse(err.into()))?;
        tracing::debug!("\n📥 response:\n{:#?}\n", &response);
        /* ------------------------------------------------------------------------- */
        // Extract the data from the response body
        /* ------------------------------------------------------------------------- */
        // builder initialized with parsed RawFiles<T>
        let files_builder: FilesBuilder = match response.status() {
            reqwest::StatusCode::OK => {
                let files_builder = match drive_provider {
                    DriveProvider::Google => google(
                        response
                            .json::<RawFiles<RawFileGoogle>>()
                            .await
                            .map_err(|err| {
                                let message = format!("Unexpected drive data: {}", err);
                                AuthError::JsonParsingError(message.into())
                            })?,
                    ),
                    DriveProvider::MSGraph => {
                        ms_graph(response.json::<RawFiles<RawFileMSGraph>>().await.map_err(
                            |err| {
                                let message = format!("Unexpected drive data: {}", err);
                                AuthError::JsonParsingError(message.into())
                            },
                        )?)
                    }

                    DriveProvider::DropBox => {
                        drop_box(response.json::<RawFiles<RawFileDropBox>>().await.map_err(
                            |err| {
                                let message = format!("Unexpected drive data: {}", err);
                                AuthError::JsonParsingError(message.into())
                            },
                        )?)
                    }
                    _ => return Err(AuthError::InternalError("Unsupported drive type".into())),
                };
                Ok(files_builder)
            }
            reqwest::StatusCode::UNAUTHORIZED => {
                // redirect to get a new token
                let message = format!("Unauthorized drive access");
                Err(AuthError::Unauthorized(message.into()))
            }
            // where dropbox fails
            err => Err(AuthError::InternalError(err.to_string().into())),
        }?
        .set_path("root".to_string())
        .set_drive_id("test_drive".to_string());

        let files = files_builder.build();

        // pretty print
        tracing::debug!("\n🎉 Files:\n{:#?}\n", &files);
        Ok(Json(files))
    } else {
        Err(AuthError::UnsupportedProvider(
            (&("Auth client not found")).into(),
        ))
    }
}
