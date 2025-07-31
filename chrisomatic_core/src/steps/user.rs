use crate::types::*;
use crate::{extra_models::RootResponse, request_builder::RequestBuilder};
use chris_oag::models;
use chrisomatic_spec::*;
use reqwest::{Method, Request, StatusCode, Url};
use std::rc::Rc;

/// A [Step] to try getting an auth token for a user who may or may not exist.
#[derive(Debug, Clone)]
pub(crate) struct UserTryGetAuthToken {
    pub(crate) username: Username,
    pub(crate) details: Rc<UserDetails>,
    pub(crate) url: CubeUrl,
}

impl Step for UserTryGetAuthToken {
    fn request(&self, map: &dyn DependencyMap) -> Result<Option<Request>, Dependency> {
        debug_assert!(
            !map.contains_key(&Dependency::AuthToken(self.username.clone())),
            "Duplicate step for \"{}\"",
            &self.username
        );
        let url = self.url.to_url().join("auth-token/").unwrap();
        let body = models::AuthTokenRequest {
            username: self.username.to_string(),
            password: self.details.password.to_string(),
        };
        let req = Request::new(Method::POST, url)
            .json(&body)
            .unwrap()
            .accept_json();
        Ok(Some(req))
    }

    fn check_status(&self, status: reqwest::StatusCode) -> bool {
        // BAD_REQUEST means user might not exist yet. No worries, a
        // subsequent step will try to create the user.
        status == StatusCode::BAD_REQUEST || status.is_success()
    }

    fn deserialize(&self, body: bytes::Bytes) -> serde_json::Result<Entries> {
        deserialize_auth_token(&self.username, body)
    }

    fn provides(&self) -> Vec<Dependency> {
        vec![Dependency::AuthToken(self.username.clone())]
    }

    fn affects(&self) -> Resource {
        Resource::User(self.username.clone())
    }

    fn effect(&self) -> StepEffect {
        StepEffect::Unmodified
    }
}

/// A [Step] to create a user.
#[derive(Clone, Debug)]
pub(crate) struct UserCreateStep {
    url: CubeUrl,
    username: Username,
    details: Rc<UserDetails>,
}

impl Step for UserCreateStep {
    fn request(&self, map: &dyn DependencyMap) -> Result<Option<reqwest::Request>, Dependency> {
        let url = self.url.to_url().join("users/").unwrap();
        let body = models::UserRequest {
            username: Some(self.username.to_string()),
            email: self.details.email.to_string(),
            password: self.details.password.to_string(),
            is_staff: None, // very bad bad bad bad bad
        };
        let req = Request::new(Method::POST, url)
            .json(&body)
            .unwrap()
            .accept_json();
        Ok(Some(req))
    }

    fn deserialize(&self, body: bytes::Bytes) -> serde_json::Result<Entries> {
        deserialize_user_response(body)
    }

    fn provides(&self) -> Vec<Dependency> {
        vec![
            (Dependency::UserUrl(self.username.clone())),
            (Dependency::UserEmail(self.username.clone())),
            (Dependency::UserGroupsUrl(self.username.clone())),
        ]
    }

    fn affects(&self) -> Resource {
        Resource::User(self.username.clone())
    }

    fn effect(&self) -> StepEffect {
        StepEffect::Created
    }
}

fn deserialize_user_response(body: impl AsRef<[u8]>) -> serde_json::Result<Entries> {
    let user: models::User = serde_json::from_slice(body.as_ref())?;
    let username = user
        .username
        .map(Username::from)
        // https://github.com/FNNDSC/ChRIS_ultron_backEnd/issues/645
        .expect("User response does not contain username");
    let outputs = vec![
        (Dependency::UserUrl(username.clone()), user.url),
        (Dependency::UserEmail(username.clone()), user.email),
        (Dependency::UserGroupsUrl(username.clone()), user.groups),
    ];
    Ok(outputs)
}

/// A [PendingStep] to get a user's authorization token. See [UserGetAuthTokenStep].
#[derive(Debug, Clone)]
pub(crate) struct UserGetAuthToken {
    pub(crate) username: Username,
    pub(crate) password: String,
    pub(crate) url: CubeUrl,
}

impl Step for UserGetAuthToken {
    fn request(&self, map: &dyn DependencyMap) -> Result<Option<reqwest::Request>, Dependency> {
        if map.contains_key(&Dependency::AuthToken(self.username.clone())) {
            return Ok(None);
        }
        let url = self.url.to_url().join("auth-token/").unwrap();
        let body = models::AuthTokenRequest {
            username: self.username.to_string(),
            password: self.password.to_string(),
        };
        let req = Request::new(Method::POST, url)
            .json(&body)
            .unwrap()
            .accept_json();
        Ok(Some(req))
    }

    fn deserialize(&self, body: bytes::Bytes) -> serde_json::Result<Entries> {
        deserialize_auth_token(&self.username, body)
    }

