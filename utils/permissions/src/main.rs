use crate::doc_parser::enums::{EnumDoc, EnumDocParser};
use regex::Regex;
use scraper::{ElementRef, Html, Selector};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{read_dir, File};
use std::io::Read;
use std::path::PathBuf;

mod doc_parser;

fn main() {
    assert!(
        std::env::current_dir()
            .ok()
            .map(|dir| dir.join("target/doc").exists())
            .unwrap_or(false),
        "can't find target directory! trying running in project root"
    );

    let pallets_regex = Regex::new("target/doc/pallet_.*").unwrap();

    let mut permission_mappings = BTreeMap::new();

    read_dir("target/doc")
        .expect("dir does not exist")
        .into_iter()
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .collect::<BTreeSet<PathBuf>>()
        .into_iter()
        .filter(|path| {
            path.to_str()
                .map(|str| pallets_regex.is_match(str))
                .unwrap_or(false)
        })
        .filter_map(|path| {
            path.file_name()
                .and_then(|name| name.to_str().map(|s| s.to_owned()))
                .map(|name| (name, path.join("enum.Call.html")))
        })
        .filter(|(_, path)| path.exists())
        .filter_map(|(pallet, path)| File::open(&path).ok().map(|f| (pallet, f)))
        .filter_map(|(pallet, mut file)| {
            let mut text = String::new();
            file.read_to_string(&mut text).ok().map(|_| (pallet, text))
        })
        .filter(|(_, text)| !text.is_empty())
        .filter_map(|(pallet, text)| {
            EnumDocParser::parse(Html::parse_document(&text)).map(|doc| (pallet, doc))
        })
        .map(|(pallet, doc)| (pallet, parse_permissions(doc)))
        .for_each(|(pallet, permissions)| {
            for (extrinsic, permission) in permissions {
                let extrinsic_mapping = permission_mappings
                    .entry(pallet.clone())
                    .or_insert(BTreeMap::new());
                extrinsic_mapping
                    .entry(extrinsic.clone())
                    .or_insert(Vec::new())
                    .push(permission);
            }
        });

    println!("{:#?}", permission_mappings);
}

fn parse_permissions(doc: EnumDoc) -> Vec<(String, String)> {
    let mut permissions = Vec::new();
    for variant in doc.variants {
        if let Some(doc_block_div) = variant
            .doc_block
            .select(&Selector::parse("div.docblock").unwrap())
            .next()
        {
            let children: Vec<ElementRef> = doc_block_div
                .select(&Selector::parse("*").unwrap())
                .collect();

            let h1_selector = Selector::parse("h1").unwrap();
            let ul_selector = Selector::parse("ul").unwrap();
            let li_selector = Selector::parse("li").unwrap();

            for (i, child) in children.iter().enumerate() {
                if h1_selector.matches(child)
                    && child
                        .value()
                        .id()
                        .map(|s| s == "permissions")
                        .unwrap_or(false)
                {
                    if let Some(ul) = children[i + 1..]
                        .iter()
                        .take_while(|element| !h1_selector.matches(element))
                        .filter(|element| ul_selector.matches(element))
                        .next()
                    {
                        for li in ul
                            .select(&li_selector)
                            .map(|element| element.text().collect())
                        {
                            permissions.push((variant.name.clone(), li))
                        }
                    }
                }
            }
        }
    }
    permissions
}
