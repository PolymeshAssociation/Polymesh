#!/usr/bin/env bash

set -e

scriptsArray=(
    "schema_test"
    "0_create_identities"
    "1_poly_transfer"
    "2_key_management"
    "3_auth_join_did"
    "4_permission_management"
    "5_claim_management"
    "6_create_assets"
    "7_create_claim_compliance"
    "8_asset_transfer"
    "10_bridge_transfer"
    "11_governance_management"
    "12_A_settlement"
    "12_B_settlement"
    )

for s in ${scriptsArray[@]}; do
    output=$(npm run $s)
    errorCode=$?

    echo "$output"
    if [[ $errorCode -ne 0 ]] 
    then
        exit $errorCode
    fi

done