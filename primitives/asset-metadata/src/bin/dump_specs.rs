use codec::Encode;
use std::io::Write;

use asset_metadata::*;
use polymesh_primitives::asset_metadata::AssetMetadataTypeDef;

fn main() {
    let mut args = std::env::args();
    args.next(); // skip program name.
    let type_name = args.next().unwrap_or("Duration".into());
    let type_def = match type_name.as_str() {
        "Duration" => AssetMetadataTypeDef::new_from_type::<Duration>(),
        "Date" => AssetMetadataTypeDef::new_from_type::<Date>(),
        "Time" => AssetMetadataTypeDef::new_from_type::<Time>(),
        "DateTime" => AssetMetadataTypeDef::new_from_type::<DateTime>(),
        type_name => {
            eprintln!("Unknown type name: {}", type_name);
            return;
        }
    };
    let type_def_filename = args.next().unwrap_or("type_def.bin".into());
    let data = type_def.encode();
    let mut out = std::fs::File::create(&type_def_filename).expect("Failed to create output file");
    out.write_all(&data[..])
        .expect("Failed to write type definition to file.");
    println!(
        "Wrote type definition for {} to file {}",
        type_name, type_def_filename
    );
}
