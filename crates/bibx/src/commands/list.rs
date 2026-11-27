use bibx_core::ops::list::{Column, Row, absent_fields, list};
use clap::{Args, ValueEnum};
use comfy_table::{ContentArrangement, Table, presets};
use serde_json::{Map, Value, json};

use super::{FileArgs, load, write_stdout};

const DEFAULT_FIELDS: [&str; 3] = ["key", "year", "title"];

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
///
/// The table, and `--sort` in both formats, turn LaTeX into Unicode where possible. Math,
/// unknown commands and `~` are kept. Macros, `#` joins, identifier, URL and citation-key
/// fields, and values that cannot be decoded stay raw.
///
/// Malformed entries are left out and counted in a warning on stderr; `bibx check` locates them.
///
/// A field named in `--fields` or `--sort` that no entry has is reported as a warning on
/// stderr, and the exit code stays 0. The default fields and files without entries are not
/// reported.
#[derive(Debug, Args)]
pub struct ListArgs {
    /// Sort by `key`, `type` or any field; numbers sort numerically, missing values last.
    #[arg(long, value_name = "FIELD")]
    sort: Option<String>,

    /// Comma-separated fields to show. `key` and `type` are always available and mean the
    /// citation key and the entry type. [default: key,year,title]
    #[arg(long, value_delimiter = ',')]
    fields: Option<Vec<String>>,

    #[arg(long, value_enum, default_value = "table")]
    format: Format,

    #[command(flatten)]
    file: FileArgs,
}

pub fn run(args: ListArgs) -> anyhow::Result<()> {
    let target = args.file.target()?;
    let file = load(&target)?;
    let bib = file.parse();
    let sort = args.sort.as_deref().map(Column::parse);
    let requested: Option<Vec<Column>> = args
        .fields
        .map(|fields| fields.iter().map(|f| Column::parse(f)).collect());
    for name in absent_fields(&bib, requested.iter().flatten().chain(&sort)) {
        eprintln!("warning: no entry has the field `{}`", name.as_str());
    }
    let columns = requested.unwrap_or_else(|| DEFAULT_FIELDS.map(Column::parse).to_vec());
    let rows = list(&file.src, &bib, sort.as_ref(), &columns);
    let out = match args.format {
        Format::Table => table(&rows, &columns),
        Format::Json => json_output(&rows),
    };
    write_stdout("list", &out)
}

fn table(rows: &[Row], columns: &[Column]) -> String {
    if rows.is_empty() {
        return String::new();
    }
    let mut table = Table::new();
    table
        .load_preset(presets::UTF8_FULL_CONDENSED)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(columns.iter().map(Column::name));
    if !table.is_tty() {
        table.set_width(80);
    }
    for row in rows {
        table.add_row(columns.iter().map(|c| row.cell(c).unwrap_or_default()));
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
                .map(|(k, v)| (k.clone(), Value::String(v.raw().to_string())))
                .collect();
            json!({ "key": row.key, "type": row.entry_type, "fields": fields })
        })
        .collect();
    let mut out = serde_json::to_string_pretty(&rows).unwrap_or_else(|_| "[]".into());
    out.push('\n');
    out
}
