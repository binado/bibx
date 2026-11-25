use std::io::Write;

use bibx_core::ops::list::{Row, SortKey, list};
use clap::{Args, ValueEnum};
use comfy_table::{ContentArrangement, Table, presets};
use serde_json::{Map, Value, json};

use super::{load, single_target};

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Format {
    Table,
    Json,
}

/// List the entries of the bib file, in file order unless `--sort` is given.
///
/// `--format json` prints a stable array of objects, one per entry:
///
///   [{"key": "Maldacena:1997re", "type": "article", "fields": {"year": "1998", "title": "..."}}]
///
/// `fields` holds the requested fields (other than `key` and `type`) that the entry has, as
/// raw text with whitespace runs collapsed.
#[derive(Debug, Args)]
pub struct ListArgs {
    /// Sort by `key`, `type` or any field; a year sorts numerically, missing values last.
    #[arg(long, value_name = "FIELD")]
    sort: Option<String>,

    /// Comma-separated fields to show. `key` and `type` are always available.
    #[arg(long, value_delimiter = ',', default_value = "key,year,title")]
    fields: Vec<String>,

    #[arg(long, value_enum, default_value = "table")]
    format: Format,
}

pub fn run(args: ListArgs, files: &[std::path::PathBuf]) -> anyhow::Result<()> {
    let target = single_target(files)?;
    let file = load(&target)?;
    let bib = file.parse();
    let sort = args.sort.as_deref().map(SortKey::parse);
    let rows = list(&file.src, &bib, sort.as_ref(), &args.fields);
    let out = match args.format {
        Format::Table => table(&rows, &args.fields),
        Format::Json => json_output(&rows),
    };
    let mut stdout = std::io::stdout().lock();
    // A closed pipe (`bibx list | head`) is not an error.
    let _ = stdout.write_all(out.as_bytes());
    Ok(())
}

fn cell(row: &Row, name: &str) -> String {
    match name.to_ascii_lowercase().as_str() {
        "key" => row.key.clone(),
        "type" => row.entry_type.clone(),
        name => row.fields.get(name).cloned().unwrap_or_default(),
    }
}

fn table(rows: &[Row], fields: &[String]) -> String {
    if rows.is_empty() {
        return String::new();
    }
    let mut table = Table::new();
    table
        .load_preset(presets::UTF8_FULL_CONDENSED)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(fields.iter().map(|f| f.to_ascii_lowercase()));
    if !table.is_tty() {
        table.set_width(80);
    }
    for row in rows {
        table.add_row(fields.iter().map(|f| cell(row, f)));
    }
    format!("{table}\n")
}

fn json_output(rows: &[Row]) -> String {
    let rows: Vec<Value> = rows
        .iter()
        .map(|row| {
            let fields: Map<String, Value> = row
                .fields
                .iter()
                .map(|(k, v)| (k.clone(), Value::String(v.clone())))
                .collect();
            json!({ "key": row.key, "type": row.entry_type, "fields": fields })
        })
        .collect();
    let mut out = serde_json::to_string_pretty(&rows).unwrap_or_else(|_| "[]".into());
    out.push('\n');
    out
}
