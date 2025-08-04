use crate::{AppResult, errors::AppError};
use sqlx::{Pool, Postgres, Row};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct CodeGeneratorConfig {
  pub table_name: String,
  pub code_column: String,
  pub workspace_column: Option<String>, // Some modules might not use workspace
  pub prefix_length: usize,             // 1-3 characters
  pub number_length: usize,             // default 5 digits
  pub separator: String,                // default "-"
}

impl Default for CodeGeneratorConfig {
  fn default() -> Self {
    Self {
      table_name: String::new(),
      code_column: "code".to_string(),
      workspace_column: Some("workspace_id".to_string()),
      prefix_length: 2,
      number_length: 5,
      separator: "-".to_string(),
    }
  }
}

pub struct CodeGenerator {
  pool: Pool<Postgres>,
}

impl CodeGenerator {
  pub fn new(pool: Pool<Postgres>) -> Self {
    Self { pool }
  }

  /// Generate next available code based on name and configuration
  pub async fn get_next_available_code(&self, config: &CodeGeneratorConfig, name: &str, workspace_id: Option<Uuid>) -> AppResult<String> {
    let prefix = self.generate_prefix_from_name(name, config.prefix_length);

    let (query, _params) = self.build_query(config, &prefix, workspace_id);

    let row = sqlx::query(&query);
    let row = match (workspace_id, &config.workspace_column) {
      (Some(ws_id), Some(_)) => row.bind(ws_id).bind(format!("{}{}%", prefix, config.separator)),
      (None, None) => row.bind(format!("{}{}%", prefix, config.separator)),
      _ => return Err(AppError::Internal("Workspace configuration mismatch".to_string())),
    };

    let row = row.fetch_optional(&self.pool).await?;

    let next_code = match row {
      Some(row) => {
        let last_code: String = row.get(config.code_column.as_str());
        self.increment_code(&last_code, config)?
      }
      None => format!("{}{}{:0width$}", prefix, config.separator, 1, width = config.number_length),
    };

    Ok(next_code)
  }

  /// Check if code exists in table
  pub async fn code_exists(&self, config: &CodeGeneratorConfig, code: &str, workspace_id: Option<Uuid>) -> AppResult<bool> {
    let (query, _) = self.build_exists_query(config, workspace_id);

    let row = sqlx::query(&query);
    let row = match (workspace_id, &config.workspace_column) {
      (Some(ws_id), Some(_)) => row.bind(ws_id).bind(code),
      (None, None) => row.bind(code),
      _ => return Err(AppError::Internal("Workspace configuration mismatch".to_string())),
    };

    let result = row.fetch_optional(&self.pool).await?;
    Ok(result.is_some())
  }

  fn build_query(&self, config: &CodeGeneratorConfig, prefix: &str, workspace_id: Option<Uuid>) -> (String, Vec<String>) {
    let pattern_regex = format!(r"^{}\{}{}\d{{{}}}$", prefix, config.separator, config.separator, config.number_length);

    match (workspace_id, &config.workspace_column) {
      (Some(_), Some(ws_col)) => {
        let query = format!(
          "SELECT {} FROM {} WHERE {} = $1 AND {} LIKE $2 AND {} ~ '{}' ORDER BY {} DESC LIMIT 1",
          config.code_column, config.table_name, ws_col, config.code_column, config.code_column, pattern_regex, config.code_column
        );
        (query, vec![])
      }
      (None, None) => {
        let query = format!(
          "SELECT {} FROM {} WHERE {} LIKE $1 AND {} ~ '{}' ORDER BY {} DESC LIMIT 1",
          config.code_column, config.table_name, config.code_column, config.code_column, pattern_regex, config.code_column
        );
        (query, vec![])
      }
      _ => ("".to_string(), vec![]),
    }
  }

  fn build_exists_query(&self, config: &CodeGeneratorConfig, workspace_id: Option<Uuid>) -> (String, Vec<String>) {
    match (workspace_id, &config.workspace_column) {
      (Some(_), Some(ws_col)) => {
        let query = format!(
          "SELECT 1 FROM {} WHERE {} = $1 AND {} = $2 LIMIT 1",
          config.table_name, ws_col, config.code_column
        );
        (query, vec![])
      }
      (None, None) => {
        let query = format!("SELECT 1 FROM {} WHERE {} = $1 LIMIT 1", config.table_name, config.code_column);
        (query, vec![])
      }
      _ => ("".to_string(), vec![]),
    }
  }

  /// Generate prefix from name
  /// Examples:
  /// - "John Doe" -> "JD"
  /// - "Acme Corporation" -> "AC"
  /// - "PT Maju Jaya" -> "PMJ"
  /// - "Apple" -> "AP"
  fn generate_prefix_from_name(&self, name: &str, max_length: usize) -> String {
    let words: Vec<&str> = name.split_whitespace().filter(|word| !word.is_empty()).collect();

    let prefix = match words.len() {
      0 => "X".to_string(),
      1 => {
        let first_word = words[0].to_uppercase();
        if first_word.len() >= max_length {
          first_word[..max_length].to_string()
        } else {
          first_word
        }
      }
      _ => {
        let mut result = String::new();
        for (i, word) in words.iter().take(max_length).enumerate() {
          if let Some(first_char) = word.chars().next() {
            result.push(first_char.to_uppercase().next().unwrap_or('X'));
          }
          if result.len() >= max_length || i >= max_length - 1 {
            break;
          }
        }
        result
      }
    };

    // Ensure minimum length of 1
    if prefix.is_empty() { "X".to_string() } else { prefix }
  }

  /// Increment code number
  fn increment_code(&self, last_code: &str, config: &CodeGeneratorConfig) -> AppResult<String> {
    let parts: Vec<&str> = last_code.split(&config.separator).collect();
    if parts.len() != 2 {
      return Err(AppError::Internal("Invalid code format".to_string()));
    }

    let prefix = parts[0];
    let number_str = parts[1];

    let current_number: u32 = number_str.parse().map_err(|_| AppError::Internal("Invalid code format".to_string()))?;

    let next_number = current_number + 1;
    let max_number = 10_u32.pow(config.number_length as u32) - 1;

    if next_number > max_number {
      return Err(AppError::Internal(format!(
        "Maximum number reached for prefix '{}' (max: {})",
        prefix, max_number
      )));
    }

    Ok(format!(
      "{}{}{:0width$}",
      prefix,
      config.separator,
      next_number,
      width = config.number_length
    ))
  }
}
