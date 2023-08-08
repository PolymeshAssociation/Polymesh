use anyhow::Result;

use integration::*;
use polymesh_api::polymesh::types::polymesh_primitives::settlement::{VenueDetails, VenueType};

#[tokio::test]
async fn create_venue() -> Result<()> {
    let mut tester = PolymeshTester::new().await?;
    let mut users = tester.users(&["CreateVenueUser1"]).await?;

    let mut res = tester
        .api
        .call()
        .settlement()
        .create_venue(VenueDetails(b"Test".to_vec()), vec![], VenueType::Other)?
        .execute(&mut users[0])
        .await?;
    let events = res.events().await?.expect("extrinsic events");
    for rec in &events.0 {
        println!("  - {:?}: {:?}", rec.name(), rec.short_doc());
        match &rec.event {
            RuntimeEvent::Settlement(SettlementEvent::VenueCreated(
                did,
                venue_id,
                details,
                venue_type,
            )) => {
                println!(
                    "    - VenueCreated({:?}, {:?}, {:?}, {:?})",
                    did, venue_id, details, venue_type
                );
            }
            ev => {
                println!("    - other: {ev:?}");
            }
        }
    }
    println!("call1 events = {:#?}", events);
    Ok(())
}
