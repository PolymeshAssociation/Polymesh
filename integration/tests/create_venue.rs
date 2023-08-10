use anyhow::Result;

use integration::*;
use polymesh_api::types::polymesh_primitives::settlement::{VenueDetails, VenueType};
use polymesh_api_client_extras::*;

#[tokio::test]
async fn create_venue() -> Result<()> {
    let mut tester = PolymeshTester::new().await?;
    let mut users = tester.users(&["CreateVenueUser1"]).await?;

    let mut res1 = tester
        .api
        .call()
        .settlement()
        .create_venue(VenueDetails(b"Test1".to_vec()), vec![], VenueType::Other)?
        .submit_and_watch(&mut users[0])
        .await?;
    let mut res2 = tester
        .api
        .call()
        .settlement()
        .create_venue(VenueDetails(b"Test2".to_vec()), vec![], VenueType::Other)?
        .submit_and_watch(&mut users[0])
        .await?;
    println!("venue1 = {:?}", get_venue_id(&mut res1).await?);
    println!("venue2 = {:?}", get_venue_id(&mut res2).await?);
    Ok(())
}
