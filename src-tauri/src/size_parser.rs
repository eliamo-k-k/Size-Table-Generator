use crate::models::{SizeDetail, SizeDetails};
use crate::{Error, Result};
use itertools::Itertools;

pub trait MySpecification<Input>
where
  Self: Sized,
{
  fn parse(input: Input) -> Result<Self>;

  fn is_match(input: Input) -> bool {
    Self::parse(input).is_ok()
  }
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
  pub fn names(&self) -> Vec<String> {
    self.0.iter().map(|sd| sd.name.to_owned()).collect()
  }

  pub fn values(&self) -> Vec<String> {
    self.0.iter().map(|sd| sd.value.to_owned()).collect()
  }

  pub async fn translate_to_zh(
    self,
    translate_client: &mut phdb_translate::TranslateClient,
  ) -> Result<Self> {
    let translated = translate_client.translate_local(&self.names())?;
    let mut cloned_self = self.clone();
    for (i, sd) in cloned_self.0.iter_mut().enumerate() {
      sd.name = translated[i].to_owned();
    }
    Ok(cloned_self)
  }
}

#[inline]
pub fn escape_colon_whitespace(s: impl AsRef<str>) -> String {
  s.as_ref()
    .replace('：', ":")
    .replace(": ", ":")
    .replace('　', " ")
}
