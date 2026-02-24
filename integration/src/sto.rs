use polymesh_api::types::polymesh_primitives::sto::FundraiserId;

use crate::*;

/// Get Fundraiser ID from the transaction results.
pub async fn get_fundraiser_id(
    res: &mut TransactionResults,
) -> Result<Option<(AssetId, FundraiserId)>> {
    if let Some(events) = res.events().await? {
        for rec in &events.0 {
            match &rec.event {
                RuntimeEvent::Sto(StoEvent::FundraiserCreated {
                    offering_asset,
                    fundraiser_id,
                    ..
                }) => {
                    return Ok(Some((*offering_asset, fundraiser_id.clone())));
                }
                _ => (),
            }
        }
    }
    Ok(None)
}
