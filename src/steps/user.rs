use crate::{
    extra_models::RootResponse,
    request_builder::RequestBuilder,
    spec::*,
    state::{Dependency, DependencyMap},
    step::*,
    types::*,
};
use chris_oag::models;
use reqwest::{Method, Request, Url};
use std::rc::Rc;

/// A [PendingStep] to make sure that a user exists. See [UserExistsStep].
#[derive(Debug, Clone)]
pub(crate) struct UserExists {
    username: Username,
    details: Rc<UserDetails>,
    url: CubeUrl,
}

impl PendingStep for UserExists {
    fn build(&self, map: &DependencyMap) -> PendingStepResult {
        debug_assert!(
            !map.contains_key(&Dependency::AuthToken(self.username.clone())),
            "Duplicate UserExists step for \"{}\"",
            &self.username
        );
        ok_step(UserExistsStep(self.clone()))
    }
}

/// A [Step] to make sure a user exists. It either:
///
/// - Obtains the user's [Dependency::AuthToken]
/// - Creates the user, producing the [Dependency::UserUrl], [Dependency::UserGroupsUrl],
///   and [Dependency::UserEmail].
pub(crate) struct UserExistsStep(UserExists);

impl Step for UserExistsStep {
    fn search(&self) -> reqwest::Request {
        let url = self.0.url.to_url().join("auth-token/").unwrap();
        let body = models::AuthTokenRequest {
            username: self.0.username.to_string(),
            password: self.0.details.password.to_string(),
        };
        Request::new(Method::POST, url)
            .json(&body)
            .unwrap()
            .accept_json()
    }

    fn deserialize(&self, body: bytes::Bytes) -> serde_json::Result<Check> {
        deserialize_auth_token(&self.0.username, body).map(Check::Exists)
    }

    fn create(&self) -> Option<Box<dyn StepRequest>> {
        Some(Box::new(CreateUserRequest::from(self.0.clone())))
    }
}

pub(crate) struct CreateUserRequest {
    url: CubeUrl,
    username: Username,
    details: Rc<UserDetails>,
}

impl From<UserExists> for CreateUserRequest {
    fn from(value: UserExists) -> Self {
        CreateUserRequest {
            username: value.username,
            url: value.url,
            details: value.details,
        }
    }
}

impl StepRequest for CreateUserRequest {
    fn request(&self) -> reqwest::Request {
        let url = self.url.to_url().join("users/").unwrap();
        let body = models::UserRequest {
            username: Some(self.username.to_string()),
            email: self
                .details
                .email
                .clone()
                .unwrap_or_else(|| format!("{}@example.org", self.username)),
            password: self.details.password.to_string(),
            is_staff: None, // very bad bad bad bad bad
        };
        Request::new(Method::POST, url)
            .json(&body)
            .unwrap()
            .accept_json()
    }

    fn deserialize(&self, body: bytes::Bytes) -> serde_json::Result<Entries> {
        deserialize_user_response(&self.username, body)
    }
}

fn deserialize_user_response(
    username: &Username,
    body: bytes::Bytes,
) -> serde_json::Result<Entries> {
    let user: models::User = serde_json::from_slice(body.as_ref())?;
    let outputs = vec![
        // (
        //     Dependency::UserId(username.clone()),
        //     user.id.to_string(),
        // ),
        (Dependency::UserUrl(username.clone()), user.url),
        (Dependency::UserEmail(username.clone()), user.email),
        (Dependency::UserGroupsUrl(username.clone()), user.groups),
    ];
    Ok(outputs)
}

/// A [PendingStep] to get a user's authorization token. See [UserGetAuthTokenStep].
#[derive(Debug, Clone)]
pub(crate) struct UserGetAuthToken {
    username: Username,
    password: String,
    url: CubeUrl,
}

impl PendingStep for UserGetAuthToken {
    fn build(&self, map: &DependencyMap) -> PendingStepResult {
        if map.contains_key(&Dependency::AuthToken(self.username.clone())) {
            return Ok(None);
        }
        debug_assert!(
            !map.contains_key(&Dependency::UserUrl(self.username.clone())),
            "UserGetAuthToken must come after UserExists for \"{}\"",
            &self.username
        );
        ok_step(UserGetAuthTokenStep(self.clone()))
    }
}

pub(crate) struct UserGetAuthTokenStep(UserGetAuthToken);

impl Step for UserGetAuthTokenStep {
    fn search(&self) -> reqwest::Request {
        let url = self.0.url.to_url().join("auth-token/").unwrap();
        let body = models::AuthTokenRequest {
            username: self.0.username.to_string(),
            password: self.0.password.to_string(),
        };
        Request::new(Method::POST, url)
            .json(&body)
            .unwrap()
            .accept_json()
    }

    fn deserialize(&self, body: bytes::Bytes) -> serde_json::Result<Check> {
        deserialize_auth_token(&self.0.username, body).map(Check::Exists)
    }
}

/// A [PendingStep] to make sure that the [DependencyMap] contains [Dependency::UserUrl].
/// See [UserGetUrlStep].
#[derive(Clone, Debug)]
pub(crate) struct UserGetUrl {
    username: Username,
    details: Rc<UserDetails>,
    url: CubeUrl,
}

impl PendingStep for UserGetUrl {
    fn build(&self, map: &DependencyMap) -> PendingStepResult {
        if map.contains_key(&Dependency::UserUrl(self.username.clone())) {
            Ok(None)
        } else if let Ok(auth_token) = map.get(Dependency::AuthToken(self.username.clone())) {
            let step = UserGetUrlStep {
                url: self.url.clone(),
                auth_token,
                username: self.username.clone(),
                details: Rc::clone(&self.details),
            };
            ok_step(step)
        } else {
            let step = UserExistsStep(UserExists {
                username: self.username.clone(),
                details: Rc::clone(&self.details),
                url: self.url.clone(),
            });
            ok_step(step)
        }
    }
}

