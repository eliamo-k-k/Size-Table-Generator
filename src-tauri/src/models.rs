use melrose_types::{ItemCode, SizeCode};
use serde::Serialize;

#[derive(Serialize)]
pub struct ProcessResponse {
  pub item_meta: Vec<ItemMeta>,
}

#[derive(Debug, Serialize)]
pub struct ItemTable {
  pub head: Vec<String>,
  pub body: Vec<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct ItemMeta {
  pub code: String,
  pub size_code: String,
  pub table: ItemTable,
}

#[derive(Serialize, Clone)]
pub struct ProcessingStatePayload {
  pub state: String,
}

#[derive(Clone)]
pub struct SizeDetail {
  pub name: String,
  pub value: String,
}

#[derive(Clone)]
pub struct SizeDetails(pub Vec<SizeDetail>);

pub struct ItemInfo {
  pub item_code: ItemCode,
  pub size_code: SizeCode,
  pub size_text: SizeDetails,
}
