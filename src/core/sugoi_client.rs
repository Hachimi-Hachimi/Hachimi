use std::sync::Arc;

use once_cell::sync::Lazy;
use serde::Serialize;

use super::{Error, Hachimi};

pub struct SugoiClient {
    agent: ureq::Agent,
    url: String
}

static INSTANCE: Lazy<Arc<SugoiClient>> = Lazy::new(|| {
    Arc::new(SugoiClient {
        agent: ureq::Agent::new(),
        url: Hachimi::instance().config.load().sugoi_url.as_ref()
            .map(|s| s.clone())
            .unwrap_or_else(|| "http://127.0.0.1:14366".to_owned())
    })
});

impl SugoiClient {
    pub fn instance() -> Arc<Self> {
        INSTANCE.clone()
    }

    pub fn translate(&self, content: &[String]) -> Result<Vec<String>, Error> {
        Ok(self.agent.post(&self.url)
            .set("Content-Type", "application/json")
            .send_json(Message::TranslateSentences { content })?
            .into_json()?
        )
    }

    pub fn translate_one(&self, content: String) -> Result<String, Error> {
        let mut res = self.translate(&[content])?;
        if res.len() != 1 {
            return Err(Error::RuntimeError("Server returned invalid amount of translated content".to_owned()));
        }
        Ok(res.pop().unwrap())
    }
}

#[derive(Serialize)]
#[serde(tag = "message")]
enum Message<'a> {
    #[serde(rename = "translate sentences")]
    TranslateSentences {
        content: &'a [String]
    }
}