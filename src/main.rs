use httpmock::{
    Method::{GET, POST},
    MockServer,
};
use oauth2::{
    basic::BasicTokenType, AccessToken, EmptyExtraTokenFields, Scope, StandardTokenResponse,
};
use serde_json::json;
use std::{net::SocketAddr, time::Duration};
use types::{
    GoogleArchiveAccessType, GoogleInitiateArchiveResponsePayload, GoogleResource, Resource,
    Settings,
};

mod types;

pub struct GoogleMockAuthServer {
    server: MockServer,
    backend_settings: Settings,
}

impl GoogleMockAuthServer {
    pub async fn new(backend_settings: Settings) -> Self {
        // take the address of the server from load.yml and remove the "http://" prefix
        let server = MockServer::connect_async(
            backend_settings
                .google
                .archive_base_uri
                .split("http://")
                .last()
                .unwrap(),
        )
        .await;

        Self {
            server,
            backend_settings,
        }
    }

    pub fn address(&self) -> &SocketAddr {
        &self.server.address()
    }

    pub async fn mock_convert_authorization_code_to_access_token(&self) {
        let mut mock_response_payload = StandardTokenResponse::new(
            AccessToken::new("test_access_token".to_string()),
            BasicTokenType::Bearer,
            EmptyExtraTokenFields {},
        );
        mock_response_payload.set_scopes(Some(vec![
            Scope::new(
                "https://www.googleapis.com/auth/dataportability.myactivity.search".to_string(),
            ),
            Scope::new(
                "https://www.googleapis.com/auth/dataportability.myactivity.youtube".to_string(),
            ),
            Scope::new(
                "https://www.googleapis.com/auth/dataportability.myactivity.shopping".to_string(),
            ),
            Scope::new(
                "https://www.googleapis.com/auth/dataportability.myactivity.myadcenter".to_string(),
            ),
            Scope::new(
                "https://www.googleapis.com/auth/dataportability.myactivity.play".to_string(),
            ),
            Scope::new(
                "https://www.googleapis.com/auth/dataportability.myactivity.maps".to_string(),
            ),
            Scope::new(
                "https://www.googleapis.com/auth/dataportability.youtube.channel".to_string(),
            ),
        ]));
        mock_response_payload.set_expires_in(Some(&Duration::from_secs(3600)));

        self.server
            .mock_async(|when, then| {
                when.method(POST).path(
                    "/".to_string()
                        + self
                            .backend_settings
                            .google
                            .access_token_uri
                            .split('/')
                            .last()
                            .unwrap(),
                );
                then.status(200)
                    .json_body(serde_json::to_value(&mock_response_payload).unwrap());
            })
            .await;
    }

    pub async fn mock_initiate_data_archive(&self) {
        let mock_response_payload = GoogleInitiateArchiveResponsePayload::mock(
            &Resource::Google(GoogleResource::MyActivitySearch),
            Some(GoogleArchiveAccessType::OneTime),
        );

        self.server
            .mock_async(|when, then| {
                when.method(POST)
                    .path(&self.backend_settings.google.initiate_archive_endpoint);
                then.status(200)
                    .json_body(serde_json::to_value(&mock_response_payload).unwrap());
            })
            .await;
    }

    pub async fn mock_get_archive_status(&self) {
        self.server
        .mock_async(|when, then| {
            when.method(GET)
                .path_contains(&self.backend_settings.google.archive_jobs_path);
            then.status(200)
                .json_body(json!({
                    "state": "COMPLETED",
                    "urls": vec![format!("http://{}/download_endpoint/some_download_id", self.address().to_string())]
                }))
                .delay(Duration::from_secs(0));
        })
        .await;
    }

    pub async fn mock_download_archive(&self) {
        self.server
            .mock_async(|when, then| {
                when.method(GET).path("/download_endpoint/some_download_id");
                then.status(200)
                    .header("Content-Type", "application/zip")
                    .body_from_file("Portability.zip")
                    .delay(Duration::from_secs(0));
            })
            .await;
    }

    pub async fn mock_revoke_token(&self) {
        self.server
            .mock_async(|when, then| {
                when.method(POST).path(
                    "/".to_string()
                        + self
                            .backend_settings
                            .google
                            .reset_authorization_uri
                            .split('/')
                            .last()
                            .unwrap(),
                );
                then.status(200);
            })
            .await;
    }
}

#[tokio::main]
async fn main() {
    let backend_settings = Settings::new().unwrap();
    let server = GoogleMockAuthServer::new(backend_settings).await;

    server
        .mock_convert_authorization_code_to_access_token()
        .await;

    server.mock_initiate_data_archive().await;

    server.mock_get_archive_status().await;

    server.mock_download_archive().await;

    server.mock_revoke_token().await;
}
