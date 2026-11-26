use crate::bibfile::BibFile;
use crate::error::Result;
use crate::identifier::Identifier;

/// Return the original text of the one entry `id` refers to.
pub fn show<'a>(file: &'a BibFile, id: &Identifier) -> Result<&'a str> {
    let bib = file.parse();
    let entry = file.find_one(&bib, id)?;
    Ok(entry.span.text(&file.src))
}
