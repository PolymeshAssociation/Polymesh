use frame_support::decl_error;

use polymesh_common_utilities::traits::asset::Config;

use crate::Module;

decl_error! {
    pub enum Error for Module<T: Config> {
        /// The user is not authorized.
        Unauthorized,
        /// The token has already been created.
        AssetAlreadyCreated,
        /// The ticker length is over the limit.
        TickerTooLong,
        /// The ticker has non-alphanumeric parts.
        TickerNotAlphanumeric,
        /// The ticker is already registered to someone else.
        TickerAlreadyRegistered,
        /// The total supply is above the limit.
        TotalSupplyAboveLimit,
        /// No security token associated to the given asset ID.
        NoSuchAsset,
        /// The token is already frozen.
        AlreadyFrozen,
        /// Not an owner of the token on Ethereum.
        NotAnOwner,
        /// An overflow while calculating the balance.
        BalanceOverflow,
        /// An overflow while calculating the total supply.
        TotalSupplyOverflow,
        /// An invalid granularity.
        InvalidGranularity,
        /// The asset must be frozen.
        NotFrozen,
        /// Transfer validation check failed.
        InvalidTransfer,
        /// The sender balance is not sufficient.
        InsufficientBalance,
        /// The token is already divisible.
        AssetAlreadyDivisible,
        /// An invalid Ethereum `EcdsaSignature`.
        InvalidEthereumSignature,
        /// Registration of ticker has expired.
        TickerRegistrationExpired,
        /// Transfers to self are not allowed
        SenderSameAsReceiver,
        /// The given Document does not exist.
        NoSuchDoc,
        /// Maximum length of asset name has been exceeded.
        MaxLengthOfAssetNameExceeded,
        /// Maximum length of the funding round name has been exceeded.
        FundingRoundNameMaxLengthExceeded,
        /// Some `AssetIdentifier` was invalid.
        InvalidAssetIdentifier,
        /// Investor Uniqueness claims are not allowed for this asset.
        InvestorUniquenessClaimNotAllowed,
        /// Invalid `CustomAssetTypeId`.
        InvalidCustomAssetTypeId,
        /// Maximum length of the asset metadata type name has been exceeded.
        AssetMetadataNameMaxLengthExceeded,
        /// Maximum length of the asset metadata value has been exceeded.
        AssetMetadataValueMaxLengthExceeded,
        /// Maximum length of the asset metadata type definition has been exceeded.
        AssetMetadataTypeDefMaxLengthExceeded,
        /// Asset Metadata key is missing.
        AssetMetadataKeyIsMissing,
        /// Asset Metadata value is locked.
        AssetMetadataValueIsLocked,
        /// Asset Metadata Local type already exists for asset.
        AssetMetadataLocalKeyAlreadyExists,
        /// Asset Metadata Global type already exists.
        AssetMetadataGlobalKeyAlreadyExists,
        /// Tickers should start with at least one valid byte.
        TickerFirstByteNotValid,
        /// Attempt to call an extrinsic that is only permitted for fungible tokens.
        UnexpectedNonFungibleToken,
        /// Attempt to update the type of a non fungible token to a fungible token or the other way around.
        IncompatibleAssetTypeUpdate,
        /// Attempt to delete a key that is needed for an NFT collection.
        AssetMetadataKeyBelongsToNFTCollection,
        /// Attempt to lock a metadata value that is empty.
        AssetMetadataValueIsEmpty,
        /// Number of asset mediators would exceed the maximum allowed.
        NumberOfAssetMediatorsExceeded,
        /// Invalid ticker character - valid set: A`..`Z` `0`..`9` `_` `-` `.` `/`.
        InvalidTickerCharacter,
        /// Failed to transfer the asset - asset is frozen.
        InvalidTransferFrozenAsset,
        /// Failed to transfer an NFT - compliance failed.
        InvalidTransferComplianceFailure,
        /// Failed to transfer the asset - receiver cdd is not valid.
        InvalidTransferInvalidReceiverCDD,
        /// Failed to transfer the asset - sender cdd is not valid.
        InvalidTransferInvalidSenderCDD,
        /// The ticker registration associated to the ticker was not found.
        TickerRegistrationNotFound,
        /// The given ticker is already linked to an asset.
        TickerIsAlreadyLinkedToAnAsset,
        /// An unexpected error when generating a new asset ID.
        AssetIDGenerationError,
    }
}