/// A [Step] to make sure the [DependencyMap] contains a [Dependency::UserUrl] for the [Username].
/// The user will be created if necessary.
pub(crate) struct UserGetUrlStep {
    url: CubeUrl,
    username: Username,
    details: Rc<UserDetails>,
    auth_token: Rc<String>,
}

impl Step for UserGetUrlStep {
    fn search(&self) -> reqwest::Request {
        let mut url = self.url.to_url();
        url.set_query(Some("limit=1"));
        Request::new(Method::GET, url).accept_json()
    }

    fn deserialize(&self, body: bytes::Bytes) -> serde_json::Result<Check> {
        let data: RootResponse = serde_json::from_slice(&body)?;
        let outputs = [(
            Dependency::UserUrl(self.username.clone()),
            data.collection_links.user,
        )];
        Ok(Check::Exists(outputs.into()))
    }

    fn create(&self) -> Option<Box<dyn StepRequest>> {
        Some(Box::new(CreateUserRequest {
            url: self.url.clone(),
            username: self.username.clone(),
            details: Rc::clone(&self.details),
        }))
    }
}

/// A [PendingStep] to make sure that [DependencyMap] contains details of a user. See [UserGetDetailsStep].
pub(crate) struct UserGetDetails {
    url: CubeUrl,
    username: Username,
    details: Rc<UserDetails>,
}

impl PendingStep for UserGetDetails {
    fn build(&self, map: &DependencyMap) -> PendingStepResult {
        let user_url = map.get(Dependency::UserUrl(self.username.clone()));
        if user_url.is_ok()
            && map.contains_key(&Dependency::UserGroupsUrl(self.username.clone()))
            && map.contains_key(&Dependency::UserEmail(self.username.clone()))
        {
            Ok(None)
        } else {
            ok_step(UserGetDetailsStep {
                url: self.url.clone(),
                username: self.username.clone(),
                details: Rc::clone(&self.details),
                user_url: user_url?,
                auth_token: map.get(Dependency::AuthToken(self.username.clone()))?,
            })
        }
    }
}

/// A [PendingStep] to make sure that [DependencyMap] contains the following keys for the [Username]:
/// [Dependency::UserUrl], [Dependency::UserGroupsUrl], [Dependency::UserEmail].
/// The user will be created if necessary.
pub(crate) struct UserGetDetailsStep {
    url: CubeUrl,
    username: Username,
    details: Rc<UserDetails>,
    user_url: Rc<String>,
    auth_token: Rc<String>,
}

impl Step for UserGetDetailsStep {
    fn search(&self) -> reqwest::Request {
        let url = Url::parse(&self.user_url).unwrap();
        Request::new(Method::GET, url)
            .auth_token(self.auth_token.as_str())
            .accept_json()
    }

    fn deserialize(&self, body: bytes::Bytes) -> serde_json::Result<Check> {
        deserialize_user_response(&self.username, body).map(Check::Exists)
    }

    fn create(&self) -> Option<Box<dyn StepRequest>> {
        Some(Box::new(CreateUserRequest {
            url: self.url.clone(),
            username: self.username.clone(),
            details: Rc::clone(&self.details),
        }))
    }
}

/// A [PendingStep] to set the user's email. See [UserSetEmailStep].
#[derive(Clone, Debug)]
pub(crate) struct UserSetEmail {
    username: Username,
    details: Rc<UserDetails>,
}

impl PendingStep for UserSetEmail {
    fn build(&self, map: &DependencyMap) -> PendingStepResult {
        let current_email = map.get(Dependency::UserEmail(self.username.clone()))?;
        if let Some(desired_email) = self.details.email.as_ref()
            && current_email.as_ref() != desired_email
        {
            let step = UserSetEmailStep {
                user_url: map.get(Dependency::UserUrl(self.username.clone()))?,
                auth_token: map.get(Dependency::AuthToken(self.username.clone()))?,
                username: self.username.clone(),
                password: self.details.password.clone(),
                email: desired_email.clone(),
            };
            ok_step(step)
        } else {
            Ok(None)
        }
    }
}

fn deserialize_auth_token(
    username: &Username,
    body: impl AsRef<[u8]>,
) -> serde_json::Result<Entries> {
    let body: models::AuthToken = serde_json::from_slice(body.as_ref())?;
    let value = format!("Token {}", body.token);
    let outputs = vec![(Dependency::AuthToken(username.clone()), value)];
    Ok(outputs)
}

/// A [Step] to set the user's email (of a user which already exists).
#[derive(Debug, Clone)]
pub(crate) struct UserSetEmailStep {
    user_url: Rc<String>,
    auth_token: Rc<String>,
    username: Username,
    password: String,
    email: String,
}

impl Step for UserSetEmailStep {
    fn search(&self) -> reqwest::Request {
        let url = Url::parse(&self.user_url).unwrap();
        let body = models::UserRequest {
            username: None,
            email: self.email.to_string(),
            password: self.password.to_string(),
            is_staff: None, // very bad bad bad bad bad
        };
        Request::new(Method::PUT, url)
            .auth_token(self.auth_token.as_str())
            .json(&body)
            .unwrap()
            .accept_json()
    }

    fn deserialize(&self, body: bytes::Bytes) -> serde_json::Result<Check> {
        deserialize_user_response(&self.username, body).map(Check::Modified)
    }
}
