use std::env;
use std::fs;
use std::path::PathBuf;

const GLOSSARY_URL: &str =
  "https://size-table-generator.s3.ap-northeast-1.amazonaws.com/phdb-glossary.csv";

fn main() {
  download_and_embed_glossary();
}

fn download_and_embed_glossary() {
  let out_dir = env::var("OUT_DIR").unwrap();
  let glossary_code_path = PathBuf::from(&out_dir).join("glossary.rs");

  println!(
    "cargo:warning=Downloading glossary file from {}",
    GLOSSARY_URL
  );

  // reqwestのblocking clientを使用してダウンロード
  match reqwest::blocking::get(GLOSSARY_URL) {
    Ok(response) => {
      if response.status().is_success() {
        match response.text() {
          Ok(text) => {
            // CSVファイルの内容をRustコードとして埋め込む
            let code = generate_glossary_code(&text);
            if let Err(e) = fs::write(&glossary_code_path, code) {
              println!("cargo:warning=Failed to write glossary code: {}", e);
            } else {
              println!("cargo:warning=Glossary file downloaded and embedded");
              println!("cargo:rerun-if-changed={}", glossary_code_path.display());
            }
          }
          Err(e) => {
            println!("cargo:warning=Failed to read response text: {}", e);
            generate_empty_glossary_code(&glossary_code_path);
          }
        }
      } else {
        println!(
          "cargo:warning=Failed to download glossary file: HTTP {}",
          response.status()
        );
        generate_empty_glossary_code(&glossary_code_path);
      }
    }
    Err(e) => {
      println!(
        "cargo:warning=Failed to download glossary file: {}. Using empty glossary.",
        e
      );
      generate_empty_glossary_code(&glossary_code_path);
    }
  }
}

fn generate_glossary_code(csv_content: &str) -> String {
  let mut code = String::from("pub fn get_glossary_content() -> &'static str {\n    r#\"");

  // CSVの内容をエスケープ
  let escaped = csv_content.replace("\"", "\"\"");
  code.push_str(&escaped);
  code.push_str("\"#\n}\n");

  code
}

fn generate_empty_glossary_code(path: &PathBuf) {
  let code = "pub fn get_glossary_content() -> &'static str {\n    \"\"\n}\n";
  let _ = fs::write(path, code);
}
