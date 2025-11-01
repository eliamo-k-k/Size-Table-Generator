use std::sync::Arc;

use calamine::{open_workbook, DataType, Reader, Xlsx};
use itertools::Itertools;
use melrose_types::{ItemCode, SizeCode};
use phdb_translate::TranslateClient;
use serde::Serialize;
use tauri::async_runtime::Mutex;
use tauri::Emitter;

use crate::{Error, Result};
pub trait MySpecification<Input>
where
  Self: Sized,
{
  fn parse(input: Input) -> Result<Self>;

  fn is_match(input: Input) -> bool {
    Self::parse(input).is_ok()
  }
}

#[derive(Serialize)]
pub struct ProcessResponse {
  item_meta: Vec<ItemMeta>,
}

struct ItemInfo {
  item_code: ItemCode,
  size_code: SizeCode,
  size_text: SizeDetails,
}

#[derive(Debug, Serialize)]
struct ItemTable {
  head: Vec<String>,
  body: Vec<Vec<String>>,
}

#[derive(Debug, Serialize)]
struct ItemMeta {
  code: String,
  size_code: String,
  table: ItemTable,
}

#[derive(Serialize, Clone)]
struct ProcessingStatePayload {
  state: String,
}

#[derive(Clone)]
struct SizeDetail {
  name: String,
  value: String,
}

impl MySpecification<String> for SizeDetail {
  fn parse(input: String) -> Result<Self> {
    if input.is_empty() {
      return Err(Error::EmptySizeText);
    }

    let escaped = escape_colon_whitespace(input);
    println!("escaped: {}", escaped);
    // should contain ':' and only one ':'
    if !escaped.contains(':') || escaped.matches(':').count() != 1 {
      println!("error line: {}", escaped);
      return Err(Error::InvalidSizeText {
        error_line: escaped,
      });
    }

    let name_value_pair = escaped.split(':').collect_vec();
    if name_value_pair.len() != 2 {
      println!("error line: {:?}", name_value_pair);
      return Err(Error::InvalidSizeText {
        error_line: escaped,
      });
    }

    if name_value_pair[0].is_empty() || name_value_pair[1].is_empty() {
      println!("error line: {:?}", name_value_pair);
      return Err(Error::InvalidSizeText {
        error_line: escaped,
      });
    }

    Ok(Self {
      name: name_value_pair[0].to_string(),
      value: name_value_pair[1].to_string(),
    })
  }
}

/// A Contain the multi name value pairs of size detail separated by whitespace
/// For instance: 肩宽:42.5cm 袖丈:62cm 胸囲:104cm 裾囲:104cm
#[derive(Clone)]
struct SizeDetails(Vec<SizeDetail>);

impl MySpecification<String> for SizeDetails {
  fn parse(input: String) -> Result<Self> {
    if input.is_empty() {
      return Err(Error::EmptySizeText);
    }

    let splitted = input.split_whitespace().collect_vec();
    if splitted.is_empty() {
      return Err(Error::EmptySizeText);
    }

    let size_details = splitted
      .iter()
      .map(|s| SizeDetail::parse(s.to_string()))
      .collect::<Result<Vec<_>>>()?;

    Ok(Self(size_details))
  }
}

impl SizeDetails {
  fn names(&self) -> Vec<String> {
    self.0.iter().map(|sd| sd.name.to_owned()).collect()
  }

  fn values(&self) -> Vec<String> {
    self.0.iter().map(|sd| sd.value.to_owned()).collect()
  }

  /// translate the name field of all size_detail to chinese
  async fn translate_to_zh(self, translate_client: &mut TranslateClient) -> Result<Self> {
    let translated = translate_client.translate_local(&self.names())?;
    let mut cloned_self = self.clone();
    for (i, sd) in cloned_self.0.iter_mut().enumerate() {
      sd.name = translated[i].to_owned();
    }
    Ok(cloned_self)
  }
}

