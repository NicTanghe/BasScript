mod fountain;
mod markdown;
mod shared;

use crate::buffer::Document;
use crate::model::{DocumentFormat, ParsedLine};

pub fn parse_document(document: &Document) -> Vec<ParsedLine> {
    parse_document_with_format(document, DocumentFormat::Fountain)
}

pub fn parse_document_with_format(document: &Document, format: DocumentFormat) -> Vec<ParsedLine> {
    match format {
        DocumentFormat::Fountain => fountain::parse(document),
        DocumentFormat::Markdown => markdown::parse(document),
    }
}
