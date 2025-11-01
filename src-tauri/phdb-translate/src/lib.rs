mod error;

use std::collections::HashMap;

pub use error::Error;
use gcp_auth::{AuthenticationManager, Token};
use reqwest::Client;
use serde::Deserialize;

use crate::error::Result;

// ビルド時に生成されたglossaryモジュールをインクルード
#[path = ""]
mod embedded_glossary {
  include!(concat!(env!("OUT_DIR"), "/glossary.rs"));
}

const TRANSLATE_URL: &str =
  "https://translation.googleapis.com/v3/projects/phdb-translate/locations/us-central1:translateText";
pub struct TranslateClient {
  gcp_token: Option<Token>,
  http_client: Client,
  auth_manager: Option<AuthenticationManager>,
  glossary: HashMap<String, String>,
}

const SCOPES: &[&str] = &["https://www.googleapis.com/auth/cloud-platform"];
const GLOSSARY_URL: &str =
  "https://size-table-generator.s3.ap-northeast-1.amazonaws.com/phdb-glossary.csv";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GoogleTranslateSuccessResponse {
  glossary_translations: Vec<Translation>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Translation {
  translated_text: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GoogleTranslateFailedResponse {
  error: ErrorMessage,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ErrorMessage {
  message: String,
}

impl TranslateClient {
  pub async fn new() -> Result<Self> {
    let http_client = reqwest::Client::new();
    let glossary = Self::read_glossary_file(&http_client).await?;
    Ok(Self {
      gcp_token: None,
      http_client,
      auth_manager: None,
      glossary,
    })
  }

  async fn ensure_token(&mut self) -> Result<&Token> {
    if self.gcp_token.is_none() || self.auth_manager.is_none() {
      // 初回取得またはトークンが無効な場合
      let auth_manager = AuthenticationManager::new().await?;
      let token = auth_manager.get_token(SCOPES).await?;
      self.gcp_token = Some(token);
      self.auth_manager = Some(auth_manager);
    }
    Ok(self.gcp_token.as_ref().unwrap())
  }

  pub async fn refresh_token(&mut self) -> Result<()> {
    let auth_manager = match &mut self.auth_manager {
      Some(am) => am,
      None => {
        let am = AuthenticationManager::new().await?;
        self.auth_manager = Some(am);
        self.auth_manager.as_mut().unwrap()
      }
    };
    let token = auth_manager.get_token(SCOPES).await?;
    self.gcp_token = Some(token);
    Ok(())
  }

  pub fn translate_local(&mut self, inputs: &[String]) -> Result<Vec<String>> {
    let mut translated = Vec::new();
    for input in inputs {
      if self.glossary.contains_key(input) {
        translated.push(self.glossary[input].clone());
      } else {
        translated.push(input.clone());
      }
    }
    Ok(translated)
  }
  /// translate the input text to zh
  ///
  /// this functions rely on google cloud translate api
  ///
  /// [WARN] so it will not work in china mainland
  pub async fn translate(&mut self, inputs: &[String]) -> Result<Vec<String>> {
    // トークンがまだ取得されていない場合、ここで取得
    let token_str = {
      let token = self.ensure_token().await?;
      token.as_str().to_string()
    };

    let translate_request_data = serde_json::json!(
        {
          "sourceLanguageCode": "ja",
          "targetLanguageCode": "zh",
          "contents": inputs,
            "glossaryConfig": {
            "glossary":"projects/phdb-translate/locations/us-central1/glossaries/phdb-glossary1"
          }
        }
    );
    let resp = self
      .http_client
      .post(TRANSLATE_URL)
      .header("Content-Type", "application/json; charset=utf-8")
      .json(&translate_request_data)
      .bearer_auth(token_str)
      .send()
      .await?;
    if resp.status().as_u16() >= 300 {
      let resp_message: GoogleTranslateFailedResponse = resp.json().await?;
      return Err(Error::TranslateResponse(resp_message.error.message));
    }

    let res: GoogleTranslateSuccessResponse = resp.json().await?;

    Ok(
      res
        .glossary_translations
        .into_iter()
        .map(|t| t.translated_text)
        .collect(),
    )
  }

  async fn read_glossary_file(http_client: &Client) -> Result<HashMap<String, String>> {
    // まず、ビルド時に埋め込まれたglossaryファイルを試す
    let glossary_file = match Self::get_embedded_glossary() {
      Some(content) if !content.is_empty() => {
        println!("Using embedded glossary file from build time");
        content.to_string()
      }
      _ => {
        // 埋め込みファイルが存在しない場合、実行時にダウンロード
        println!("Embedded glossary not found or empty, downloading from URL...");
        Self::fetch_glossary_file(http_client).await?
      }
    };

    let mut reader = csv::Reader::from_reader(glossary_file.as_bytes());
    let mut glossary = HashMap::new();
    for result in reader.records() {
      let record = result.unwrap();
      glossary.insert(record[0].to_string(), record[1].to_string());
    }
    Ok(glossary)
  }

  fn get_embedded_glossary() -> Option<&'static str> {
    // ビルド時に生成されたglossaryコードから取得
    let content = embedded_glossary::get_glossary_content();
    if content.is_empty() {
      None
    } else {
      Some(content)
    }
  }

  async fn fetch_glossary_file(http_client: &Client) -> Result<String> {
    let resp = http_client.get(GLOSSARY_URL).send().await?;
    let text = resp.text().await?;
    Ok(text)
  }
}

#[cfg(test)]
mod tests {
  use crate::TranslateClient;
  /// need env GOOGLE_APPLICATION_CREDENTIALS
  #[tokio::test]
  async fn it_works() {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt()
      .with_max_level(tracing::Level::INFO)
      .init();
    let mut client = TranslateClient::new().await.unwrap();
    let inputs = vec![
      "ヒップ:104",
      "裾周り:41",
      "ウエスト(ゴム):62~72",
      "もも周り:70",
      "股上:37",
      "股下:66",
      "高さ:23.5",
      "縦:17",
      "重量(g):225",
      "前身頃:57",
    ];
    let glossary_path = std::env::var("GLOSSARY_PATH").unwrap();
    let resp = client.translate(&inputs).await.unwrap();
    resp.iter().for_each(|s| println!("{s}"));
  }
}
