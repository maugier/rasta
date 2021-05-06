use reqwest::{Method, RequestBuilder};
use anyhow::{Result, anyhow};
use serde::Deserialize;
use log::debug;

use crate::{Credentials, schema::{Room, ShortUser}};

#[derive(Clone,Debug)]
struct Login {
    user_id: String,
    token: String,
}

#[derive(Clone,Debug)]
pub struct Client {
    base_url: String,
    client: reqwest::Client,
    login: Option<Login>,
}


impl Client {

    pub fn token(&self) -> Option<Credentials> {
        self.login.as_ref().map(|l| Credentials::Token(l.token.clone()))
    }

    fn request(&self, method: Method, url: &str) -> RequestBuilder {
        let url = self.base_url.clone() + url;

        let req = self.client.request(method, url);

        if let Some(login) = &self.login {
            req.header("X-User-Id", &login.user_id)
               .header("X-Auth-Token", &login.token)
        } else {
            req
        }

    }

    pub fn new(host: &str) -> Self {

        let base_url = format!("https://{}/api/", host);
        let client = reqwest::Client::new();
        Self { base_url, client, login: None }

    }

    pub async fn login(&mut self, creds: &Credentials) -> Result<Credentials> {

        #[derive(Deserialize)]
        #[serde(tag="status", content="data")]
        #[serde(rename_all="camelCase")]
        enum LoginResult {
            #[serde(rename_all="camelCase")]
            Success { auth_token: String, user_id: String },
            Error {},
        }

        let req = self.request(Method::POST, "v1/login");

        let req = match creds {
            Credentials::Clear { user, password } => { 
                req.form(&[("user", user), ("password", password)])
            },
            Credentials::Token(tok) => {
                req.form(&[("resume", tok)])
            }
        };

        debug!("Sending login request {:?}", req);
        let reply: LoginResult = req.send()
                       .await?
                       .json()
                       .await?;

        match reply {
            LoginResult::Success { auth_token, user_id } => {
                self.login = Some( Login { user_id, token: auth_token.clone() });
                Ok( Credentials::Token(auth_token) )
            },
            LoginResult::Error {} => {
                Err(anyhow!("Login failed"))
            }
        }

    }

    pub async fn channel_members(&self, room: &Room) -> Result<Vec<ShortUser>> {

        #[derive(Deserialize)]
        struct Response { members: Vec<ShortUser> }

        let endpoint = match room {
            Room::Chat{..} => "v1/channels.members",
            Room::Private{..} => "v1/groups.members",
            _ => return Ok(vec![]),
        };

        Ok(self.request(Method::GET, endpoint)
               .query(&[("roomId", room.id())])
               .send()
               .await?
               .json::<Response>()
               .await?
               .members)
    }

}