use std::{collections::HashMap, sync::Arc};

use axum::Json;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::middleware::base_url::BaseUrl;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Link {
    #[serde(rename = "rel")]
    pub relation: String,

    #[serde(rename = "href")]
    pub hypertext_reference: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Response<T> {
    #[serde(flatten)]
    pub content: T,
    pub debug_info: HashMap<String, Value>,
    pub links: Vec<Link>,
}

impl<T> Response<T> {
    pub fn new(content: T) -> Self {
        Self {
            content,
            debug_info: HashMap::new(),
            links: vec![],
        }
    }

    pub fn builder(content: T, base_url: Arc<BaseUrl>) -> ResponseBuilder<T> {
        ResponseBuilder::new(content, base_url)
    }

    pub fn json(self) -> Json<Self> {
        Json(self)
    }
}

pub struct ResponseBuilder<T> {
    pub response: Response<T>,
    pub base_url: Arc<BaseUrl>,
}

impl<T> ResponseBuilder<T> {
    pub fn new(content: T, base_url: Arc<BaseUrl>) -> Self {
        Self {
            response: Response::new(content),
            base_url,
        }
    }

    pub fn debug_info<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Serialize,
    {
        self.response
            .debug_info
            .insert(key.into(), serde_json::to_value(value).unwrap());
        self
    }

    pub fn debug_info_option<K, V>(self, key: K, value: Option<V>) -> Self
    where
        K: Into<String>,
        V: Serialize,
    {
        match value {
            Some(v) => self.debug_info(key, v),
            None => self,
        }
    }

    pub fn link<R, H>(self, relation: R, hypertext_reference: H) -> Self
    where
        R: Into<String>,
        H: Into<String>,
    {
        let url = self.base_url.full_url(hypertext_reference);
        self.link_extern(relation, url)
    }

    pub fn link_option<R, H>(
        self,
        relation: R,
        hypertext_reference: Option<H>,
    ) -> Self
    where
        R: Into<String>,
        H: Into<String>,
    {
        let url = hypertext_reference.map(|path| self.base_url.full_url(path));
        self.link_extern_option(relation, url)
    }

    pub fn link_extern<R, H>(mut self, relation: R, hypertext_reference: H) -> Self
    where
        R: Into<String>,
        H: Into<String>,
    {
        self.response.links.push(Link {
            relation: relation.into(),
            hypertext_reference: hypertext_reference.into(),
        });
        self
    }

    pub fn link_extern_option<R, H>(
        self,
        relation: R,
        hypertext_reference: Option<H>,
    ) -> Self
    where
        R: Into<String>,
        H: Into<String>,
    {
        if let Some(href) = hypertext_reference {
            self.link_extern(relation, href)
        } else {
            self
        }
    }

    pub fn build(self) -> Response<T> {
        self.response
    }
}
