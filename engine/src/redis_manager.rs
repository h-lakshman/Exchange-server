use engine::types::{DbMessage, MessageFromApi, MessageToApi, WsMessage};
use redis::{Client, Commands, Connection};
use std::{env, error::Error, sync::OnceLock};

pub static REDIS_MANAGER: OnceLock<RedisManager> = OnceLock::new();

pub struct RedisManager {
    pub client: Client,
}

impl RedisManager {
    fn new() -> Result<Self, Box<dyn Error>> {
        let redis_url = env::var("REDIS_URL")?;
        let client = Client::open(redis_url.clone())?;
        Ok(RedisManager { client })
    }
    pub fn get_instance() -> &'static RedisManager {
        REDIS_MANAGER.get_or_init(|| {
            RedisManager::new()
                .expect("Failed to create RedisManager instance or REDIS_URL is not set")
        })
    }

    pub fn get_message(&self) -> Result<(String, MessageFromApi), Box<dyn Error>> {
        let mut connection = self.client.get_connection()?;
        let (_, payload): (String, String) = connection.brpop("messages".to_string(), 0)?;
        println!("Received raw message from queue: {}", payload);
        let (client_id, message): (String, MessageFromApi) = serde_json::from_str(&payload)?;
        println!("Deserialized message: {:?}", message);
        Ok((client_id, message))
    }

    pub fn push_message(&self, message: DbMessage) -> Result<(), Box<dyn Error>> {
        let payload = serde_json::to_string(&message)?;
        let mut connection = self.client.get_connection()?;
        let _: () = connection.lpush("db_processor", payload)?;
        Ok(())
    }

    pub fn send_to_api(
        &self,
        channel: String,
        message: MessageToApi,
    ) -> Result<(), Box<dyn Error>> {
        let payload = serde_json::to_string(&message)?;
        let mut connection = self.client.get_connection()?;
        let _: () = connection.publish(channel, payload)?;
        Ok(())
    }

    pub fn publish_message(
        &self,
        channel: String,
        message: WsMessage,
    ) -> Result<(), Box<dyn Error>> {
        let payload = serde_json::to_string(&message)?;
        let mut connection = self.client.get_connection()?;
        let _: () = connection.publish(channel, payload)?;
        Ok(())
    }
}