#[tauri::command]
pub async fn process_excel_file(
  window: tauri::Window,
  excel_path: String,
  client: tauri::State<'_, Arc<Mutex<TranslateClient>>>,
) -> std::result::Result<ProcessResponse, String> {
  println!("command invoked");
  window
    .emit(
      "update-state",
      ProcessingStatePayload {
        state: "processing file".into(),
      },
    )
    .map_err(Error::Tauri)?;

  let mut excel_file: Xlsx<_> = open_workbook(excel_path).map_err(|_| Error::ExcelRead)?;
  let items_code_sheet = excel_file
    .worksheet_range_at(0)
    .ok_or(Error::EmptyFile)?
    .map_err(|_| Error::EmptyFile)?;
  let (item_code_idx, size_code_idx, size_text_idx) = items_code_sheet
    .rows()
    .next()
    .ok_or(Error::EmptyFile)?
    .iter()
    .enumerate()
    .filter_map(|(i, cell)| check_column(i, cell.to_string()).ok())
    .collect_tuple()
    .ok_or(Error::InvalidSheetFormat)?;
  // TODO)) 必要なフィールドに空欄がある場合、無視にする？
  // いらない行に消し忘れがあると、気づかない
  let item_code_size_unique = items_code_sheet
    .rows()
    .skip(1)
    .unique_by(|row| {
      (
        row[item_code_idx].to_string(),
        row[size_code_idx].to_string(),
      )
    })
    .sorted_by(|a, b| {
      a[item_code_idx]
        .to_string()
        .cmp(&b[item_code_idx].to_string())
    })
    .collect_vec();
  let mut item_code_isolated_rows: Vec<Vec<&[DataType]>> = Vec::new();
  let mut changed_item_code = item_code_size_unique[0][item_code_idx].to_string();
  let mut rows_vec = Vec::new();
  let rows_len = item_code_size_unique.len();
  for (i, row) in item_code_size_unique.into_iter().enumerate() {
    if row[item_code_idx].to_string() == changed_item_code {
      rows_vec.push(row);
      continue;
    }
    item_code_isolated_rows.push(rows_vec);
    changed_item_code = row[item_code_idx].to_string();
    rows_vec = vec![row];
    if i == rows_len - 1 {
      item_code_isolated_rows.push(rows_vec);
      break;
    }
  }
  window
    .emit(
      "update-state",
      ProcessingStatePayload {
        state: "translating".into(),
      },
    )
    .unwrap();
  let mut local_client = client.lock().await;
  let mut item_code_size_data = Vec::new();
  for item_code_isolated_row in item_code_isolated_rows {
    let mut item_infos = Vec::new();
    for row in item_code_isolated_row {
      println!("row: {:?}", row);
      println!("item_code: {:?}", &row[item_code_idx].to_string());
      println!("size_code: {:?}", &row[size_code_idx].to_string());
      let item_code = row[item_code_idx]
        .to_string()
        .replace(" ", "_")
        .parse::<ItemCode>()
        .map_err(|e| Error::MelroseType(melrose_types::error::Error::from(e)))?;
      let size_code = row[size_code_idx]
        .to_string()
        .parse::<SizeCode>()
        .map_err(|e| Error::MelroseType(melrose_types::error::Error::from(e)))?;
      let size_text = SizeDetails::parse(row[size_text_idx].to_string())?;
      let size_text_zh = size_text.translate_to_zh(&mut local_client).await?;
      item_infos.push(ItemInfo {
        item_code,
        size_code,
        size_text: size_text_zh,
      });
    }
    item_code_size_data.push(item_infos);
  }

  window
    .emit(
      "update-state",
      ProcessingStatePayload {
        state: "processing file".into(),
      },
    )
    .unwrap();
  let mut item_meta = Vec::new();
  for item_infos in item_code_size_data {
    let mut table_head = item_infos[0].size_text.names();
    table_head.insert(0, String::from("尺码"));
    let table_body = item_infos
      .iter()
      .map(|item_info| {
        let mut size_row_raw = item_info.size_text.values();
        size_row_raw.insert(0, item_info.size_code.to_roman_numeral());
        size_row_raw
      })
      .collect::<Vec<_>>();
    let table = ItemTable {
      head: table_head,
      body: table_body,
    };
    item_meta.push(ItemMeta {
      code: item_infos[0].item_code.to_string(),
      size_code: item_infos[0].size_code.to_string(),
      table,
    });
  }
  println!("item_meta: {:?}", item_meta);
  Ok(ProcessResponse { item_meta })
}

fn check_column(i: usize, s: impl AsRef<str>) -> Result<usize> {
  match s.as_ref().trim() {
    "品番" | "SZ" | "採寸" => Ok(i),
    _ => Err(Error::InvalidSheetFormat),
  }
}

#[inline]
fn escape_colon_whitespace(s: impl AsRef<str>) -> String {
  s.as_ref()
    .replace('：', ":")
    .replace(": ", ":")
    .replace('　', " ")
}
