use std::io::{ErrorKind, Write};

use anyhow::Context;
use bibx_core::ops::list::{Row, SortKey, list};
use clap::{Args, ValueEnum};
use comfy_table::{ContentArrangement, Table, presets};
use serde_json::{Map, Value, json};

use super::{FileArgs, load};

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
#[derive(Debug, Args)]
pub struct ListArgs {
    /// Sort by `key`, `type` or any field; numbers sort numerically, missing values last.
    #[arg(long, value_name = "FIELD")]
    sort: Option<String>,

    /// Comma-separated fields to show. `key` and `type` are always available.
    #[arg(long, value_delimiter = ',', default_value = "key,year,title")]
    fields: Vec<String>,

    #[arg(long, value_enum, default_value = "table")]
    format: Format,

    #[command(flatten)]
    file: FileArgs,
}

pub fn run(args: ListArgs) -> anyhow::Result<()> {
    let target = args.file.target()?;
    let file = load(&target)?;
    let bib = file.parse();
    let sort = args.sort.as_deref().map(SortKey::parse);
    let rows = list(&file.src, &bib, sort.as_ref(), &args.fields);
    let out = match args.format {
        Format::Table => table(&rows, &args.fields),
        Format::Json => json_output(&rows),
    };
    let mut stdout = stdout_writer().context("opening stdout for list output")?;
    match stdout
        .write_all(out.as_bytes())
        .and_then(|()| stdout.flush())
    {
        Err(err) if err.kind() == ErrorKind::BrokenPipe => Ok(()),
        result => result.context("writing list output to stdout"),
    }
}

#[cfg(unix)]
fn stdout_writer() -> std::io::Result<impl Write> {
    use std::os::fd::AsFd;

    // Stdout treats EBADF as success. Writing through a duplicate reports that error.
    let fd = std::io::stdout().as_fd().try_clone_to_owned()?;
    Ok(std::fs::File::from(fd))
}

#[cfg(not(unix))]
fn stdout_writer() -> std::io::Result<impl Write> {
    Ok(std::io::stdout().lock())
}

fn cell(row: &Row, name: &str) -> String {
    match name.to_ascii_lowercase().as_str() {
        "key" => row.key.clone(),
        "type" => row.entry_type.clone(),
        name => row
            .fields
            .get(name)
            .map(|v| v.decoded().into_owned())
            .unwrap_or_default(),
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
                .map(|(k, v)| (k.clone(), Value::String(v.raw().to_string())))
                .collect();
            json!({ "key": row.key, "type": row.entry_type, "fields": fields })
        })
        .collect();
    let mut out = serde_json::to_string_pretty(&rows).unwrap_or_else(|_| "[]".into());
    out.push('\n');
    out
}
