//! CUBE API models which were not produced by OpenAPI-Generator.

use chris_oag::models;

#[derive(serde::Deserialize)]
pub(crate) struct CollectionLinks {
    // chrisinstance: String,
    // public_feeds: String,
    // compute_resouces: String,
    // plugin_metas: String,
    pub user: String,
}

#[derive(serde::Deserialize)]
pub(crate) struct RootResponse {
    #[serde(flatten)]
    pub list: models::PaginatedFeedList,
    pub collection_links: CollectionLinks,
}
