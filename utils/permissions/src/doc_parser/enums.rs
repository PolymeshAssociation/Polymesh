use lazy_static::lazy_static;
use regex::Regex;
use scraper::{ElementRef, Html, Selector};

lazy_static! {
    static ref VARIANT_REGEX: Regex = Regex::new(r#"variant\.(.+)"#).unwrap();
}

#[derive(Debug)]
pub struct EnumDoc {
    pub variants: Vec<Variant>,
}

#[derive(Debug)]
pub struct Variant {
    pub name: String,
    pub doc_block: Html,
}

pub struct EnumDocParser {
    html: Html,
}

impl EnumDocParser {
    pub fn parse(html: Html) -> Option<EnumDoc> {
        let parser = Self { html };
        Some(EnumDoc {
            variants: parser.parse_all_variants(),
        })
    }

    fn parse_all_variants(&self) -> Vec<Variant> {
        let all_div_selector = Selector::parse("div").unwrap();
        let all_divs: Vec<ElementRef> = self.html.select(&all_div_selector).collect();

        all_divs
            .iter()
            .enumerate()
            .filter_map(|(i, _)| self.parse_variant(&all_divs, i))
            .collect()
    }

    fn parse_variant(&self, all_divs: &[ElementRef], index: usize) -> Option<Variant> {
        let variant_div = all_divs[index];
        // Look for a variant div and parse the name
        let name = VARIANT_REGEX
            .captures(variant_div.value().id()?)?
            .get(1)?
            .as_str()
            .to_owned();

        // Check for a doc block associated with the above variant
        let doc_block = all_divs[index + 1..]
            .iter()
            // Stop iterating once we hit the next variant
            .take_while(|div| {
                div.value()
                    .id()
                    .map(|id| !VARIANT_REGEX.is_match(id))
                    .unwrap_or(true)
            })
            .filter(|div| div.value().classes.iter().any(|s| s == "docblock"))
            .map(|div| Html::parse_document(&div.html()))
            .next()?;

        Some(Variant { name, doc_block })
    }
}
