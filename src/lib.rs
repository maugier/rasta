use reqwest::{Client, RequestBuilder, Url};
use anyhow::{anyhow, Result};
use serde::{Serialize, Deserialize};

static APP_USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION")
);

pub struct Rasta {
    backend: Url,
    client: Client,
    login: Option<LoginData>,
}

#[derive(Serialize, Deserialize, Debug)]
struct RoomId(String);

#[derive(Serialize)]
struct Login {
    user: String,
    password: String,
}


#[derive(Deserialize)]
pub struct Me {
    pub name: String,
    pub username: String,
}

#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
struct LoginData {
    auth_token: String,
    user_id: String,
    pub me: Me,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all="camelCase")]
pub struct Channel {
    pub name: String,
    pub t: String,
    #[serde(rename="_id")]
    room_id: RoomId,
    #[serde(default)]
    topic: String,
}

#[derive(Deserialize)]
struct LoginReply {
    data: LoginData
}

#[derive(Deserialize)]
struct ChannelReply {
    channels: Vec<Channel>
}

impl Rasta {

    pub fn new(backend: &str) -> Result<Self> {

        let backend = Url::parse(backend)?;

        let client = Client::builder()
        .user_agent(APP_USER_AGENT)
        .build()?;

        Ok(Self { backend, client, login: None })
    }

    pub async fn login(&mut self, user: String, password: String) -> Result<()> {
        let response: LoginReply = self.client.post(self.backend.join("/api/v1/login")?)
            .json(&Login { user, password })
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        let login = response.data;
        
        eprintln!("Login successful, uid={}, token={}", login.user_id, login.auth_token);
        self.login = Some(login);

        Ok(())
    }

    fn get(&self, path: &str) -> Result<RequestBuilder> {
        let url = self.backend.join(path)?;
        let login = self.login.as_ref().ok_or(anyhow!("not logged in"))?;

        Ok(self.client.get(url)
            .header("X-Auth-Token", &login.auth_token)
            .header("X-User-Id", &login.user_id))
    }

    fn post(&self, path: &str) -> Result<RequestBuilder> {
        let url = self.backend.join(path)?;
        let login = self.login.as_ref().ok_or(anyhow!("not logged in"))?;

        Ok(self.client.post(url)
            .header("X-Auth-Token", &login.auth_token)
            .header("X-User-Id", &login.user_id))
    }

    pub async fn channels(&self) -> Result<Vec<Channel>> {
        let reply: ChannelReply = self.get("/api/v1/channels.list.joined")?
            .send()
            .await?
            .json()
            .await?;

        Ok(reply.channels)

    }

    pub async fn setTopic(&self, c: &Channel, topic: &str) -> Result<() >{
        //self.client.post("/api/v1/channels.setTopic")
        todo!()
    }

}