    fn provides(&self) -> Vec<Dependency> {
        vec![Dependency::AuthToken(self.username.clone())]
    }

    fn affects(&self) -> Resource {
        Resource::User(self.username.clone())
    }

    fn effect(&self) -> StepEffect {
        StepEffect::Unmodified
    }
}

#[derive(Clone, Debug)]
pub(crate) struct UserGetUrl {
    pub(crate) username: Username,
    pub(crate) details: Rc<UserDetails>,
    pub(crate) url: CubeUrl,
}

impl Step for UserGetUrl {
    fn request(&self, map: &dyn DependencyMap) -> Result<Option<reqwest::Request>, Dependency> {
        if map.contains_key(&Dependency::UserUrl(self.username.clone())) {
            return Ok(None);
        }
        let auth_token = map.get(Dependency::AuthToken(self.username.clone()))?;
        let mut url = self.url.to_url();
        url.set_query(Some("limit=1"));
        let req = Request::new(Method::GET, url)
            .auth_token(auth_token.as_str())
            .accept_json();
        Ok(Some(req))
    }

    fn deserialize(&self, body: bytes::Bytes) -> serde_json::Result<Entries> {
        let data: RootResponse = serde_json::from_slice(&body)?;
        Ok(vec![(
            Dependency::UserUrl(self.username.clone()),
            data.collection_links.user,
        )])
    }

    fn provides(&self) -> Vec<Dependency> {
        vec![Dependency::UserUrl(self.username.clone())]
    }

    fn affects(&self) -> Resource {
        Resource::User(self.username.clone())
    }

    fn effect(&self) -> StepEffect {
        StepEffect::Unmodified
    }
}

#[derive(Clone, Debug)]
pub(crate) struct UserGetDetails {
    pub(crate) url: CubeUrl,
    pub(crate) username: Username,
    pub(crate) details: Rc<UserDetails>,
}

impl Step for UserGetDetails {
    fn request(&self, map: &dyn DependencyMap) -> Result<Option<reqwest::Request>, Dependency> {
        let user_url = map.get(Dependency::UserUrl(self.username.clone()));
        if user_url.is_ok()
            && map.contains_key(&Dependency::UserGroupsUrl(self.username.clone()))
            && map.contains_key(&Dependency::UserEmail(self.username.clone()))
        {
            return Ok(None);
        }
        let auth_token = map.get(Dependency::AuthToken(self.username.clone()))?;
        let url = Url::parse(user_url?.as_str()).unwrap();
        let req = Request::new(Method::GET, url)
            .auth_token(auth_token.as_str())
            .accept_json();
        Ok(Some(req))
    }

    fn affects(&self) -> Resource {
        Resource::User(self.username.clone())
    }

    fn effect(&self) -> StepEffect {
        StepEffect::Unmodified
    }

    fn deserialize(&self, body: bytes::Bytes) -> serde_json::Result<Entries> {
        deserialize_user_response(body)
    }

    fn provides(&self) -> Vec<Dependency> {
        vec![
            Dependency::UserUrl(self.username.clone()),
            Dependency::UserGroupsUrl(self.username.clone()),
            Dependency::UserEmail(self.username.clone()),
        ]
    }
}

#[derive(Clone, Debug)]
pub(crate) struct UserSetDetails {
    pub(crate) username: Username,
    pub(crate) details: Rc<UserDetails>,
}

impl Step for UserSetDetails {
    fn request(&self, map: &dyn DependencyMap) -> Result<Option<reqwest::Request>, Dependency> {
        let user_url = map.get(Dependency::UserUrl(self.username.clone()))?;
        let auth_token = map.get(Dependency::AuthToken(self.username.clone()))?;
        let prev_email = map.get(Dependency::UserEmail(self.username.clone()))?;
        if prev_email.as_str() == &self.details.email {
            return Ok(None);
        }
        let url = Url::parse(user_url.as_str()).unwrap();
        let body = models::UserRequest {
            username: None,
            email: self.details.email.to_string(),
            password: self.details.password.to_string(),
            is_staff: None, // very bad bad bad bad bad
        };
        let req = Request::new(Method::PUT, url)
            .auth_token(auth_token.as_str())
            .json(&body)
            .unwrap()
            .accept_json();
        Ok(Some(req))
    }

    fn affects(&self) -> Resource {
        Resource::User(self.username.clone())
    }

    fn effect(&self) -> StepEffect {
        StepEffect::Modified
    }

    fn deserialize(&self, body: bytes::Bytes) -> serde_json::Result<Entries> {
        deserialize_user_response(body)
    }

    fn provides(&self) -> Vec<Dependency> {
        vec![
            (Dependency::UserUrl(self.username.clone())),
            (Dependency::UserEmail(self.username.clone())),
            (Dependency::UserGroupsUrl(self.username.clone())),
        ]
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
