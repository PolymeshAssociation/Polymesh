/// # Overview
/// The intended purpose of this script is to parse the permissions of individual extrinsics.
/// Extrinsics are documented as variants of a `Call` enum located at the root of the pallet.
/// The output is in CSV in the following format `<pallet>, <extrinsic>, <permission>, <permission>...`
///
/// # Usage
/// Build a fresh version of the rustdoc
/// ```
/// cargo doc
/// ```
///
/// Run the script
/// ```
/// cargo run --package permissions --bin permissions
/// ```
use crate::doc_parser::enums::{EnumDoc, EnumDocParser};
use regex::Regex;
use scraper::{Html, Selector};
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
        "can't find target directory! try running in project root"
    );

    let pallets_regex = Regex::new("target/doc/pallet_.*").unwrap();

    let mut permission_mappings: BTreeMap<String, BTreeMap<String, Vec<String>>> = BTreeMap::new();

    // read the docs of all crates in target that match a regex
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
                permission_mappings
                    .entry(pallet.clone())
                    .or_insert(Default::default())
                    .entry(extrinsic.clone())
                    .or_insert(Default::default())
                    .push(permission);
            }
        });

    permission_mappings
        .into_iter()
        .for_each(|(pallet, mapping)| {
            mapping.into_iter().for_each(|(extrinsic, permissions)| {
                println!("{},{},{}", pallet, extrinsic, permissions.join(","))
            })
        });
}

fn parse_permissions(doc: EnumDoc) -> Vec<(String, String)> {
    let h1_selector = Selector::parse("h1").unwrap();
    let h2_selector = Selector::parse("h2").unwrap();
    let ul_selector = Selector::parse("ul").unwrap();
    let li_selector = Selector::parse("li").unwrap();
    let is_heading = |child| h1_selector.matches(&child) || h2_selector.matches(&child);

    let mut permissions = Vec::new();
    doc.variants
        .iter()
        // Find the div that contains the content of the doc block
        .filter_map(|variant| {
            variant
                .doc_block
                .select(&Selector::parse("div.docblock").unwrap())
                .next()
                .map(|div| (variant.name.clone(), div))
        })
        .for_each(|(name, doc_block_div)| {
            // Collect all children of the doc block, which in practice is the content of the doc block
            let children: Vec<_> = doc_block_div
                .select(&Selector::parse("*").unwrap())
                .collect();

            children
                .iter()
                .enumerate()
                .filter(|(_, child)| is_heading(**child))
                // Check if the heading is `Permissions`
                .filter(|(_, child)| {
                    child
                        .value()
                        .id()
                        .map(|s| s.starts_with("permissions"))
                        .unwrap_or(false)
                })
                // Look for a ul in the contents before the next h1/h2
                .for_each(|(i, _)| {
                    children[i + 1..]
                        .iter()
                        // Stop once we hit another h1
                        .take_while(|element| !is_heading(**element))
                        .filter(|element| ul_selector.matches(element))
                        .next()
                        .map(|ul| {
                            ul.select(&li_selector)
                                .map(|element| element.text().collect())
                                .for_each(|li| permissions.push((name.clone(), li)))
                        });
                })
        });

    permissions
}
