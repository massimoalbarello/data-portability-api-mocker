use config::ConfigError;
use oauth2::{AuthUrl, ClientId, ClientSecret, RedirectUrl, RevocationUrl, Scope, TokenUrl};
use secrecy::{ExposeSecret, Secret};
use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgConnectOptions, PgSslMode};
use std::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Deserialize, Clone, Debug)]
pub struct ServerSettings {
    pub host: String,
    pub port: u16,
    pub refresher_poll_interval_millis: u64,
    pub refresher_refresh_interval_secs: u64,
}

#[derive(Deserialize, Clone, Debug)]
pub struct WorkerSettings {
    pub request_timeout_millis: u64,
    pub polling_interval_millis: u64,
    pub loop_interval_millis: u64,
    pub max_tries: u8,
    pub max_polling_iterations: u8,
}

#[derive(Deserialize, Clone, Debug)]
pub struct AwsSettings {
    pub region: String,
    pub bucket_name: String,
    /// If provided, the S3 client will use this test endpoint URL instead of the default one
    pub test_s3_endpoint_url: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct GoogleSettings {
    pub client_id: ClientId,
    pub client_secret: ClientSecret,
    /// URI of the frontend callback component triggered by Google after the OAuth flow terminates
    pub redirect_uri: RedirectUrl,
    pub auth_uri: AuthUrl,
    /// endpoint called by the backend to convert an authorization code into an access token
    pub access_token_uri: TokenUrl,
    /// uri called by the backend to reset the authorization of a user
    pub reset_authorization_uri: RevocationUrl,
    /// base uri called by the backend to initiate and read the state of an archive job for a resource
    pub archive_base_uri: String,
    pub initiate_archive_endpoint: String,
    /// endpoint called by the backend to initiate an archive job for a resource
    pub archive_jobs_path: String,
    /// endpoint called by the backend to read the state of an archive job for a resource
    pub poll_archive_state_endpoint: String,
}

impl AsRef<GoogleSettings> for GoogleSettings {
    fn as_ref(&self) -> &GoogleSettings {
        self
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct PinterestSettings {
    pub app_id: ClientId,
    pub app_secret_key: ClientSecret,
    pub redirect_uri: RedirectUrl,
    pub auth_uri: AuthUrl,
    pub access_token_uri: TokenUrl,
    pub revocation_uri: RevocationUrl,
    pub resource_base_uri: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct TikTokSettings {
    pub client_key: ClientId,
    pub client_secret: ClientSecret,
    pub redirect_uri: RedirectUrl,
    pub auth_uri: AuthUrl,
    pub access_token_uri: TokenUrl,
    pub revocation_uri: RevocationUrl,
    pub data_base_uri: String,
    pub add_data_endpoint: String,
    pub check_status_endpoint: String,
    pub download_data_endpoint: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct LinkedinSettings {
    pub client_id: ClientId,
    pub client_secret: ClientSecret,
    pub redirect_uri: RedirectUrl,
    pub auth_uri: AuthUrl,
    pub access_token_uri: TokenUrl,
    pub revocation_uri: RevocationUrl,
    pub query_snapshot_uri: String,
    pub query_changelog_uri: String,
    pub api_version: String,
    pub changelog_pagination_size: usize,
}

#[derive(Deserialize, Clone, Debug)]
pub struct PostgresSettings {
    pub username: String,
    pub password: Secret<String>,
    pub host: String,
    pub port: u16,
    pub database_name: String,
    pub max_connections: u32,
}

impl PostgresSettings {
    /// Returns the connection URL without the database name
    /// Used for connecting to the Postgres instance instead of a specific logical database
    /// Needed during integration testing as we spin up a new logical database for each test
    /// so that the tests do not interfere with each other
    pub fn without_db(&self) -> PgConnectOptions {
        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(self.password.expose_secret())
            .port(self.port)
            .ssl_mode(PgSslMode::Prefer)
    }

    pub fn with_db(&self) -> PgConnectOptions {
        self.without_db().database(&self.database_name)
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct SentrySettings {
    pub dsn: Option<String>,
    // Set the stack as the environment
    // Used to differentiate the traces in Sentry
    pub environment: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Settings {
    pub server: ServerSettings,
    pub worker: WorkerSettings,
    pub aws: AwsSettings,
    pub google: GoogleSettings,
    pub tiktok: TikTokSettings,
    pub linkedin: LinkedinSettings,
    pub pinterest: PinterestSettings,
    pub postgres: PostgresSettings,
    pub sentry: SentrySettings,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let base_path = std::env::current_dir().expect("Failed to determine the current directory");
        let config_dir = base_path.join("configuration");

        let settings: Settings = config::Config::builder()
            .add_source(config::File::from(config_dir.join("base.yml")))
            .add_source(config::File::from(config_dir.join("load.yml")))
            .build()?
            .try_deserialize()?;

        Ok(settings)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum Resource {
    Google(GoogleResource),
}

impl AsRef<str> for Resource {
    fn as_ref(&self) -> &str {
        match self {
            Resource::Google(resource) => resource.as_ref(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum GoogleResource {
    MyActivityYoutube,
    MyActivitySearch,
    MyActivityShopping,
    MyActivityPlay,
    MyActivityMyAdCenter,
    MyActivityMaps,
    YouTubeChannel,
}

impl TryFrom<&Scope> for GoogleResource {
    type Error = String;

    fn try_from(scope: &Scope) -> Result<Self, Self::Error> {
        match scope.as_str() {
            "https://www.googleapis.com/auth/dataportability.myactivity.youtube" => {
                Ok(GoogleResource::MyActivityYoutube)
            }
            "https://www.googleapis.com/auth/dataportability.myactivity.search" => {
                Ok(GoogleResource::MyActivitySearch)
            }
            "https://www.googleapis.com/auth/dataportability.myactivity.shopping" => {
                Ok(GoogleResource::MyActivityShopping)
            }
            "https://www.googleapis.com/auth/dataportability.myactivity.play" => {
                Ok(GoogleResource::MyActivityPlay)
            }
            "https://www.googleapis.com/auth/dataportability.myactivity.myadcenter" => {
                Ok(GoogleResource::MyActivityMyAdCenter)
            }
            "https://www.googleapis.com/auth/dataportability.myactivity.maps" => {
                Ok(GoogleResource::MyActivityMaps)
            }
            "https://www.googleapis.com/auth/dataportability.youtube.channel" => {
                Ok(GoogleResource::YouTubeChannel)
            }
            _ => Err(format!(
                "Unsupported scope: {:?} for Google provider",
                scope,
            )),
        }
    }
}

impl TryFrom<&str> for GoogleResource {
    type Error = String;

    fn try_from(resource: &str) -> Result<Self, Self::Error> {
        match resource {
            "myactivity.youtube" => Ok(GoogleResource::MyActivityYoutube),
            "myactivity.search" => Ok(GoogleResource::MyActivitySearch),
            "myactivity.shopping" => Ok(GoogleResource::MyActivityShopping),
            "myactivity.play" => Ok(GoogleResource::MyActivityPlay),
            "myactivity.myadcenter" => Ok(GoogleResource::MyActivityMyAdCenter),
            "myactivity.maps" => Ok(GoogleResource::MyActivityMaps),
            "youtube.channel" => Ok(GoogleResource::YouTubeChannel),
            _ => Err(format!(
                "Unsupported resource: {:?} for Google provider",
                resource,
            )),
        }
    }
}

impl AsRef<str> for GoogleResource {
    fn as_ref(&self) -> &str {
        match self {
            GoogleResource::MyActivityYoutube => "myactivity.youtube",
            GoogleResource::MyActivitySearch => "myactivity.search",
            GoogleResource::MyActivityShopping => "myactivity.shopping",
            GoogleResource::MyActivityPlay => "myactivity.play",
            GoogleResource::MyActivityMyAdCenter => "myactivity.myadcenter",
            GoogleResource::MyActivityMaps => "myactivity.maps",
            GoogleResource::YouTubeChannel => "youtube.channel",
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum GoogleArchiveAccessType {
    #[serde(rename = "ACCESS_TYPE_ONE_TIME")]
    OneTime,
    #[serde(rename = "ACCESS_TYPE_TIME_BASED")]
    TimeBased,
}

impl std::fmt::Display for GoogleArchiveAccessType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GoogleArchiveAccessType::OneTime => write!(f, "ACCESS_TYPE_ONE_TIME"),
            GoogleArchiveAccessType::TimeBased => write!(f, "ACCESS_TYPE_TIME_BASED"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GoogleInitiateArchiveResponsePayload {
    #[serde(rename = "archiveJobId")]
    archive_job_id: JobId,
    #[serde(
        rename = "accessType",
        default = "GoogleInitiateArchiveResponsePayload::default_access_type"
    )]
    access_type: GoogleArchiveAccessType,
}

impl GoogleInitiateArchiveResponsePayload {
    pub fn mock(resource: &Resource, access_type: Option<GoogleArchiveAccessType>) -> Self {
        Self {
            // appended 'resource' to make job id different for each resource and also deterministic
            // needed when mocking the provider APIs
            archive_job_id: JobId::from((resource.as_ref().to_string() + "_test_job_id").as_str()),
            // if the response does not include access_type, serde is set to default to 'GoogleArchiveAccessType::OneTime'
            access_type: access_type.unwrap_or(GoogleArchiveAccessType::OneTime),
        }
    }

    fn default_access_type() -> GoogleArchiveAccessType {
        GoogleArchiveAccessType::OneTime
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct JobId(String);

impl From<&str> for JobId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

impl Display for JobId {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.0)
    }
}

impl AsRef<String> for JobId {
    fn as_ref(&self) -> &String {
        &self.0
    }
}
