use calamine::{open_workbook, DataType, Reader, Xlsx};
use itertools::Itertools;
use melrose_types::{ItemCode, SizeCode};
use tauri::Window;

use crate::models::{ItemInfo, ItemMeta, ItemTable, ProcessingStatePayload};
use crate::size_parser::SizeDetails;
use crate::{Error, Result};

pub fn check_column(i: usize, s: impl AsRef<str>) -> Result<usize> {
  match s.as_ref().trim() {
    "品番" | "SZ" | "採寸" => Ok(i),
    _ => Err(Error::InvalidSheetFormat),
  }
}

pub fn process_excel_rows(
  rows: Vec<Vec<&[DataType]>>,
  item_code_idx: usize,
  size_code_idx: usize,
  size_text_idx: usize,
) -> Result<Vec<Vec<ItemInfo>>> {
  let mut item_code_isolated_rows: Vec<Vec<&[DataType]>> = Vec::new();
  let mut changed_item_code = rows[0][item_code_idx].to_string();
  let mut rows_vec = Vec::new();
  let rows_len = rows.len();

  for (i, row) in rows.into_iter().enumerate() {
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

  let mut item_code_size_data = Vec::new();
  for item_code_isolated_row in item_code_isolated_rows {
    let mut item_infos = Vec::new();
    for row in item_code_isolated_row {
      let item_code = row[item_code_idx]
        .to_string()
        .replace(" ", "_")
        .parse_by_specification()
        .map_err(Error::MelroseType)?;
      let size_code = row[size_code_idx]
        .to_string()
        .parse_by_specification()
        .map_err(Error::MelroseType)?;
      let size_text = SizeDetails::parse(row[size_text_idx].to_string())?;
      item_infos.push(ItemInfo {
        item_code,
        size_code,
        size_text,
      });
    }
    item_code_size_data.push(item_infos);
  }

  Ok(item_code_size_data)
}

pub fn create_item_meta(item_infos: Vec<ItemInfo>) -> ItemMeta {
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
  ItemMeta {
    code: item_infos[0].item_code.to_string(),
    size_code: item_infos[0].size_code.to_string(),
    table,
  }
}

pub fn read_excel_file(excel_path: String) -> Result<Xlsx<std::fs::File>> {
  open_workbook(excel_path).map_err(|_| Error::ExcelRead)
}

pub fn get_sheet_data(excel_file: &mut Xlsx<std::fs::File>) -> Result<calamine::Range<DataType>> {
  excel_file
    .worksheet_range_at(0)
    .ok_or(Error::EmptyFile)?
    .map_err(|_| Error::EmptyFile)
}

pub fn get_column_indices(sheet: &calamine::Range<DataType>) -> Result<(usize, usize, usize)> {
  sheet
    .rows()
    .next()
    .ok_or(Error::EmptyFile)?
    .iter()
    .enumerate()
    .filter_map(|(i, cell)| check_column(i, cell.to_string()).ok())
    .collect_tuple()
    .ok_or(Error::InvalidSheetFormat)
}

pub fn get_unique_rows(
  sheet: &calamine::Range<DataType>,
  item_code_idx: usize,
  size_code_idx: usize,
) -> Vec<Vec<&[DataType]>> {
  sheet
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
    .collect_vec()
}
