// Auto-generated via `yarn polkadot-types-from-defs`, do not edit
/* eslint-disable */

/* eslint-disable sort-keys */

export default {
  /**
   * Lookup3: frame_system::AccountInfo<Index, polymesh_common_utilities::traits::balances::AccountData>
   **/
  FrameSystemAccountInfo: {
    nonce: 'u32',
    consumers: 'u32',
    providers: 'u32',
    sufficients: 'u32',
    data: 'PolymeshCommonUtilitiesBalancesAccountData'
  },
  /**
   * Lookup5: polymesh_common_utilities::traits::balances::AccountData
   **/
  PolymeshCommonUtilitiesBalancesAccountData: {
    free: 'u128',
    reserved: 'u128',
    miscFrozen: 'u128',
    feeFrozen: 'u128'
  },
  /**
   * Lookup7: frame_support::weights::PerDispatchClass<T>
   **/
  FrameSupportWeightsPerDispatchClassU64: {
    normal: 'u64',
    operational: 'u64',
    mandatory: 'u64'
  },
  /**
   * Lookup11: sp_runtime::generic::digest::Digest
   **/
  SpRuntimeDigest: {
    logs: 'Vec<SpRuntimeDigestDigestItem>'
  },
  /**
   * Lookup13: sp_runtime::generic::digest::DigestItem
   **/
  SpRuntimeDigestDigestItem: {
    _enum: {
      Other: 'Bytes',
      __Unused1: 'Null',
      __Unused2: 'Null',
      __Unused3: 'Null',
      Consensus: '([u8;4],Bytes)',
      Seal: '([u8;4],Bytes)',
      PreRuntime: '([u8;4],Bytes)',
      __Unused7: 'Null',
      RuntimeEnvironmentUpdated: 'Null'
    }
  },
  /**
   * Lookup16: frame_system::EventRecord<polymesh_runtime_develop::runtime::Event, primitive_types::H256>
   **/
  FrameSystemEventRecord: {
    phase: 'FrameSystemPhase',
    event: 'Event',
    topics: 'Vec<H256>'
  },
  /**
   * Lookup18: frame_system::pallet::Event<T>
   **/
  FrameSystemEvent: {
    _enum: {
      ExtrinsicSuccess: {
        dispatchInfo: 'FrameSupportWeightsDispatchInfo',
      },
      ExtrinsicFailed: {
        dispatchError: 'SpRuntimeDispatchError',
        dispatchInfo: 'FrameSupportWeightsDispatchInfo',
      },
      CodeUpdated: 'Null',
      NewAccount: {
        account: 'AccountId32',
      },
      KilledAccount: {
        account: 'AccountId32',
      },
      Remarked: {
        _alias: {
          hash_: 'hash',
        },
        sender: 'AccountId32',
        hash_: 'H256'
      }
    }
  },
  /**
   * Lookup19: frame_support::weights::DispatchInfo
   **/
  FrameSupportWeightsDispatchInfo: {
    weight: 'u64',
    class: 'FrameSupportWeightsDispatchClass',
    paysFee: 'FrameSupportWeightsPays'
  },
  /**
   * Lookup20: frame_support::weights::DispatchClass
   **/
  FrameSupportWeightsDispatchClass: {
    _enum: ['Normal', 'Operational', 'Mandatory']
  },
  /**
   * Lookup21: frame_support::weights::Pays
   **/
  FrameSupportWeightsPays: {
    _enum: ['Yes', 'No']
  },
  /**
   * Lookup22: sp_runtime::DispatchError
   **/
  SpRuntimeDispatchError: {
    _enum: {
      Other: 'Null',
      CannotLookup: 'Null',
      BadOrigin: 'Null',
      Module: 'SpRuntimeModuleError',
      ConsumerRemaining: 'Null',
      NoProviders: 'Null',
      TooManyConsumers: 'Null',
      Token: 'SpRuntimeTokenError',
      Arithmetic: 'SpRuntimeArithmeticError',
      Transactional: 'SpRuntimeTransactionalError'
    }
  },
  /**
   * Lookup23: sp_runtime::ModuleError
   **/
  SpRuntimeModuleError: {
    index: 'u8',
    error: '[u8;4]'
  },
  /**
   * Lookup24: sp_runtime::TokenError
   **/
  SpRuntimeTokenError: {
    _enum: ['NoFunds', 'WouldDie', 'BelowMinimum', 'CannotCreate', 'UnknownAsset', 'Frozen', 'Unsupported']
  },
  /**
   * Lookup25: sp_runtime::ArithmeticError
   **/
  SpRuntimeArithmeticError: {
    _enum: ['Underflow', 'Overflow', 'DivisionByZero']
  },
  /**
   * Lookup26: sp_runtime::TransactionalError
   **/
  SpRuntimeTransactionalError: {
    _enum: ['LimitReached', 'NoLayer']
  },
  /**
   * Lookup27: pallet_indices::pallet::Event<T>
   **/
  PalletIndicesEvent: {
    _enum: {
      IndexAssigned: {
        who: 'AccountId32',
        index: 'u32',
      },
      IndexFreed: {
        index: 'u32',
      },
      IndexFrozen: {
        index: 'u32',
        who: 'AccountId32'
      }
    }
  },
  /**
   * Lookup28: polymesh_common_utilities::traits::balances::RawEvent<sp_core::crypto::AccountId32>
   **/
  PolymeshCommonUtilitiesBalancesRawEvent: {
    _enum: {
      Endowed: '(Option<PolymeshPrimitivesIdentityId>,AccountId32,u128)',
      Transfer: '(Option<PolymeshPrimitivesIdentityId>,AccountId32,Option<PolymeshPrimitivesIdentityId>,AccountId32,u128,Option<PolymeshCommonUtilitiesBalancesMemo>)',
      BalanceSet: '(PolymeshPrimitivesIdentityId,AccountId32,u128,u128)',
      AccountBalanceBurned: '(PolymeshPrimitivesIdentityId,AccountId32,u128)',
      Reserved: '(AccountId32,u128)',
      Unreserved: '(AccountId32,u128)',
      ReserveRepatriated: '(AccountId32,AccountId32,u128,FrameSupportTokensMiscBalanceStatus)'
    }
  },
  /**
   * Lookup30: polymesh_primitives::identity_id::IdentityId
   **/
  PolymeshPrimitivesIdentityId: '[u8;32]',
  /**
   * Lookup32: polymesh_common_utilities::traits::balances::Memo
   **/
  PolymeshCommonUtilitiesBalancesMemo: '[u8;32]',
  /**
   * Lookup33: frame_support::traits::tokens::misc::BalanceStatus
   **/
  FrameSupportTokensMiscBalanceStatus: {
    _enum: ['Free', 'Reserved']
  },
  /**
   * Lookup34: polymesh_common_utilities::traits::identity::RawEvent<sp_core::crypto::AccountId32, Moment>
   **/
  PolymeshCommonUtilitiesIdentityRawEvent: {
    _enum: {
      DidCreated: '(PolymeshPrimitivesIdentityId,AccountId32,Vec<PolymeshPrimitivesSecondaryKey>)',
      SecondaryKeysAdded: '(PolymeshPrimitivesIdentityId,Vec<PolymeshPrimitivesSecondaryKey>)',
      SecondaryKeysRemoved: '(PolymeshPrimitivesIdentityId,Vec<AccountId32>)',
      SecondaryKeyLeftIdentity: '(PolymeshPrimitivesIdentityId,AccountId32)',
      SecondaryKeyPermissionsUpdated: '(PolymeshPrimitivesIdentityId,AccountId32,PolymeshPrimitivesSecondaryKeyPermissions,PolymeshPrimitivesSecondaryKeyPermissions)',
      PrimaryKeyUpdated: '(PolymeshPrimitivesIdentityId,AccountId32,AccountId32)',
      ClaimAdded: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesIdentityClaim)',
      ClaimRevoked: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesIdentityClaim)',
      AssetDidRegistered: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesTicker)',
      AuthorizationAdded: '(PolymeshPrimitivesIdentityId,Option<PolymeshPrimitivesIdentityId>,Option<AccountId32>,u64,PolymeshPrimitivesAuthorizationAuthorizationData,Option<u64>)',
      AuthorizationRevoked: '(Option<PolymeshPrimitivesIdentityId>,Option<AccountId32>,u64)',
      AuthorizationRejected: '(Option<PolymeshPrimitivesIdentityId>,Option<AccountId32>,u64)',
      AuthorizationConsumed: '(Option<PolymeshPrimitivesIdentityId>,Option<AccountId32>,u64)',
      CddRequirementForPrimaryKeyUpdated: 'bool',
      CddClaimsInvalidated: '(PolymeshPrimitivesIdentityId,u64)',
      SecondaryKeysFrozen: 'PolymeshPrimitivesIdentityId',
      SecondaryKeysUnfrozen: 'PolymeshPrimitivesIdentityId'
    }
  },
  /**
   * Lookup36: polymesh_primitives::secondary_key::SecondaryKey<sp_core::crypto::AccountId32>
   **/
  PolymeshPrimitivesSecondaryKey: {
    key: 'AccountId32',
    permissions: 'PolymeshPrimitivesSecondaryKeyPermissions'
  },
  /**
   * Lookup37: polymesh_primitives::secondary_key::Permissions
   **/
  PolymeshPrimitivesSecondaryKeyPermissions: {
    asset: 'PolymeshPrimitivesSubsetSubsetRestrictionTicker',
    extrinsic: 'PolymeshPrimitivesSubsetSubsetRestrictionPalletPermissions',
    portfolio: 'PolymeshPrimitivesSubsetSubsetRestrictionPortfolioId'
  },
  /**
   * Lookup38: polymesh_primitives::subset::SubsetRestriction<polymesh_primitives::ticker::Ticker>
   **/
  PolymeshPrimitivesSubsetSubsetRestrictionTicker: {
    _enum: {
      Whole: 'Null',
      These: 'BTreeSetTicker',
      Except: 'BTreeSetTicker'
    }
  },
  /**
   * Lookup39: polymesh_primitives::ticker::Ticker
   **/
  PolymeshPrimitivesTicker: '[u8;12]',
  /**
   * Lookup41: BTreeSet<polymesh_primitives::ticker::Ticker>
   **/
  BTreeSetTicker: 'Vec<PolymeshPrimitivesTicker>',
  /**
   * Lookup43: polymesh_primitives::subset::SubsetRestriction<polymesh_primitives::secondary_key::PalletPermissions>
   **/
  PolymeshPrimitivesSubsetSubsetRestrictionPalletPermissions: {
    _enum: {
      Whole: 'Null',
      These: 'BTreeSetPalletPermissions',
      Except: 'BTreeSetPalletPermissions'
    }
  },
  /**
   * Lookup44: polymesh_primitives::secondary_key::PalletPermissions
   **/
  PolymeshPrimitivesSecondaryKeyPalletPermissions: {
    palletName: 'Bytes',
    dispatchableNames: 'PolymeshPrimitivesSubsetSubsetRestrictionDispatchableName'
  },
  /**
   * Lookup46: polymesh_primitives::subset::SubsetRestriction<polymesh_primitives::DispatchableName>
   **/
  PolymeshPrimitivesSubsetSubsetRestrictionDispatchableName: {
    _enum: {
      Whole: 'Null',
      These: 'BTreeSetDispatchableName',
      Except: 'BTreeSetDispatchableName'
    }
  },
  /**
   * Lookup48: BTreeSet<polymesh_primitives::DispatchableName>
   **/
  BTreeSetDispatchableName: 'Vec<Bytes>',
  /**
   * Lookup50: BTreeSet<polymesh_primitives::secondary_key::PalletPermissions>
   **/
  BTreeSetPalletPermissions: 'Vec<PolymeshPrimitivesSecondaryKeyPalletPermissions>',
  /**
   * Lookup52: polymesh_primitives::subset::SubsetRestriction<polymesh_primitives::identity_id::PortfolioId>
   **/
  PolymeshPrimitivesSubsetSubsetRestrictionPortfolioId: {
    _enum: {
      Whole: 'Null',
      These: 'BTreeSetPortfolioId',
      Except: 'BTreeSetPortfolioId'
    }
  },
  /**
   * Lookup53: polymesh_primitives::identity_id::PortfolioId
   **/
  PolymeshPrimitivesIdentityIdPortfolioId: {
    did: 'PolymeshPrimitivesIdentityId',
    kind: 'PolymeshPrimitivesIdentityIdPortfolioKind'
  },
  /**
   * Lookup54: polymesh_primitives::identity_id::PortfolioKind
   **/
  PolymeshPrimitivesIdentityIdPortfolioKind: {
    _enum: {
      Default: 'Null',
      User: 'u64'
    }
  },
  /**
   * Lookup56: BTreeSet<polymesh_primitives::identity_id::PortfolioId>
   **/
  BTreeSetPortfolioId: 'Vec<PolymeshPrimitivesIdentityIdPortfolioId>',
  /**
   * Lookup59: polymesh_primitives::identity_claim::IdentityClaim
   **/
  PolymeshPrimitivesIdentityClaim: {
    claimIssuer: 'PolymeshPrimitivesIdentityId',
    issuanceDate: 'u64',
    lastUpdateDate: 'u64',
    expiry: 'Option<u64>',
    claim: 'PolymeshPrimitivesIdentityClaimClaim'
  },
  /**
   * Lookup61: polymesh_primitives::identity_claim::Claim
   **/
  PolymeshPrimitivesIdentityClaimClaim: {
    _enum: {
      Accredited: 'PolymeshPrimitivesIdentityClaimScope',
      Affiliate: 'PolymeshPrimitivesIdentityClaimScope',
      BuyLockup: 'PolymeshPrimitivesIdentityClaimScope',
      SellLockup: 'PolymeshPrimitivesIdentityClaimScope',
      CustomerDueDiligence: 'PolymeshPrimitivesCddId',
      KnowYourCustomer: 'PolymeshPrimitivesIdentityClaimScope',
      Jurisdiction: '(PolymeshPrimitivesJurisdictionCountryCode,PolymeshPrimitivesIdentityClaimScope)',
      Exempted: 'PolymeshPrimitivesIdentityClaimScope',
      Blocked: 'PolymeshPrimitivesIdentityClaimScope',
      InvestorUniqueness: '(PolymeshPrimitivesIdentityClaimScope,PolymeshPrimitivesIdentityId,PolymeshPrimitivesCddId)',
      NoData: 'Null',
      InvestorUniquenessV2: 'PolymeshPrimitivesCddId'
    }
  },
  /**
   * Lookup62: polymesh_primitives::identity_claim::Scope
   **/
  PolymeshPrimitivesIdentityClaimScope: {
    _enum: {
      Identity: 'PolymeshPrimitivesIdentityId',
      Ticker: 'PolymeshPrimitivesTicker',
      Custom: 'Bytes'
    }
  },
  /**
   * Lookup63: polymesh_primitives::cdd_id::CddId
   **/
  PolymeshPrimitivesCddId: '[u8;32]',
  /**
   * Lookup64: polymesh_primitives::jurisdiction::CountryCode
   **/
  PolymeshPrimitivesJurisdictionCountryCode: {
    _enum: ['AF', 'AX', 'AL', 'DZ', 'AS', 'AD', 'AO', 'AI', 'AQ', 'AG', 'AR', 'AM', 'AW', 'AU', 'AT', 'AZ', 'BS', 'BH', 'BD', 'BB', 'BY', 'BE', 'BZ', 'BJ', 'BM', 'BT', 'BO', 'BA', 'BW', 'BV', 'BR', 'VG', 'IO', 'BN', 'BG', 'BF', 'BI', 'KH', 'CM', 'CA', 'CV', 'KY', 'CF', 'TD', 'CL', 'CN', 'HK', 'MO', 'CX', 'CC', 'CO', 'KM', 'CG', 'CD', 'CK', 'CR', 'CI', 'HR', 'CU', 'CY', 'CZ', 'DK', 'DJ', 'DM', 'DO', 'EC', 'EG', 'SV', 'GQ', 'ER', 'EE', 'ET', 'FK', 'FO', 'FJ', 'FI', 'FR', 'GF', 'PF', 'TF', 'GA', 'GM', 'GE', 'DE', 'GH', 'GI', 'GR', 'GL', 'GD', 'GP', 'GU', 'GT', 'GG', 'GN', 'GW', 'GY', 'HT', 'HM', 'VA', 'HN', 'HU', 'IS', 'IN', 'ID', 'IR', 'IQ', 'IE', 'IM', 'IL', 'IT', 'JM', 'JP', 'JE', 'JO', 'KZ', 'KE', 'KI', 'KP', 'KR', 'KW', 'KG', 'LA', 'LV', 'LB', 'LS', 'LR', 'LY', 'LI', 'LT', 'LU', 'MK', 'MG', 'MW', 'MY', 'MV', 'ML', 'MT', 'MH', 'MQ', 'MR', 'MU', 'YT', 'MX', 'FM', 'MD', 'MC', 'MN', 'ME', 'MS', 'MA', 'MZ', 'MM', 'NA', 'NR', 'NP', 'NL', 'AN', 'NC', 'NZ', 'NI', 'NE', 'NG', 'NU', 'NF', 'MP', 'NO', 'OM', 'PK', 'PW', 'PS', 'PA', 'PG', 'PY', 'PE', 'PH', 'PN', 'PL', 'PT', 'PR', 'QA', 'RE', 'RO', 'RU', 'RW', 'BL', 'SH', 'KN', 'LC', 'MF', 'PM', 'VC', 'WS', 'SM', 'ST', 'SA', 'SN', 'RS', 'SC', 'SL', 'SG', 'SK', 'SI', 'SB', 'SO', 'ZA', 'GS', 'SS', 'ES', 'LK', 'SD', 'SR', 'SJ', 'SZ', 'SE', 'CH', 'SY', 'TW', 'TJ', 'TZ', 'TH', 'TL', 'TG', 'TK', 'TO', 'TT', 'TN', 'TR', 'TM', 'TC', 'TV', 'UG', 'UA', 'AE', 'GB', 'US', 'UM', 'UY', 'UZ', 'VU', 'VE', 'VN', 'VI', 'WF', 'EH', 'YE', 'ZM', 'ZW', 'BQ', 'CW', 'SX']
  },
  /**
   * Lookup66: polymesh_primitives::authorization::AuthorizationData<sp_core::crypto::AccountId32>
   **/
  PolymeshPrimitivesAuthorizationAuthorizationData: {
    _enum: {
      AttestPrimaryKeyRotation: 'PolymeshPrimitivesIdentityId',
      RotatePrimaryKey: 'Null',
      TransferTicker: 'PolymeshPrimitivesTicker',
      AddMultiSigSigner: 'AccountId32',
      TransferAssetOwnership: 'PolymeshPrimitivesTicker',
      JoinIdentity: 'PolymeshPrimitivesSecondaryKeyPermissions',
      PortfolioCustody: 'PolymeshPrimitivesIdentityIdPortfolioId',
      BecomeAgent: '(PolymeshPrimitivesTicker,PolymeshPrimitivesAgentAgentGroup)',
      AddRelayerPayingKey: '(AccountId32,AccountId32,u128)',
      RotatePrimaryKeyToSecondary: 'PolymeshPrimitivesSecondaryKeyPermissions'
    }
  },
  /**
   * Lookup67: polymesh_primitives::agent::AgentGroup
   **/
  PolymeshPrimitivesAgentAgentGroup: {
    _enum: {
      Full: 'Null',
      Custom: 'u32',
      ExceptMeta: 'Null',
      PolymeshV1CAA: 'Null',
      PolymeshV1PIA: 'Null'
    }
  },
  /**
   * Lookup70: polymesh_common_utilities::traits::group::RawEvent<sp_core::crypto::AccountId32, polymesh_runtime_develop::runtime::Event, pallet_group::Instance2>
   **/
  PolymeshCommonUtilitiesGroupRawEventInstance2: {
    _enum: {
      MemberAdded: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesIdentityId)',
      MemberRemoved: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesIdentityId)',
      MemberRevoked: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesIdentityId)',
      MembersSwapped: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesIdentityId,PolymeshPrimitivesIdentityId)',
      MembersReset: '(PolymeshPrimitivesIdentityId,Vec<PolymeshPrimitivesIdentityId>)',
      ActiveLimitChanged: '(PolymeshPrimitivesIdentityId,u32,u32)',
      Dummy: 'Null'
    }
  },
  /**
   * Lookup71: pallet_group::Instance2
   **/
  PalletGroupInstance2: 'Null',
  /**
   * Lookup73: pallet_committee::RawEvent<primitive_types::H256, BlockNumber, pallet_committee::Instance1>
   **/
  PalletCommitteeRawEventInstance1: {
    _enum: {
      Proposed: '(PolymeshPrimitivesIdentityId,u32,H256)',
      Voted: '(PolymeshPrimitivesIdentityId,u32,H256,bool,u32,u32,u32)',
      VoteRetracted: '(PolymeshPrimitivesIdentityId,u32,H256,bool)',
      FinalVotes: '(PolymeshPrimitivesIdentityId,u32,H256,Vec<PolymeshPrimitivesIdentityId>,Vec<PolymeshPrimitivesIdentityId>)',
      Approved: '(PolymeshPrimitivesIdentityId,H256,u32,u32,u32)',
      Rejected: '(PolymeshPrimitivesIdentityId,H256,u32,u32,u32)',
      Executed: '(PolymeshPrimitivesIdentityId,H256,Result<Null, SpRuntimeDispatchError>)',
      ReleaseCoordinatorUpdated: '(PolymeshPrimitivesIdentityId,Option<PolymeshPrimitivesIdentityId>)',
      ExpiresAfterUpdated: '(PolymeshPrimitivesIdentityId,PolymeshCommonUtilitiesMaybeBlock)',
      VoteThresholdUpdated: '(PolymeshPrimitivesIdentityId,u32,u32)'
    }
  },
  /**
   * Lookup74: pallet_committee::Instance1
   **/
  PalletCommitteeInstance1: 'Null',
  /**
   * Lookup77: polymesh_common_utilities::MaybeBlock<BlockNumber>
   **/
  PolymeshCommonUtilitiesMaybeBlock: {
    _enum: {
      Some: 'u32',
      None: 'Null'
    }
  },
  /**
   * Lookup78: polymesh_common_utilities::traits::group::RawEvent<sp_core::crypto::AccountId32, polymesh_runtime_develop::runtime::Event, pallet_group::Instance1>
   **/
  PolymeshCommonUtilitiesGroupRawEventInstance1: {
    _enum: {
      MemberAdded: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesIdentityId)',
      MemberRemoved: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesIdentityId)',
      MemberRevoked: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesIdentityId)',
      MembersSwapped: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesIdentityId,PolymeshPrimitivesIdentityId)',
      MembersReset: '(PolymeshPrimitivesIdentityId,Vec<PolymeshPrimitivesIdentityId>)',
      ActiveLimitChanged: '(PolymeshPrimitivesIdentityId,u32,u32)',
      Dummy: 'Null'
    }
  },
  /**
   * Lookup79: pallet_group::Instance1
   **/
  PalletGroupInstance1: 'Null',
  /**
   * Lookup80: pallet_committee::RawEvent<primitive_types::H256, BlockNumber, pallet_committee::Instance3>
   **/
  PalletCommitteeRawEventInstance3: {
    _enum: {
      Proposed: '(PolymeshPrimitivesIdentityId,u32,H256)',
      Voted: '(PolymeshPrimitivesIdentityId,u32,H256,bool,u32,u32,u32)',
      VoteRetracted: '(PolymeshPrimitivesIdentityId,u32,H256,bool)',
      FinalVotes: '(PolymeshPrimitivesIdentityId,u32,H256,Vec<PolymeshPrimitivesIdentityId>,Vec<PolymeshPrimitivesIdentityId>)',
      Approved: '(PolymeshPrimitivesIdentityId,H256,u32,u32,u32)',
      Rejected: '(PolymeshPrimitivesIdentityId,H256,u32,u32,u32)',
      Executed: '(PolymeshPrimitivesIdentityId,H256,Result<Null, SpRuntimeDispatchError>)',
      ReleaseCoordinatorUpdated: '(PolymeshPrimitivesIdentityId,Option<PolymeshPrimitivesIdentityId>)',
      ExpiresAfterUpdated: '(PolymeshPrimitivesIdentityId,PolymeshCommonUtilitiesMaybeBlock)',
      VoteThresholdUpdated: '(PolymeshPrimitivesIdentityId,u32,u32)'
    }
  },
  /**
   * Lookup81: pallet_committee::Instance3
   **/
  PalletCommitteeInstance3: 'Null',
  /**
   * Lookup82: polymesh_common_utilities::traits::group::RawEvent<sp_core::crypto::AccountId32, polymesh_runtime_develop::runtime::Event, pallet_group::Instance3>
   **/
  PolymeshCommonUtilitiesGroupRawEventInstance3: {
    _enum: {
      MemberAdded: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesIdentityId)',
      MemberRemoved: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesIdentityId)',
      MemberRevoked: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesIdentityId)',
      MembersSwapped: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesIdentityId,PolymeshPrimitivesIdentityId)',
      MembersReset: '(PolymeshPrimitivesIdentityId,Vec<PolymeshPrimitivesIdentityId>)',
      ActiveLimitChanged: '(PolymeshPrimitivesIdentityId,u32,u32)',
      Dummy: 'Null'
    }
  },
  /**
   * Lookup83: pallet_group::Instance3
   **/
  PalletGroupInstance3: 'Null',
  /**
   * Lookup84: pallet_committee::RawEvent<primitive_types::H256, BlockNumber, pallet_committee::Instance4>
   **/
  PalletCommitteeRawEventInstance4: {
    _enum: {
      Proposed: '(PolymeshPrimitivesIdentityId,u32,H256)',
      Voted: '(PolymeshPrimitivesIdentityId,u32,H256,bool,u32,u32,u32)',
      VoteRetracted: '(PolymeshPrimitivesIdentityId,u32,H256,bool)',
      FinalVotes: '(PolymeshPrimitivesIdentityId,u32,H256,Vec<PolymeshPrimitivesIdentityId>,Vec<PolymeshPrimitivesIdentityId>)',
      Approved: '(PolymeshPrimitivesIdentityId,H256,u32,u32,u32)',
      Rejected: '(PolymeshPrimitivesIdentityId,H256,u32,u32,u32)',
      Executed: '(PolymeshPrimitivesIdentityId,H256,Result<Null, SpRuntimeDispatchError>)',
      ReleaseCoordinatorUpdated: '(PolymeshPrimitivesIdentityId,Option<PolymeshPrimitivesIdentityId>)',
      ExpiresAfterUpdated: '(PolymeshPrimitivesIdentityId,PolymeshCommonUtilitiesMaybeBlock)',
      VoteThresholdUpdated: '(PolymeshPrimitivesIdentityId,u32,u32)'
    }
  },
  /**
   * Lookup85: pallet_committee::Instance4
   **/
  PalletCommitteeInstance4: 'Null',
  /**
   * Lookup86: polymesh_common_utilities::traits::group::RawEvent<sp_core::crypto::AccountId32, polymesh_runtime_develop::runtime::Event, pallet_group::Instance4>
   **/
  PolymeshCommonUtilitiesGroupRawEventInstance4: {
    _enum: {
      MemberAdded: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesIdentityId)',
      MemberRemoved: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesIdentityId)',
      MemberRevoked: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesIdentityId)',
      MembersSwapped: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesIdentityId,PolymeshPrimitivesIdentityId)',
      MembersReset: '(PolymeshPrimitivesIdentityId,Vec<PolymeshPrimitivesIdentityId>)',
      ActiveLimitChanged: '(PolymeshPrimitivesIdentityId,u32,u32)',
      Dummy: 'Null'
    }
  },
  /**
   * Lookup87: pallet_group::Instance4
   **/
  PalletGroupInstance4: 'Null',
  /**
   * Lookup88: pallet_multisig::RawEvent<sp_core::crypto::AccountId32>
   **/
  PalletMultisigRawEvent: {
    _enum: {
      MultiSigCreated: '(PolymeshPrimitivesIdentityId,AccountId32,AccountId32,Vec<PolymeshPrimitivesSecondaryKeySignatory>,u64)',
      ProposalAdded: '(PolymeshPrimitivesIdentityId,AccountId32,u64)',
      ProposalExecuted: '(PolymeshPrimitivesIdentityId,AccountId32,u64,bool)',
      MultiSigSignerAdded: '(PolymeshPrimitivesIdentityId,AccountId32,PolymeshPrimitivesSecondaryKeySignatory)',
      MultiSigSignerAuthorized: '(PolymeshPrimitivesIdentityId,AccountId32,PolymeshPrimitivesSecondaryKeySignatory)',
      MultiSigSignerRemoved: '(PolymeshPrimitivesIdentityId,AccountId32,PolymeshPrimitivesSecondaryKeySignatory)',
      MultiSigSignaturesRequiredChanged: '(PolymeshPrimitivesIdentityId,AccountId32,u64)',
      ProposalApproved: '(PolymeshPrimitivesIdentityId,AccountId32,PolymeshPrimitivesSecondaryKeySignatory,u64)',
      ProposalRejectionVote: '(PolymeshPrimitivesIdentityId,AccountId32,PolymeshPrimitivesSecondaryKeySignatory,u64)',
      ProposalRejected: '(PolymeshPrimitivesIdentityId,AccountId32,u64)',
      ProposalExecutionFailed: 'SpRuntimeDispatchError',
      SchedulingFailed: 'SpRuntimeDispatchError'
    }
  },
  /**
   * Lookup90: polymesh_primitives::secondary_key::Signatory<sp_core::crypto::AccountId32>
   **/
  PolymeshPrimitivesSecondaryKeySignatory: {
    _enum: {
      Identity: 'PolymeshPrimitivesIdentityId',
      Account: 'AccountId32'
    }
  },
  /**
   * Lookup91: pallet_bridge::RawEvent<sp_core::crypto::AccountId32, BlockNumber>
   **/
  PalletBridgeRawEvent: {
    _enum: {
      ControllerChanged: '(PolymeshPrimitivesIdentityId,AccountId32)',
      AdminChanged: '(PolymeshPrimitivesIdentityId,AccountId32)',
      TimelockChanged: '(PolymeshPrimitivesIdentityId,u32)',
      Bridged: '(PolymeshPrimitivesIdentityId,PalletBridgeBridgeTx)',
      Frozen: 'PolymeshPrimitivesIdentityId',
      Unfrozen: 'PolymeshPrimitivesIdentityId',
      FrozenTx: '(PolymeshPrimitivesIdentityId,PalletBridgeBridgeTx)',
      UnfrozenTx: '(PolymeshPrimitivesIdentityId,PalletBridgeBridgeTx)',
      ExemptedUpdated: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesIdentityId,bool)',
      BridgeLimitUpdated: '(PolymeshPrimitivesIdentityId,u128,u32)',
      TxsHandled: 'Vec<(AccountId32,u32,PalletBridgeHandledTxStatus)>',
      BridgeTxScheduled: '(PolymeshPrimitivesIdentityId,PalletBridgeBridgeTx,u32)',
      BridgeTxScheduleFailed: '(PolymeshPrimitivesIdentityId,PalletBridgeBridgeTx,Bytes)',
      FreezeAdminAdded: '(PolymeshPrimitivesIdentityId,AccountId32)',
      FreezeAdminRemoved: '(PolymeshPrimitivesIdentityId,AccountId32)',
      TxRemoved: '(PolymeshPrimitivesIdentityId,PalletBridgeBridgeTx)'
    }
  },
  /**
   * Lookup92: pallet_bridge::BridgeTx<sp_core::crypto::AccountId32>
   **/
  PalletBridgeBridgeTx: {
    nonce: 'u32',
    recipient: 'AccountId32',
    amount: 'u128',
    txHash: 'H256'
  },
  /**
   * Lookup95: pallet_bridge::HandledTxStatus
   **/
  PalletBridgeHandledTxStatus: {
    _enum: {
      Success: 'Null',
      Error: 'Bytes'
    }
  },
  /**
   * Lookup96: pallet_staking::RawEvent<Balance, sp_core::crypto::AccountId32>
   **/
  PalletStakingRawEvent: {
    _enum: {
      EraPayout: '(u32,u128,u128)',
      Reward: '(PolymeshPrimitivesIdentityId,AccountId32,u128)',
      Slash: '(AccountId32,u128)',
      OldSlashingReportDiscarded: 'u32',
      StakingElection: 'PalletStakingElectionCompute',
      SolutionStored: 'PalletStakingElectionCompute',
      Bonded: '(PolymeshPrimitivesIdentityId,AccountId32,u128)',
      Unbonded: '(PolymeshPrimitivesIdentityId,AccountId32,u128)',
      Nominated: '(PolymeshPrimitivesIdentityId,AccountId32,Vec<AccountId32>)',
      Withdrawn: '(AccountId32,u128)',
      PermissionedIdentityAdded: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesIdentityId)',
      PermissionedIdentityRemoved: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesIdentityId)',
      InvalidatedNominators: '(PolymeshPrimitivesIdentityId,AccountId32,Vec<AccountId32>)',
      CommissionCapUpdated: '(PolymeshPrimitivesIdentityId,Perbill,Perbill)',
      MinimumBondThresholdUpdated: '(Option<PolymeshPrimitivesIdentityId>,u128)',
      RewardPaymentSchedulingInterrupted: '(AccountId32,u32,SpRuntimeDispatchError)',
      SlashingAllowedForChanged: 'PalletStakingSlashingSwitch'
    }
  },
  /**
   * Lookup97: pallet_staking::ElectionCompute
   **/
  PalletStakingElectionCompute: {
    _enum: ['OnChain', 'Signed', 'Unsigned']
  },
  /**
   * Lookup99: pallet_staking::SlashingSwitch
   **/
  PalletStakingSlashingSwitch: {
    _enum: ['Validator', 'ValidatorAndNominator', 'None']
  },
  /**
   * Lookup100: pallet_offences::pallet::Event
   **/
  PalletOffencesEvent: {
    _enum: {
      Offence: {
        kind: '[u8;16]',
        timeslot: 'Bytes'
      }
    }
  },
  /**
   * Lookup102: pallet_session::pallet::Event
   **/
  PalletSessionEvent: {
    _enum: {
      NewSession: {
        sessionIndex: 'u32'
      }
    }
  },
  /**
   * Lookup103: pallet_grandpa::pallet::Event
   **/
  PalletGrandpaEvent: {
    _enum: {
      NewAuthorities: {
        authoritySet: 'Vec<(SpFinalityGrandpaAppPublic,u64)>',
      },
      Paused: 'Null',
      Resumed: 'Null'
    }
  },
  /**
   * Lookup106: sp_finality_grandpa::app::Public
   **/
  SpFinalityGrandpaAppPublic: 'SpCoreEd25519Public',
  /**
   * Lookup107: sp_core::ed25519::Public
   **/
  SpCoreEd25519Public: '[u8;32]',
  /**
   * Lookup108: pallet_im_online::pallet::Event<T>
   **/
  PalletImOnlineEvent: {
    _enum: {
      HeartbeatReceived: {
        authorityId: 'PalletImOnlineSr25519AppSr25519Public',
      },
      AllGood: 'Null',
      SomeOffline: {
        offline: 'Vec<(AccountId32,PalletStakingExposure)>'
      }
    }
  },
  /**
   * Lookup109: pallet_im_online::sr25519::app_sr25519::Public
   **/
  PalletImOnlineSr25519AppSr25519Public: 'SpCoreSr25519Public',
  /**
   * Lookup110: sp_core::sr25519::Public
   **/
  SpCoreSr25519Public: '[u8;32]',
  /**
   * Lookup113: pallet_staking::Exposure<sp_core::crypto::AccountId32, Balance>
   **/
  PalletStakingExposure: {
    total: 'Compact<u128>',
    own: 'Compact<u128>',
    others: 'Vec<PalletStakingIndividualExposure>'
  },
  /**
   * Lookup116: pallet_staking::IndividualExposure<sp_core::crypto::AccountId32, Balance>
   **/
  PalletStakingIndividualExposure: {
    who: 'AccountId32',
    value: 'Compact<u128>'
  },
  /**
   * Lookup117: pallet_sudo::RawEvent<sp_core::crypto::AccountId32>
   **/
  PalletSudoRawEvent: {
    _enum: {
      Sudid: 'Result<Null, SpRuntimeDispatchError>',
      KeyChanged: 'AccountId32',
      SudoAsDone: 'Result<Null, SpRuntimeDispatchError>'
    }
  },
  /**
   * Lookup118: polymesh_common_utilities::traits::asset::RawEvent<Moment, sp_core::crypto::AccountId32>
   **/
  PolymeshCommonUtilitiesAssetRawEvent: {
    _enum: {
      Transfer: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesTicker,PolymeshPrimitivesIdentityIdPortfolioId,PolymeshPrimitivesIdentityIdPortfolioId,u128)',
      Issued: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesTicker,PolymeshPrimitivesIdentityId,u128,Bytes,u128)',
      Redeemed: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesTicker,PolymeshPrimitivesIdentityId,u128)',
      AssetCreated: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesTicker,bool,PolymeshPrimitivesAssetAssetType,PolymeshPrimitivesIdentityId,bool)',
      IdentifiersUpdated: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesTicker,Vec<PolymeshPrimitivesAssetIdentifier>)',
      DivisibilityChanged: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesTicker,bool)',
      TransferWithData: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesTicker,PolymeshPrimitivesIdentityId,PolymeshPrimitivesIdentityId,u128,Bytes)',
      IsIssuable: '(PolymeshPrimitivesTicker,bool)',
      TickerRegistered: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesTicker,Option<u64>)',
      TickerTransferred: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesTicker,PolymeshPrimitivesIdentityId)',
      AssetOwnershipTransferred: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesTicker,PolymeshPrimitivesIdentityId)',
      AssetFrozen: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesTicker)',
      AssetUnfrozen: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesTicker)',
      AssetRenamed: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesTicker,Bytes)',
      FundingRoundSet: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesTicker,Bytes)',
      DocumentAdded: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesTicker,u32,PolymeshPrimitivesDocument)',
      DocumentRemoved: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesTicker,u32)',
      ExtensionRemoved: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesTicker,AccountId32)',
      ClassicTickerClaimed: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesTicker,PolymeshPrimitivesEthereumEthereumAddress)',
      ControllerTransfer: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesTicker,PolymeshPrimitivesIdentityIdPortfolioId,u128)',
      CustomAssetTypeExists: '(PolymeshPrimitivesIdentityId,u32,Bytes)',
      CustomAssetTypeRegistered: '(PolymeshPrimitivesIdentityId,u32,Bytes)',
      SetAssetMetadataValue: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesTicker,Bytes,Option<PolymeshPrimitivesAssetMetadataAssetMetadataValueDetail>)',
      SetAssetMetadataValueDetails: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesTicker,PolymeshPrimitivesAssetMetadataAssetMetadataValueDetail)',
      RegisterAssetMetadataLocalType: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesTicker,Bytes,u64,PolymeshPrimitivesAssetMetadataAssetMetadataSpec)',
      RegisterAssetMetadataGlobalType: '(Bytes,u64,PolymeshPrimitivesAssetMetadataAssetMetadataSpec)'
    }
  },
  /**
   * Lookup120: polymesh_primitives::asset::AssetType
   **/
  PolymeshPrimitivesAssetAssetType: {
    _enum: {
      EquityCommon: 'Null',
      EquityPreferred: 'Null',
      Commodity: 'Null',
      FixedIncome: 'Null',
      REIT: 'Null',
      Fund: 'Null',
      RevenueShareAgreement: 'Null',
      StructuredProduct: 'Null',
      Derivative: 'Null',
      Custom: 'u32',
      StableCoin: 'Null'
    }
  },
  /**
   * Lookup123: polymesh_primitives::asset_identifier::AssetIdentifier
   **/
  PolymeshPrimitivesAssetIdentifier: {
    _enum: {
      CUSIP: '[u8;9]',
      CINS: '[u8;9]',
      ISIN: '[u8;12]',
      LEI: '[u8;20]',
      FIGI: '[u8;12]'
    }
  },
  /**
   * Lookup128: polymesh_primitives::document::Document
   **/
  PolymeshPrimitivesDocument: {
    uri: 'Bytes',
    contentHash: 'PolymeshPrimitivesDocumentHash',
    name: 'Bytes',
    docType: 'Option<Bytes>',
    filingDate: 'Option<u64>'
  },
  /**
   * Lookup130: polymesh_primitives::document_hash::DocumentHash
   **/
  PolymeshPrimitivesDocumentHash: {
    _enum: {
      None: 'Null',
      H512: '[u8;64]',
      H384: '[u8;48]',
      H320: '[u8;40]',
      H256: '[u8;32]',
      H224: '[u8;28]',
      H192: '[u8;24]',
      H160: '[u8;20]',
      H128: '[u8;16]'
    }
  },
  /**
   * Lookup139: polymesh_primitives::ethereum::EthereumAddress
   **/
  PolymeshPrimitivesEthereumEthereumAddress: '[u8;20]',
  /**
   * Lookup142: polymesh_primitives::asset_metadata::AssetMetadataValueDetail<Moment>
   **/
  PolymeshPrimitivesAssetMetadataAssetMetadataValueDetail: {
    expire: 'Option<u64>',
    lockStatus: 'PolymeshPrimitivesAssetMetadataAssetMetadataLockStatus'
  },
  /**
   * Lookup143: polymesh_primitives::asset_metadata::AssetMetadataLockStatus<Moment>
   **/
  PolymeshPrimitivesAssetMetadataAssetMetadataLockStatus: {
    _enum: {
      Unlocked: 'Null',
      Locked: 'Null',
      LockedUntil: 'u64'
    }
  },
  /**
   * Lookup146: polymesh_primitives::asset_metadata::AssetMetadataSpec
   **/
  PolymeshPrimitivesAssetMetadataAssetMetadataSpec: {
    url: 'Option<Bytes>',
    description: 'Option<Bytes>',
    typeDef: 'Option<Bytes>'
  },
  /**
   * Lookup153: pallet_corporate_actions::distribution::Event
   **/
  PalletCorporateActionsDistributionEvent: {
    _enum: {
      Created: '(PolymeshPrimitivesEventOnly,PalletCorporateActionsCaId,PalletCorporateActionsDistribution)',
      BenefitClaimed: '(PolymeshPrimitivesEventOnly,PolymeshPrimitivesEventOnly,PalletCorporateActionsCaId,PalletCorporateActionsDistribution,u128,Permill)',
      Reclaimed: '(PolymeshPrimitivesEventOnly,PalletCorporateActionsCaId,u128)',
      Removed: '(PolymeshPrimitivesEventOnly,PalletCorporateActionsCaId)'
    }
  },
  /**
   * Lookup154: polymesh_primitives::event_only::EventOnly<polymesh_primitives::identity_id::IdentityId>
   **/
  PolymeshPrimitivesEventOnly: 'PolymeshPrimitivesIdentityId',
  /**
   * Lookup155: pallet_corporate_actions::CAId
   **/
  PalletCorporateActionsCaId: {
    ticker: 'PolymeshPrimitivesTicker',
    localId: 'u32'
  },
  /**
   * Lookup157: pallet_corporate_actions::distribution::Distribution
   **/
  PalletCorporateActionsDistribution: {
    from: 'PolymeshPrimitivesIdentityIdPortfolioId',
    currency: 'PolymeshPrimitivesTicker',
    perShare: 'u128',
    amount: 'u128',
    remaining: 'u128',
    reclaimed: 'bool',
    paymentAt: 'u64',
    expiresAt: 'Option<u64>'
  },
  /**
   * Lookup159: polymesh_common_utilities::traits::checkpoint::Event
   **/
  PolymeshCommonUtilitiesCheckpointEvent: {
    _enum: {
      CheckpointCreated: '(Option<PolymeshPrimitivesEventOnly>,PolymeshPrimitivesTicker,u64,u128,u64)',
      MaximumSchedulesComplexityChanged: '(PolymeshPrimitivesIdentityId,u64)',
      ScheduleCreated: '(PolymeshPrimitivesEventOnly,PolymeshPrimitivesTicker,PolymeshCommonUtilitiesCheckpointStoredSchedule)',
      ScheduleRemoved: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesTicker,PolymeshCommonUtilitiesCheckpointStoredSchedule)'
    }
  },
  /**
   * Lookup162: polymesh_common_utilities::traits::checkpoint::StoredSchedule
   **/
  PolymeshCommonUtilitiesCheckpointStoredSchedule: {
    schedule: 'PolymeshPrimitivesCalendarCheckpointSchedule',
    id: 'u64',
    at: 'u64',
    remaining: 'u32'
  },
  /**
   * Lookup163: polymesh_primitives::calendar::CheckpointSchedule
   **/
  PolymeshPrimitivesCalendarCheckpointSchedule: {
    start: 'u64',
    period: 'PolymeshPrimitivesCalendarCalendarPeriod'
  },
  /**
   * Lookup164: polymesh_primitives::calendar::CalendarPeriod
   **/
  PolymeshPrimitivesCalendarCalendarPeriod: {
    unit: 'PolymeshPrimitivesCalendarCalendarUnit',
    amount: 'u64'
  },
  /**
   * Lookup165: polymesh_primitives::calendar::CalendarUnit
   **/
  PolymeshPrimitivesCalendarCalendarUnit: {
    _enum: ['Second', 'Minute', 'Hour', 'Day', 'Week', 'Month', 'Year']
  },
  /**
   * Lookup167: pallet_compliance_manager::Event
   **/
  PalletComplianceManagerEvent: {
    _enum: {
      ComplianceRequirementCreated: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesTicker,PolymeshPrimitivesComplianceManagerComplianceRequirement)',
      ComplianceRequirementRemoved: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesTicker,u32)',
      AssetComplianceReplaced: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesTicker,Vec<PolymeshPrimitivesComplianceManagerComplianceRequirement>)',
      AssetComplianceReset: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesTicker)',
      AssetComplianceResumed: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesTicker)',
      AssetCompliancePaused: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesTicker)',
      ComplianceRequirementChanged: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesTicker,PolymeshPrimitivesComplianceManagerComplianceRequirement)',
      TrustedDefaultClaimIssuerAdded: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesTicker,PolymeshPrimitivesConditionTrustedIssuer)',
      TrustedDefaultClaimIssuerRemoved: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesTicker,PolymeshPrimitivesIdentityId)'
    }
  },
  /**
   * Lookup168: polymesh_primitives::compliance_manager::ComplianceRequirement
   **/
  PolymeshPrimitivesComplianceManagerComplianceRequirement: {
    senderConditions: 'Vec<PolymeshPrimitivesCondition>',
    receiverConditions: 'Vec<PolymeshPrimitivesCondition>',
    id: 'u32'
  },
  /**
   * Lookup170: polymesh_primitives::condition::Condition
   **/
  PolymeshPrimitivesCondition: {
    conditionType: 'PolymeshPrimitivesConditionConditionType',
    issuers: 'Vec<PolymeshPrimitivesConditionTrustedIssuer>'
  },
  /**
   * Lookup171: polymesh_primitives::condition::ConditionType
   **/
  PolymeshPrimitivesConditionConditionType: {
    _enum: {
      IsPresent: 'PolymeshPrimitivesIdentityClaimClaim',
      IsAbsent: 'PolymeshPrimitivesIdentityClaimClaim',
      IsAnyOf: 'Vec<PolymeshPrimitivesIdentityClaimClaim>',
      IsNoneOf: 'Vec<PolymeshPrimitivesIdentityClaimClaim>',
      IsIdentity: 'PolymeshPrimitivesConditionTargetIdentity'
    }
  },
  /**
   * Lookup173: polymesh_primitives::condition::TargetIdentity
   **/
  PolymeshPrimitivesConditionTargetIdentity: {
    _enum: {
      ExternalAgent: 'Null',
      Specific: 'PolymeshPrimitivesIdentityId'
    }
  },
  /**
   * Lookup175: polymesh_primitives::condition::TrustedIssuer
   **/
  PolymeshPrimitivesConditionTrustedIssuer: {
    issuer: 'PolymeshPrimitivesIdentityId',
    trustedFor: 'PolymeshPrimitivesConditionTrustedFor'
  },
  /**
   * Lookup176: polymesh_primitives::condition::TrustedFor
   **/
  PolymeshPrimitivesConditionTrustedFor: {
    _enum: {
      Any: 'Null',
      Specific: 'Vec<PolymeshPrimitivesIdentityClaimClaimType>'
    }
  },
  /**
   * Lookup178: polymesh_primitives::identity_claim::ClaimType
   **/
  PolymeshPrimitivesIdentityClaimClaimType: {
    _enum: ['Accredited', 'Affiliate', 'BuyLockup', 'SellLockup', 'CustomerDueDiligence', 'KnowYourCustomer', 'Jurisdiction', 'Exempted', 'Blocked', 'InvestorUniqueness', 'NoType', 'InvestorUniquenessV2']
  },
  /**
   * Lookup180: pallet_corporate_actions::Event
   **/
  PalletCorporateActionsEvent: {
    _enum: {
      MaxDetailsLengthChanged: '(PolymeshPrimitivesIdentityId,u32)',
      DefaultTargetIdentitiesChanged: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesTicker,PalletCorporateActionsTargetIdentities)',
      DefaultWithholdingTaxChanged: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesTicker,Permill)',
      DidWithholdingTaxChanged: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesTicker,PolymeshPrimitivesIdentityId,Option<Permill>)',
      CAATransferred: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesTicker,PolymeshPrimitivesIdentityId)',
      CAInitiated: '(PolymeshPrimitivesEventOnly,PalletCorporateActionsCaId,PalletCorporateActionsCorporateAction,Bytes)',
      CALinkedToDoc: '(PolymeshPrimitivesIdentityId,PalletCorporateActionsCaId,Vec<u32>)',
      CARemoved: '(PolymeshPrimitivesEventOnly,PalletCorporateActionsCaId)',
      RecordDateChanged: '(PolymeshPrimitivesEventOnly,PalletCorporateActionsCaId,PalletCorporateActionsCorporateAction)'
    }
  },
  /**
   * Lookup181: pallet_corporate_actions::TargetIdentities
   **/
  PalletCorporateActionsTargetIdentities: {
    identities: 'Vec<PolymeshPrimitivesIdentityId>',
    treatment: 'PalletCorporateActionsTargetTreatment'
  },
  /**
   * Lookup182: pallet_corporate_actions::TargetTreatment
   **/
  PalletCorporateActionsTargetTreatment: {
    _enum: ['Include', 'Exclude']
  },
  /**
   * Lookup184: pallet_corporate_actions::CorporateAction
   **/
  PalletCorporateActionsCorporateAction: {
    kind: 'PalletCorporateActionsCaKind',
    declDate: 'u64',
    recordDate: 'Option<PalletCorporateActionsRecordDate>',
    targets: 'PalletCorporateActionsTargetIdentities',
    defaultWithholdingTax: 'Permill',
    withholdingTax: 'Vec<(PolymeshPrimitivesIdentityId,Permill)>'
  },
  /**
   * Lookup185: pallet_corporate_actions::CAKind
   **/
  PalletCorporateActionsCaKind: {
    _enum: ['PredictableBenefit', 'UnpredictableBenefit', 'IssuerNotice', 'Reorganization', 'Other']
  },
  /**
   * Lookup187: pallet_corporate_actions::RecordDate
   **/
  PalletCorporateActionsRecordDate: {
    date: 'u64',
    checkpoint: 'PalletCorporateActionsCaCheckpoint'
  },
  /**
   * Lookup188: pallet_corporate_actions::CACheckpoint
   **/
  PalletCorporateActionsCaCheckpoint: {
    _enum: {
      Scheduled: '(u64,u64)',
      Existing: 'u64'
    }
  },
  /**
   * Lookup193: pallet_corporate_actions::ballot::Event
   **/
  PalletCorporateActionsBallotEvent: {
    _enum: {
      Created: '(PolymeshPrimitivesIdentityId,PalletCorporateActionsCaId,PalletCorporateActionsBallotBallotTimeRange,PalletCorporateActionsBallotBallotMeta,bool)',
      VoteCast: '(PolymeshPrimitivesIdentityId,PalletCorporateActionsCaId,Vec<PalletCorporateActionsBallotBallotVote>)',
      RangeChanged: '(PolymeshPrimitivesIdentityId,PalletCorporateActionsCaId,PalletCorporateActionsBallotBallotTimeRange)',
      MetaChanged: '(PolymeshPrimitivesIdentityId,PalletCorporateActionsCaId,PalletCorporateActionsBallotBallotMeta)',
      RCVChanged: '(PolymeshPrimitivesIdentityId,PalletCorporateActionsCaId,bool)',
      Removed: '(PolymeshPrimitivesEventOnly,PalletCorporateActionsCaId)'
    }
  },
  /**
   * Lookup194: pallet_corporate_actions::ballot::BallotTimeRange
   **/
  PalletCorporateActionsBallotBallotTimeRange: {
    start: 'u64',
    end: 'u64'
  },
  /**
   * Lookup195: pallet_corporate_actions::ballot::BallotMeta
   **/
  PalletCorporateActionsBallotBallotMeta: {
    title: 'Bytes',
    motions: 'Vec<PalletCorporateActionsBallotMotion>'
  },
  /**
   * Lookup198: pallet_corporate_actions::ballot::Motion
   **/
  PalletCorporateActionsBallotMotion: {
    title: 'Bytes',
    infoLink: 'Bytes',
    choices: 'Vec<Bytes>'
  },
  /**
   * Lookup204: pallet_corporate_actions::ballot::BallotVote
   **/
  PalletCorporateActionsBallotBallotVote: {
    power: 'u128',
    fallback: 'Option<u16>'
  },
  /**
   * Lookup207: pallet_pips::RawEvent<sp_core::crypto::AccountId32, BlockNumber>
   **/
  PalletPipsRawEvent: {
    _enum: {
      HistoricalPipsPruned: '(PolymeshPrimitivesIdentityId,bool,bool)',
      ProposalCreated: '(PolymeshPrimitivesIdentityId,PalletPipsProposer,u32,u128,Option<Bytes>,Option<Bytes>,PolymeshCommonUtilitiesMaybeBlock,PalletPipsProposalData)',
      ProposalStateUpdated: '(PolymeshPrimitivesIdentityId,u32,PalletPipsProposalState)',
      Voted: '(PolymeshPrimitivesIdentityId,AccountId32,u32,bool,u128)',
      PipClosed: '(PolymeshPrimitivesIdentityId,u32,bool)',
      ExecutionScheduled: '(PolymeshPrimitivesIdentityId,u32,u32)',
      DefaultEnactmentPeriodChanged: '(PolymeshPrimitivesIdentityId,u32,u32)',
      MinimumProposalDepositChanged: '(PolymeshPrimitivesIdentityId,u128,u128)',
      PendingPipExpiryChanged: '(PolymeshPrimitivesIdentityId,PolymeshCommonUtilitiesMaybeBlock,PolymeshCommonUtilitiesMaybeBlock)',
      MaxPipSkipCountChanged: '(PolymeshPrimitivesIdentityId,u8,u8)',
      ActivePipLimitChanged: '(PolymeshPrimitivesIdentityId,u32,u32)',
      ProposalRefund: '(PolymeshPrimitivesIdentityId,u32,u128)',
      SnapshotCleared: '(PolymeshPrimitivesIdentityId,u32)',
      SnapshotTaken: '(PolymeshPrimitivesIdentityId,u32,Vec<PalletPipsSnapshottedPip>)',
      PipSkipped: '(PolymeshPrimitivesIdentityId,u32,u8)',
      SnapshotResultsEnacted: '(PolymeshPrimitivesIdentityId,Option<u32>,Vec<(u32,u8)>,Vec<u32>,Vec<u32>)',
      ExecutionSchedulingFailed: '(PolymeshPrimitivesIdentityId,u32,u32)',
      ExpiryScheduled: '(PolymeshPrimitivesIdentityId,u32,u32)',
      ExpirySchedulingFailed: '(PolymeshPrimitivesIdentityId,u32,u32)',
      ExecutionCancellingFailed: 'u32'
    }
  },
  /**
   * Lookup208: pallet_pips::Proposer<sp_core::crypto::AccountId32>
   **/
  PalletPipsProposer: {
    _enum: {
      Community: 'AccountId32',
      Committee: 'PalletPipsCommittee'
    }
  },
  /**
   * Lookup209: pallet_pips::Committee
   **/
  PalletPipsCommittee: {
    _enum: ['Technical', 'Upgrade']
  },
  /**
   * Lookup213: pallet_pips::ProposalData
   **/
  PalletPipsProposalData: {
    _enum: {
      Hash: 'H256',
      Proposal: 'Bytes'
    }
  },
  /**
   * Lookup214: pallet_pips::ProposalState
   **/
  PalletPipsProposalState: {
    _enum: ['Pending', 'Rejected', 'Scheduled', 'Failed', 'Executed', 'Expired']
  },
  /**
   * Lookup217: pallet_pips::SnapshottedPip
   **/
  PalletPipsSnapshottedPip: {
    id: 'u32',
    weight: '(bool,u128)'
  },
  /**
   * Lookup223: polymesh_common_utilities::traits::portfolio::Event
   **/
  PolymeshCommonUtilitiesPortfolioEvent: {
    _enum: {
      PortfolioCreated: '(PolymeshPrimitivesIdentityId,u64,Bytes)',
      PortfolioDeleted: '(PolymeshPrimitivesIdentityId,u64)',
      MovedBetweenPortfolios: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesIdentityIdPortfolioId,PolymeshPrimitivesIdentityIdPortfolioId,PolymeshPrimitivesTicker,u128,Option<PolymeshCommonUtilitiesBalancesMemo>)',
      PortfolioRenamed: '(PolymeshPrimitivesIdentityId,u64,Bytes)',
      UserPortfolios: '(PolymeshPrimitivesIdentityId,Vec<(u64,Bytes)>)',
      PortfolioCustodianChanged: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesIdentityIdPortfolioId,PolymeshPrimitivesIdentityId)'
    }
  },
  /**
   * Lookup227: pallet_protocol_fee::RawEvent<sp_core::crypto::AccountId32>
   **/
  PalletProtocolFeeRawEvent: {
    _enum: {
      FeeSet: '(PolymeshPrimitivesIdentityId,u128)',
      CoefficientSet: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesPosRatio)',
      FeeCharged: '(AccountId32,u128)'
    }
  },
  /**
   * Lookup228: polymesh_primitives::PosRatio
   **/
  PolymeshPrimitivesPosRatio: '(u32,u32)',
  /**
   * Lookup229: pallet_scheduler::pallet::Event<T>
   **/
  PalletSchedulerEvent: {
    _enum: {
      Scheduled: {
        when: 'u32',
        index: 'u32',
      },
      Canceled: {
        when: 'u32',
        index: 'u32',
      },
      Dispatched: {
        task: '(u32,u32)',
        id: 'Option<Bytes>',
        result: 'Result<Null, SpRuntimeDispatchError>',
      },
      CallLookupFailed: {
        task: '(u32,u32)',
        id: 'Option<Bytes>',
        error: 'FrameSupportScheduleLookupError'
      }
    }
  },
  /**
   * Lookup231: frame_support::traits::schedule::LookupError
   **/
  FrameSupportScheduleLookupError: {
    _enum: ['Unknown', 'BadFormat']
  },
  /**
   * Lookup232: pallet_settlement::RawEvent<Moment, BlockNumber, sp_core::crypto::AccountId32>
   **/
  PalletSettlementRawEvent: {
    _enum: {
      VenueCreated: '(PolymeshPrimitivesIdentityId,u64,Bytes,PalletSettlementVenueType)',
      VenueDetailsUpdated: '(PolymeshPrimitivesIdentityId,u64,Bytes)',
      VenueTypeUpdated: '(PolymeshPrimitivesIdentityId,u64,PalletSettlementVenueType)',
      InstructionCreated: '(PolymeshPrimitivesIdentityId,u64,u64,PalletSettlementSettlementType,Option<u64>,Option<u64>,Vec<PalletSettlementLeg>)',
      InstructionAffirmed: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesIdentityIdPortfolioId,u64)',
      AffirmationWithdrawn: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesIdentityIdPortfolioId,u64)',
      InstructionRejected: '(PolymeshPrimitivesIdentityId,u64)',
      ReceiptClaimed: '(PolymeshPrimitivesIdentityId,u64,u64,u64,AccountId32,Bytes)',
      ReceiptValidityChanged: '(PolymeshPrimitivesIdentityId,AccountId32,u64,bool)',
      ReceiptUnclaimed: '(PolymeshPrimitivesIdentityId,u64,u64,u64,AccountId32)',
      VenueFiltering: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesTicker,bool)',
      VenuesAllowed: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesTicker,Vec<u64>)',
      VenuesBlocked: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesTicker,Vec<u64>)',
      LegFailedExecution: '(PolymeshPrimitivesIdentityId,u64,u64)',
      InstructionFailed: '(PolymeshPrimitivesIdentityId,u64)',
      InstructionExecuted: '(PolymeshPrimitivesIdentityId,u64)',
      VenueUnauthorized: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesTicker,u64)',
      SchedulingFailed: 'SpRuntimeDispatchError',
      InstructionRescheduled: '(PolymeshPrimitivesIdentityId,u64)'
    }
  },
  /**
   * Lookup235: pallet_settlement::VenueType
   **/
  PalletSettlementVenueType: {
    _enum: ['Other', 'Distribution', 'Sto', 'Exchange']
  },
  /**
   * Lookup237: pallet_settlement::SettlementType<BlockNumber>
   **/
  PalletSettlementSettlementType: {
    _enum: {
      SettleOnAffirmation: 'Null',
      SettleOnBlock: 'u32'
    }
  },
  /**
   * Lookup239: pallet_settlement::Leg
   **/
  PalletSettlementLeg: {
    from: 'PolymeshPrimitivesIdentityIdPortfolioId',
    to: 'PolymeshPrimitivesIdentityIdPortfolioId',
    asset: 'PolymeshPrimitivesTicker',
    amount: 'u128'
  },
  /**
   * Lookup243: polymesh_common_utilities::traits::statistics::Event
   **/
  PolymeshCommonUtilitiesStatisticsEvent: {
    _enum: {
      StatTypesAdded: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesStatisticsAssetScope,Vec<PolymeshPrimitivesStatisticsStatType>)',
      StatTypesRemoved: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesStatisticsAssetScope,Vec<PolymeshPrimitivesStatisticsStatType>)',
      AssetStatsUpdated: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesStatisticsAssetScope,PolymeshPrimitivesStatisticsStatType,Vec<PolymeshPrimitivesStatisticsStatUpdate>)',
      SetAssetTransferCompliance: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesStatisticsAssetScope,Vec<PolymeshPrimitivesTransferComplianceTransferCondition>)',
      TransferConditionExemptionsAdded: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesTransferComplianceTransferConditionExemptKey,Vec<PolymeshPrimitivesIdentityId>)',
      TransferConditionExemptionsRemoved: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesTransferComplianceTransferConditionExemptKey,Vec<PolymeshPrimitivesIdentityId>)'
    }
  },
  /**
   * Lookup244: polymesh_primitives::statistics::AssetScope
   **/
  PolymeshPrimitivesStatisticsAssetScope: {
    _enum: {
      Ticker: 'PolymeshPrimitivesTicker'
    }
  },
  /**
   * Lookup246: polymesh_primitives::statistics::StatType
   **/
  PolymeshPrimitivesStatisticsStatType: {
    op: 'PolymeshPrimitivesStatisticsStatOpType',
    claimIssuer: 'Option<(PolymeshPrimitivesIdentityClaimClaimType,PolymeshPrimitivesIdentityId)>'
  },
  /**
   * Lookup247: polymesh_primitives::statistics::StatOpType
   **/
  PolymeshPrimitivesStatisticsStatOpType: {
    _enum: ['Count', 'Balance']
  },
  /**
   * Lookup251: polymesh_primitives::statistics::StatUpdate
   **/
  PolymeshPrimitivesStatisticsStatUpdate: {
    key2: 'PolymeshPrimitivesStatisticsStat2ndKey',
    value: 'Option<u128>'
  },
  /**
   * Lookup252: polymesh_primitives::statistics::Stat2ndKey
   **/
  PolymeshPrimitivesStatisticsStat2ndKey: {
    _enum: {
      NoClaimStat: 'Null',
      Claim: 'PolymeshPrimitivesStatisticsStatClaim'
    }
  },
  /**
   * Lookup253: polymesh_primitives::statistics::StatClaim
   **/
  PolymeshPrimitivesStatisticsStatClaim: {
    _enum: {
      Accredited: 'bool',
      Affiliate: 'bool',
      Jurisdiction: 'Option<PolymeshPrimitivesJurisdictionCountryCode>'
    }
  },
  /**
   * Lookup257: polymesh_primitives::transfer_compliance::TransferCondition
   **/
  PolymeshPrimitivesTransferComplianceTransferCondition: {
    _enum: {
      MaxInvestorCount: 'u64',
      MaxInvestorOwnership: 'Permill',
      ClaimCount: '(PolymeshPrimitivesStatisticsStatClaim,PolymeshPrimitivesIdentityId,u64,Option<u64>)',
      ClaimOwnership: '(PolymeshPrimitivesStatisticsStatClaim,PolymeshPrimitivesIdentityId,Permill,Permill)'
    }
  },
  /**
   * Lookup259: polymesh_primitives::transfer_compliance::TransferConditionExemptKey
   **/
  PolymeshPrimitivesTransferComplianceTransferConditionExemptKey: {
    asset: 'PolymeshPrimitivesStatisticsAssetScope',
    op: 'PolymeshPrimitivesStatisticsStatOpType',
    claimType: 'Option<PolymeshPrimitivesIdentityClaimClaimType>'
  },
  /**
   * Lookup261: pallet_sto::RawEvent<Moment>
   **/
  PalletStoRawEvent: {
    _enum: {
      FundraiserCreated: '(PolymeshPrimitivesIdentityId,u64,Bytes,PalletStoFundraiser)',
      Invested: '(PolymeshPrimitivesIdentityId,u64,PolymeshPrimitivesTicker,PolymeshPrimitivesTicker,u128,u128)',
      FundraiserFrozen: '(PolymeshPrimitivesIdentityId,u64)',
      FundraiserUnfrozen: '(PolymeshPrimitivesIdentityId,u64)',
      FundraiserWindowModified: '(PolymeshPrimitivesEventOnly,u64,u64,Option<u64>,u64,Option<u64>)',
      FundraiserClosed: '(PolymeshPrimitivesIdentityId,u64)'
    }
  },
  /**
   * Lookup264: pallet_sto::Fundraiser<Moment>
   **/
  PalletStoFundraiser: {
    creator: 'PolymeshPrimitivesIdentityId',
    offeringPortfolio: 'PolymeshPrimitivesIdentityIdPortfolioId',
    offeringAsset: 'PolymeshPrimitivesTicker',
    raisingPortfolio: 'PolymeshPrimitivesIdentityIdPortfolioId',
    raisingAsset: 'PolymeshPrimitivesTicker',
    tiers: 'Vec<PalletStoFundraiserTier>',
    venueId: 'u64',
    start: 'u64',
    end: 'Option<u64>',
    status: 'PalletStoFundraiserStatus',
    minimumInvestment: 'u128'
  },
  /**
   * Lookup266: pallet_sto::FundraiserTier
   **/
  PalletStoFundraiserTier: {
    total: 'u128',
    price: 'u128',
    remaining: 'u128'
  },
  /**
   * Lookup267: pallet_sto::FundraiserStatus
   **/
  PalletStoFundraiserStatus: {
    _enum: ['Live', 'Frozen', 'Closed', 'ClosedEarly']
  },
  /**
   * Lookup268: pallet_treasury::RawEvent<Balance, sp_core::crypto::AccountId32>
   **/
  PalletTreasuryRawEvent: {
    _enum: {
      TreasuryDisbursement: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesIdentityId,AccountId32,u128)',
      TreasuryDisbursementFailed: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesIdentityId,AccountId32,u128)',
      TreasuryReimbursement: '(PolymeshPrimitivesIdentityId,u128)'
    }
  },
  /**
   * Lookup269: pallet_utility::Event
   **/
  PalletUtilityEvent: {
    _enum: {
      BatchInterrupted: '(Vec<u32>,(u32,SpRuntimeDispatchError))',
      BatchOptimisticFailed: '(Vec<u32>,Vec<(u32,SpRuntimeDispatchError)>)',
      BatchCompleted: 'Vec<u32>'
    }
  },
  /**
   * Lookup273: polymesh_common_utilities::traits::base::Event
   **/
  PolymeshCommonUtilitiesBaseEvent: {
    _enum: {
      UnexpectedError: 'Option<SpRuntimeDispatchError>'
    }
  },
  /**
   * Lookup275: polymesh_common_utilities::traits::external_agents::Event
   **/
  PolymeshCommonUtilitiesExternalAgentsEvent: {
    _enum: {
      GroupCreated: '(PolymeshPrimitivesEventOnly,PolymeshPrimitivesTicker,u32,PolymeshPrimitivesSubsetSubsetRestrictionPalletPermissions)',
      GroupPermissionsUpdated: '(PolymeshPrimitivesEventOnly,PolymeshPrimitivesTicker,u32,PolymeshPrimitivesSubsetSubsetRestrictionPalletPermissions)',
      AgentAdded: '(PolymeshPrimitivesEventOnly,PolymeshPrimitivesTicker,PolymeshPrimitivesAgentAgentGroup)',
      AgentRemoved: '(PolymeshPrimitivesEventOnly,PolymeshPrimitivesTicker,PolymeshPrimitivesIdentityId)',
      GroupChanged: '(PolymeshPrimitivesEventOnly,PolymeshPrimitivesTicker,PolymeshPrimitivesIdentityId,PolymeshPrimitivesAgentAgentGroup)'
    }
  },
  /**
   * Lookup276: polymesh_common_utilities::traits::relayer::RawEvent<sp_core::crypto::AccountId32>
   **/
  PolymeshCommonUtilitiesRelayerRawEvent: {
    _enum: {
      AuthorizedPayingKey: '(PolymeshPrimitivesEventOnly,AccountId32,AccountId32,u128,u64)',
      AcceptedPayingKey: '(PolymeshPrimitivesEventOnly,AccountId32,AccountId32)',
      RemovedPayingKey: '(PolymeshPrimitivesEventOnly,AccountId32,AccountId32)',
      UpdatedPolyxLimit: '(PolymeshPrimitivesEventOnly,AccountId32,AccountId32,u128,u128)'
    }
  },
  /**
   * Lookup277: pallet_rewards::RawEvent<sp_core::crypto::AccountId32>
   **/
  PalletRewardsRawEvent: {
    _enum: {
      ItnRewardClaimed: '(AccountId32,u128)'
    }
  },
  /**
   * Lookup278: pallet_contracts::pallet::Event<T>
   **/
  PalletContractsEvent: {
    _enum: {
      Instantiated: {
        deployer: 'AccountId32',
        contract: 'AccountId32',
      },
      Terminated: {
        contract: 'AccountId32',
        beneficiary: 'AccountId32',
      },
      CodeStored: {
        codeHash: 'H256',
      },
      ContractEmitted: {
        contract: 'AccountId32',
        data: 'Bytes',
      },
      CodeRemoved: {
        codeHash: 'H256',
      },
      ContractCodeUpdated: {
        contract: 'AccountId32',
        newCodeHash: 'H256',
        oldCodeHash: 'H256'
      }
    }
  },
  /**
   * Lookup279: polymesh_contracts::Event
   **/
  PolymeshContractsEvent: 'Null',
  /**
   * Lookup280: pallet_preimage::pallet::Event<T>
   **/
  PalletPreimageEvent: {
    _enum: {
      Noted: {
        _alias: {
          hash_: 'hash',
        },
        hash_: 'H256',
      },
      Requested: {
        _alias: {
          hash_: 'hash',
        },
        hash_: 'H256',
      },
      Cleared: {
        _alias: {
          hash_: 'hash',
        },
        hash_: 'H256'
      }
    }
  },
  /**
   * Lookup281: pallet_test_utils::RawEvent<sp_core::crypto::AccountId32>
   **/
  PalletTestUtilsRawEvent: {
    _enum: {
      MockInvestorUIDCreated: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesCddIdInvestorUid)',
      DidStatus: '(PolymeshPrimitivesIdentityId,AccountId32)',
      CddStatus: '(Option<PolymeshPrimitivesIdentityId>,AccountId32,bool)'
    }
  },
  /**
   * Lookup282: polymesh_primitives::cdd_id::InvestorUid
   **/
  PolymeshPrimitivesCddIdInvestorUid: '[u8;16]',
  /**
   * Lookup283: frame_system::Phase
   **/
  FrameSystemPhase: {
    _enum: {
      ApplyExtrinsic: 'u32',
      Finalization: 'Null',
      Initialization: 'Null'
    }
  },
  /**
   * Lookup286: frame_system::LastRuntimeUpgradeInfo
   **/
  FrameSystemLastRuntimeUpgradeInfo: {
    specVersion: 'Compact<u32>',
    specName: 'Text'
  },
  /**
   * Lookup289: frame_system::pallet::Call<T>
   **/
  FrameSystemCall: {
    _enum: {
      fill_block: {
        ratio: 'Perbill',
      },
      remark: {
        remark: 'Bytes',
      },
      set_heap_pages: {
        pages: 'u64',
      },
      set_code: {
        code: 'Bytes',
      },
      set_code_without_checks: {
        code: 'Bytes',
      },
      set_storage: {
        items: 'Vec<(Bytes,Bytes)>',
      },
      kill_storage: {
        _alias: {
          keys_: 'keys',
        },
        keys_: 'Vec<Bytes>',
      },
      kill_prefix: {
        prefix: 'Bytes',
        subkeys: 'u32',
      },
      remark_with_event: {
        remark: 'Bytes'
      }
    }
  },
  /**
   * Lookup293: frame_system::limits::BlockWeights
   **/
  FrameSystemLimitsBlockWeights: {
    baseBlock: 'u64',
    maxBlock: 'u64',
    perClass: 'FrameSupportWeightsPerDispatchClassWeightsPerClass'
  },
  /**
   * Lookup294: frame_support::weights::PerDispatchClass<frame_system::limits::WeightsPerClass>
   **/
  FrameSupportWeightsPerDispatchClassWeightsPerClass: {
    normal: 'FrameSystemLimitsWeightsPerClass',
    operational: 'FrameSystemLimitsWeightsPerClass',
    mandatory: 'FrameSystemLimitsWeightsPerClass'
  },
  /**
   * Lookup295: frame_system::limits::WeightsPerClass
   **/
  FrameSystemLimitsWeightsPerClass: {
    baseExtrinsic: 'u64',
    maxExtrinsic: 'Option<u64>',
    maxTotal: 'Option<u64>',
    reserved: 'Option<u64>'
  },
  /**
   * Lookup296: frame_system::limits::BlockLength
   **/
  FrameSystemLimitsBlockLength: {
    max: 'FrameSupportWeightsPerDispatchClassU32'
  },
  /**
   * Lookup297: frame_support::weights::PerDispatchClass<T>
   **/
  FrameSupportWeightsPerDispatchClassU32: {
    normal: 'u32',
    operational: 'u32',
    mandatory: 'u32'
  },
  /**
   * Lookup298: frame_support::weights::RuntimeDbWeight
   **/
  FrameSupportWeightsRuntimeDbWeight: {
    read: 'u64',
    write: 'u64'
  },
  /**
   * Lookup299: sp_version::RuntimeVersion
   **/
  SpVersionRuntimeVersion: {
    specName: 'Text',
    implName: 'Text',
    authoringVersion: 'u32',
    specVersion: 'u32',
    implVersion: 'u32',
    apis: 'Vec<([u8;8],u32)>',
    transactionVersion: 'u32',
    stateVersion: 'u8'
  },
  /**
   * Lookup304: frame_system::pallet::Error<T>
   **/
  FrameSystemError: {
    _enum: ['InvalidSpecName', 'SpecVersionNeedsToIncrease', 'FailedToExtractRuntimeVersion', 'NonDefaultComposite', 'NonZeroRefCount', 'CallFiltered']
  },
  /**
   * Lookup307: sp_consensus_babe::app::Public
   **/
  SpConsensusBabeAppPublic: 'SpCoreSr25519Public',
  /**
   * Lookup310: sp_consensus_babe::digests::NextConfigDescriptor
   **/
  SpConsensusBabeDigestsNextConfigDescriptor: {
    _enum: {
      __Unused0: 'Null',
      V1: {
        c: '(u64,u64)',
        allowedSlots: 'SpConsensusBabeAllowedSlots'
      }
    }
  },
  /**
   * Lookup312: sp_consensus_babe::AllowedSlots
   **/
  SpConsensusBabeAllowedSlots: {
    _enum: ['PrimarySlots', 'PrimaryAndSecondaryPlainSlots', 'PrimaryAndSecondaryVRFSlots']
  },
  /**
   * Lookup316: sp_consensus_babe::BabeEpochConfiguration
   **/
  SpConsensusBabeBabeEpochConfiguration: {
    c: '(u64,u64)',
    allowedSlots: 'SpConsensusBabeAllowedSlots'
  },
  /**
   * Lookup317: pallet_babe::pallet::Call<T>
   **/
  PalletBabeCall: {
    _enum: {
      report_equivocation: {
        equivocationProof: 'SpConsensusSlotsEquivocationProof',
        keyOwnerProof: 'SpSessionMembershipProof',
      },
      report_equivocation_unsigned: {
        equivocationProof: 'SpConsensusSlotsEquivocationProof',
        keyOwnerProof: 'SpSessionMembershipProof',
      },
      plan_config_change: {
        config: 'SpConsensusBabeDigestsNextConfigDescriptor'
      }
    }
  },
  /**
   * Lookup318: sp_consensus_slots::EquivocationProof<sp_runtime::generic::header::Header<Number, sp_runtime::traits::BlakeTwo256>, sp_consensus_babe::app::Public>
   **/
  SpConsensusSlotsEquivocationProof: {
    offender: 'SpConsensusBabeAppPublic',
    slot: 'u64',
    firstHeader: 'SpRuntimeHeader',
    secondHeader: 'SpRuntimeHeader'
  },
  /**
   * Lookup319: sp_runtime::generic::header::Header<Number, sp_runtime::traits::BlakeTwo256>
   **/
  SpRuntimeHeader: {
    parentHash: 'H256',
    number: 'Compact<u32>',
    stateRoot: 'H256',
    extrinsicsRoot: 'H256',
    digest: 'SpRuntimeDigest'
  },
  /**
   * Lookup320: sp_runtime::traits::BlakeTwo256
   **/
  SpRuntimeBlakeTwo256: 'Null',
  /**
   * Lookup321: sp_session::MembershipProof
   **/
  SpSessionMembershipProof: {
    session: 'u32',
    trieNodes: 'Vec<Bytes>',
    validatorCount: 'u32'
  },
  /**
   * Lookup322: pallet_babe::pallet::Error<T>
   **/
  PalletBabeError: {
    _enum: ['InvalidEquivocationProof', 'InvalidKeyOwnershipProof', 'DuplicateOffenceReport']
  },
  /**
   * Lookup323: pallet_timestamp::pallet::Call<T>
   **/
  PalletTimestampCall: {
    _enum: {
      set: {
        now: 'Compact<u64>'
      }
    }
  },
  /**
   * Lookup326: pallet_indices::pallet::Call<T>
   **/
  PalletIndicesCall: {
    _enum: {
      claim: {
        index: 'u32',
      },
      transfer: {
        _alias: {
          new_: 'new',
        },
        new_: 'AccountId32',
        index: 'u32',
      },
      free: {
        index: 'u32',
      },
      force_transfer: {
        _alias: {
          new_: 'new',
        },
        new_: 'AccountId32',
        index: 'u32',
        freeze: 'bool',
      },
      freeze: {
        index: 'u32'
      }
    }
  },
  /**
   * Lookup327: pallet_indices::pallet::Error<T>
   **/
  PalletIndicesError: {
    _enum: ['NotAssigned', 'NotOwner', 'InUse', 'NotTransfer', 'Permanent']
  },
  /**
   * Lookup329: pallet_authorship::UncleEntryItem<BlockNumber, primitive_types::H256, sp_core::crypto::AccountId32>
   **/
  PalletAuthorshipUncleEntryItem: {
    _enum: {
      InclusionHeight: 'u32',
      Uncle: '(H256,Option<AccountId32>)'
    }
  },
  /**
   * Lookup330: pallet_authorship::pallet::Call<T>
   **/
  PalletAuthorshipCall: {
    _enum: {
      set_uncles: {
        newUncles: 'Vec<SpRuntimeHeader>'
      }
    }
  },
  /**
   * Lookup332: pallet_authorship::pallet::Error<T>
   **/
  PalletAuthorshipError: {
    _enum: ['InvalidUncleParent', 'UnclesAlreadySet', 'TooManyUncles', 'GenesisUncle', 'TooHighUncle', 'UncleAlreadyIncluded', 'OldUncle']
  },
  /**
   * Lookup334: pallet_balances::BalanceLock<Balance>
   **/
  PalletBalancesBalanceLock: {
    id: '[u8;8]',
    amount: 'u128',
    reasons: 'PolymeshCommonUtilitiesBalancesReasons'
  },
  /**
   * Lookup335: polymesh_common_utilities::traits::balances::Reasons
   **/
  PolymeshCommonUtilitiesBalancesReasons: {
    _enum: ['Fee', 'Misc', 'All']
  },
  /**
   * Lookup336: pallet_balances::Call<T>
   **/
  PalletBalancesCall: {
    _enum: {
      transfer: {
        dest: 'MultiAddress',
        value: 'Compact<u128>',
      },
      transfer_with_memo: {
        dest: 'MultiAddress',
        value: 'Compact<u128>',
        memo: 'Option<PolymeshCommonUtilitiesBalancesMemo>',
      },
      deposit_block_reward_reserve_balance: {
        value: 'Compact<u128>',
      },
      set_balance: {
        who: 'MultiAddress',
        newFree: 'Compact<u128>',
        newReserved: 'Compact<u128>',
      },
      force_transfer: {
        source: 'MultiAddress',
        dest: 'MultiAddress',
        value: 'Compact<u128>',
      },
      burn_account_balance: {
        amount: 'u128'
      }
    }
  },
  /**
   * Lookup338: pallet_balances::Error<T>
   **/
  PalletBalancesError: {
    _enum: ['LiquidityRestrictions', 'Overflow', 'InsufficientBalance', 'ExistentialDeposit', 'ReceiverCddMissing']
  },
  /**
   * Lookup340: pallet_transaction_payment::Releases
   **/
  PalletTransactionPaymentReleases: {
    _enum: ['V1Ancient', 'V2']
  },
  /**
   * Lookup342: frame_support::weights::WeightToFeeCoefficient<Balance>
   **/
  FrameSupportWeightsWeightToFeeCoefficient: {
    coeffInteger: 'u128',
    coeffFrac: 'Perbill',
    negative: 'bool',
    degree: 'u8'
  },
  /**
   * Lookup343: polymesh_primitives::identity::DidRecord<sp_core::crypto::AccountId32>
   **/
  PolymeshPrimitivesIdentityDidRecord: {
    primaryKey: 'Option<AccountId32>'
  },
  /**
   * Lookup345: pallet_identity::types::Claim1stKey
   **/
  PalletIdentityClaim1stKey: {
    target: 'PolymeshPrimitivesIdentityId',
    claimType: 'PolymeshPrimitivesIdentityClaimClaimType'
  },
  /**
   * Lookup346: pallet_identity::types::Claim2ndKey
   **/
  PalletIdentityClaim2ndKey: {
    issuer: 'PolymeshPrimitivesIdentityId',
    scope: 'Option<PolymeshPrimitivesIdentityClaimScope>'
  },
  /**
   * Lookup348: polymesh_primitives::secondary_key::KeyRecord<sp_core::crypto::AccountId32>
   **/
  PolymeshPrimitivesSecondaryKeyKeyRecord: {
    _enum: {
      PrimaryKey: 'PolymeshPrimitivesIdentityId',
      SecondaryKey: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesSecondaryKeyPermissions)',
      MultiSigSignerKey: 'AccountId32'
    }
  },
  /**
   * Lookup351: polymesh_primitives::authorization::Authorization<sp_core::crypto::AccountId32, Moment>
   **/
  PolymeshPrimitivesAuthorization: {
    authorizationData: 'PolymeshPrimitivesAuthorizationAuthorizationData',
    authorizedBy: 'PolymeshPrimitivesIdentityId',
    expiry: 'Option<u64>',
    authId: 'u64'
  },
  /**
   * Lookup354: pallet_identity::Call<T>
   **/
  PalletIdentityCall: {
    _enum: {
      cdd_register_did: {
        targetAccount: 'AccountId32',
        secondaryKeys: 'Vec<PolymeshPrimitivesSecondaryKey>',
      },
      invalidate_cdd_claims: {
        cdd: 'PolymeshPrimitivesIdentityId',
        disableFrom: 'u64',
        expiry: 'Option<u64>',
      },
      remove_secondary_keys_old: {
        keysToRemove: 'Vec<PolymeshPrimitivesSecondaryKeySignatory>',
      },
      accept_primary_key: {
        rotationAuthId: 'u64',
        optionalCddAuthId: 'Option<u64>',
      },
      change_cdd_requirement_for_mk_rotation: {
        authRequired: 'bool',
      },
      join_identity_as_key: {
        authId: 'u64',
      },
      leave_identity_as_key: 'Null',
      add_claim: {
        target: 'PolymeshPrimitivesIdentityId',
        claim: 'PolymeshPrimitivesIdentityClaimClaim',
        expiry: 'Option<u64>',
      },
      revoke_claim: {
        target: 'PolymeshPrimitivesIdentityId',
        claim: 'PolymeshPrimitivesIdentityClaimClaim',
      },
      set_permission_to_signer: {
        key: 'PolymeshPrimitivesSecondaryKeySignatory',
        perms: 'PolymeshPrimitivesSecondaryKeyPermissions',
      },
      placeholder_legacy_set_permission_to_signer: 'Null',
      freeze_secondary_keys: 'Null',
      unfreeze_secondary_keys: 'Null',
      add_authorization: {
        target: 'PolymeshPrimitivesSecondaryKeySignatory',
        data: 'PolymeshPrimitivesAuthorizationAuthorizationData',
        expiry: 'Option<u64>',
      },
      remove_authorization: {
        target: 'PolymeshPrimitivesSecondaryKeySignatory',
        authId: 'u64',
        authIssuerPays: 'bool',
      },
      add_secondary_keys_with_authorization_old: {
        additionalKeys: 'Vec<PolymeshCommonUtilitiesIdentitySecondaryKeyWithAuthV1>',
        expiresAt: 'u64',
      },
      add_investor_uniqueness_claim: {
        target: 'PolymeshPrimitivesIdentityId',
        claim: 'PolymeshPrimitivesIdentityClaimClaim',
        proof: 'PolymeshPrimitivesInvestorZkproofDataV1InvestorZKProofData',
        expiry: 'Option<u64>',
      },
      gc_add_cdd_claim: {
        target: 'PolymeshPrimitivesIdentityId',
      },
      gc_revoke_cdd_claim: {
        target: 'PolymeshPrimitivesIdentityId',
      },
      add_investor_uniqueness_claim_v2: {
        target: 'PolymeshPrimitivesIdentityId',
        scope: 'PolymeshPrimitivesIdentityClaimScope',
        claim: 'PolymeshPrimitivesIdentityClaimClaim',
        proof: 'ConfidentialIdentityClaimProofsScopeClaimProof',
        expiry: 'Option<u64>',
      },
      revoke_claim_by_index: {
        target: 'PolymeshPrimitivesIdentityId',
        claimType: 'PolymeshPrimitivesIdentityClaimClaimType',
        scope: 'Option<PolymeshPrimitivesIdentityClaimScope>',
      },
      rotate_primary_key_to_secondary: {
        authId: 'u64',
        optionalCddAuthId: 'Option<u64>',
      },
      add_secondary_keys_with_authorization: {
        additionalKeys: 'Vec<PolymeshCommonUtilitiesIdentitySecondaryKeyWithAuth>',
        expiresAt: 'u64',
      },
      set_secondary_key_permissions: {
        key: 'AccountId32',
        perms: 'PolymeshPrimitivesSecondaryKeyPermissions',
      },
      remove_secondary_keys: {
        keysToRemove: 'Vec<AccountId32>'
      }
    }
  },
  /**
   * Lookup356: polymesh_common_utilities::traits::identity::SecondaryKeyWithAuthV1<sp_core::crypto::AccountId32>
   **/
  PolymeshCommonUtilitiesIdentitySecondaryKeyWithAuthV1: {
    secondaryKey: 'PolymeshPrimitivesSecondaryKeyV1SecondaryKey',
    authSignature: 'H512'
  },
  /**
   * Lookup357: polymesh_primitives::secondary_key::v1::SecondaryKey<sp_core::crypto::AccountId32>
   **/
  PolymeshPrimitivesSecondaryKeyV1SecondaryKey: {
    signer: 'PolymeshPrimitivesSecondaryKeySignatory',
    permissions: 'PolymeshPrimitivesSecondaryKeyPermissions'
  },
  /**
   * Lookup359: polymesh_primitives::investor_zkproof_data::v1::InvestorZKProofData
   **/
  PolymeshPrimitivesInvestorZkproofDataV1InvestorZKProofData: 'SchnorrkelSignSignature',
  /**
   * Lookup360: schnorrkel::sign::Signature
   **/
  SchnorrkelSignSignature: {
    r: 'Curve25519DalekRistrettoCompressedRistretto',
    s: 'Curve25519DalekScalar'
  },
  /**
   * Lookup361: curve25519_dalek::ristretto::CompressedRistretto
   **/
  Curve25519DalekRistrettoCompressedRistretto: '[u8;32]',
  /**
   * Lookup362: curve25519_dalek::scalar::Scalar
   **/
  Curve25519DalekScalar: {
    bytes: '[u8;32]'
  },
  /**
   * Lookup363: confidential_identity::claim_proofs::ScopeClaimProof
   **/
  ConfidentialIdentityClaimProofsScopeClaimProof: {
    proofScopeIdWellformed: 'ConfidentialIdentitySignSignature',
    proofScopeIdCddIdMatch: 'ConfidentialIdentityClaimProofsZkProofData',
    scopeId: 'Curve25519DalekRistrettoRistrettoPoint'
  },
  /**
   * Lookup364: confidential_identity::sign::Signature
   **/
  ConfidentialIdentitySignSignature: {
    r: 'Curve25519DalekRistrettoCompressedRistretto',
    s: 'Curve25519DalekScalar'
  },
  /**
   * Lookup365: confidential_identity::claim_proofs::ZkProofData
   **/
  ConfidentialIdentityClaimProofsZkProofData: {
    challengeResponses: '[Lookup362;2]',
    subtractExpressionsRes: 'Curve25519DalekRistrettoRistrettoPoint',
    blindedScopeDidHash: 'Curve25519DalekRistrettoRistrettoPoint'
  },
  /**
   * Lookup367: curve25519_dalek::ristretto::RistrettoPoint
   **/
  Curve25519DalekRistrettoRistrettoPoint: 'Curve25519DalekEdwardsEdwardsPoint',
  /**
   * Lookup368: curve25519_dalek::edwards::EdwardsPoint
   **/
  Curve25519DalekEdwardsEdwardsPoint: {
    x: 'Curve25519DalekBackendSerialU64FieldFieldElement51',
    y: 'Curve25519DalekBackendSerialU64FieldFieldElement51',
    z: 'Curve25519DalekBackendSerialU64FieldFieldElement51',
    t: 'Curve25519DalekBackendSerialU64FieldFieldElement51'
  },
  /**
   * Lookup369: curve25519_dalek::backend::serial::u64::field::FieldElement51
   **/
  Curve25519DalekBackendSerialU64FieldFieldElement51: '[u64;5]',
  /**
   * Lookup372: polymesh_common_utilities::traits::identity::SecondaryKeyWithAuth<sp_core::crypto::AccountId32>
   **/
  PolymeshCommonUtilitiesIdentitySecondaryKeyWithAuth: {
    secondaryKey: 'PolymeshPrimitivesSecondaryKey',
    authSignature: 'H512'
  },
  /**
   * Lookup373: pallet_identity::Error<T>
   **/
  PalletIdentityError: {
    _enum: ['AlreadyLinked', 'MissingCurrentIdentity', 'Unauthorized', 'InvalidAccountKey', 'UnAuthorizedCddProvider', 'InvalidAuthorizationFromOwner', 'InvalidAuthorizationFromCddProvider', 'NotCddProviderAttestation', 'AuthorizationsNotForSameDids', 'DidMustAlreadyExist', 'CurrentIdentityCannotBeForwarded', 'AuthorizationExpired', 'TargetHasNoCdd', 'AuthorizationHasBeenRevoked', 'InvalidAuthorizationSignature', 'KeyNotAllowed', 'NotPrimaryKey', 'DidDoesNotExist', 'DidAlreadyExists', 'SecondaryKeysContainPrimaryKey', 'FailedToChargeFee', 'NotASigner', 'CannotDecodeSignerAccountId', 'MultiSigHasBalance', 'ConfidentialScopeClaimNotAllowed', 'InvalidScopeClaim', 'ClaimVariantNotAllowed', 'TargetHasNonZeroBalanceAtScopeId', 'CDDIdNotUniqueForIdentity', 'InvalidCDDId', 'ClaimAndProofVersionsDoNotMatch', 'AccountKeyIsBeingUsed', 'CustomScopeTooLong']
  },
  /**
   * Lookup375: polymesh_common_utilities::traits::group::InactiveMember<Moment>
   **/
  PolymeshCommonUtilitiesGroupInactiveMember: {
    id: 'PolymeshPrimitivesIdentityId',
    deactivatedAt: 'u64',
    expiry: 'Option<u64>'
  },
  /**
   * Lookup376: pallet_group::Call<T, I>
   **/
  PalletGroupCall: {
    _enum: {
      set_active_members_limit: {
        limit: 'u32',
      },
      disable_member: {
        who: 'PolymeshPrimitivesIdentityId',
        expiry: 'Option<u64>',
        at: 'Option<u64>',
      },
      add_member: {
        who: 'PolymeshPrimitivesIdentityId',
      },
      remove_member: {
        who: 'PolymeshPrimitivesIdentityId',
      },
      swap_member: {
        remove: 'PolymeshPrimitivesIdentityId',
        add: 'PolymeshPrimitivesIdentityId',
      },
      reset_members: {
        members: 'Vec<PolymeshPrimitivesIdentityId>',
      },
      abdicate_membership: 'Null'
    }
  },
  /**
   * Lookup377: pallet_group::Error<T, I>
   **/
  PalletGroupError: {
    _enum: ['OnlyPrimaryKeyAllowed', 'DuplicateMember', 'NoSuchMember', 'LastMemberCannotQuit', 'MissingCurrentIdentity', 'ActiveMembersLimitExceeded', 'ActiveMembersLimitOverflow']
  },
  /**
   * Lookup379: pallet_committee::Call<T, I>
   **/
  PalletCommitteeCall: {
    _enum: {
      set_vote_threshold: {
        n: 'u32',
        d: 'u32',
      },
      set_release_coordinator: {
        id: 'PolymeshPrimitivesIdentityId',
      },
      set_expires_after: {
        expiry: 'PolymeshCommonUtilitiesMaybeBlock',
      },
      vote_or_propose: {
        approve: 'bool',
        call: 'Call',
      },
      vote: {
        proposal: 'H256',
        index: 'u32',
        approve: 'bool'
      }
    }
  },
  /**
   * Lookup385: pallet_multisig::Call<T>
   **/
  PalletMultisigCall: {
    _enum: {
      create_multisig: {
        signers: 'Vec<PolymeshPrimitivesSecondaryKeySignatory>',
        sigsRequired: 'u64',
      },
      create_or_approve_proposal_as_identity: {
        multisig: 'AccountId32',
        proposal: 'Call',
        expiry: 'Option<u64>',
        autoClose: 'bool',
      },
      create_or_approve_proposal_as_key: {
        multisig: 'AccountId32',
        proposal: 'Call',
        expiry: 'Option<u64>',
        autoClose: 'bool',
      },
      create_proposal_as_identity: {
        multisig: 'AccountId32',
        proposal: 'Call',
        expiry: 'Option<u64>',
        autoClose: 'bool',
      },
      create_proposal_as_key: {
        multisig: 'AccountId32',
        proposal: 'Call',
        expiry: 'Option<u64>',
        autoClose: 'bool',
      },
      approve_as_identity: {
        multisig: 'AccountId32',
        proposalId: 'u64',
      },
      approve_as_key: {
        multisig: 'AccountId32',
        proposalId: 'u64',
      },
      reject_as_identity: {
        multisig: 'AccountId32',
        proposalId: 'u64',
      },
      reject_as_key: {
        multisig: 'AccountId32',
        proposalId: 'u64',
      },
      accept_multisig_signer_as_identity: {
        authId: 'u64',
      },
      accept_multisig_signer_as_key: {
        authId: 'u64',
      },
      add_multisig_signer: {
        signer: 'PolymeshPrimitivesSecondaryKeySignatory',
      },
      remove_multisig_signer: {
        signer: 'PolymeshPrimitivesSecondaryKeySignatory',
      },
      add_multisig_signers_via_creator: {
        multisig: 'AccountId32',
        signers: 'Vec<PolymeshPrimitivesSecondaryKeySignatory>',
      },
      remove_multisig_signers_via_creator: {
        multisig: 'AccountId32',
        signers: 'Vec<PolymeshPrimitivesSecondaryKeySignatory>',
      },
      change_sigs_required: {
        sigsRequired: 'u64',
      },
      make_multisig_secondary: {
        multisig: 'AccountId32',
      },
      make_multisig_primary: {
        multisig: 'AccountId32',
        optionalCddAuthId: 'Option<u64>',
      },
      execute_scheduled_proposal: {
        multisig: 'AccountId32',
        proposalId: 'u64',
        multisigDid: 'PolymeshPrimitivesIdentityId',
        proposalWeight: 'u64'
      }
    }
  },
  /**
   * Lookup386: pallet_bridge::Call<T>
   **/
  PalletBridgeCall: {
    _enum: {
      change_controller: {
        controller: 'AccountId32',
      },
      change_admin: {
        admin: 'AccountId32',
      },
      change_timelock: {
        timelock: 'u32',
      },
      freeze: 'Null',
      unfreeze: 'Null',
      change_bridge_limit: {
        amount: 'u128',
        duration: 'u32',
      },
      change_bridge_exempted: {
        exempted: 'Vec<(PolymeshPrimitivesIdentityId,bool)>',
      },
      force_handle_bridge_tx: {
        bridgeTx: 'PalletBridgeBridgeTx',
      },
      batch_propose_bridge_tx: {
        bridgeTxs: 'Vec<PalletBridgeBridgeTx>',
      },
      propose_bridge_tx: {
        bridgeTx: 'PalletBridgeBridgeTx',
      },
      handle_bridge_tx: {
        bridgeTx: 'PalletBridgeBridgeTx',
      },
      freeze_txs: {
        bridgeTxs: 'Vec<PalletBridgeBridgeTx>',
      },
      unfreeze_txs: {
        bridgeTxs: 'Vec<PalletBridgeBridgeTx>',
      },
      handle_scheduled_bridge_tx: {
        bridgeTx: 'PalletBridgeBridgeTx',
      },
      add_freeze_admin: {
        freezeAdmin: 'AccountId32',
      },
      remove_freeze_admin: {
        freezeAdmin: 'AccountId32',
      },
      remove_txs: {
        bridgeTxs: 'Vec<PalletBridgeBridgeTx>'
      }
    }
  },
  /**
   * Lookup390: pallet_staking::Call<T>
   **/
  PalletStakingCall: {
    _enum: {
      bond: {
        controller: 'MultiAddress',
        value: 'Compact<u128>',
        payee: 'PalletStakingRewardDestination',
      },
      bond_extra: {
        maxAdditional: 'Compact<u128>',
      },
      unbond: {
        value: 'Compact<u128>',
      },
      withdraw_unbonded: {
        numSlashingSpans: 'u32',
      },
      validate: {
        prefs: 'PalletStakingValidatorPrefs',
      },
      nominate: {
        targets: 'Vec<MultiAddress>',
      },
      chill: 'Null',
      set_payee: {
        payee: 'PalletStakingRewardDestination',
      },
      set_controller: {
        controller: 'MultiAddress',
      },
      set_validator_count: {
        _alias: {
          new_: 'new',
        },
        new_: 'Compact<u32>',
      },
      increase_validator_count: {
        additional: 'Compact<u32>',
      },
      scale_validator_count: {
        factor: 'Percent',
      },
      add_permissioned_validator: {
        identity: 'PolymeshPrimitivesIdentityId',
        intendedCount: 'Option<u32>',
      },
      remove_permissioned_validator: {
        identity: 'PolymeshPrimitivesIdentityId',
      },
      validate_cdd_expiry_nominators: {
        targets: 'Vec<AccountId32>',
      },
      set_commission_cap: {
        newCap: 'Perbill',
      },
      set_min_bond_threshold: {
        newValue: 'u128',
      },
      force_no_eras: 'Null',
      force_new_era: 'Null',
      set_invulnerables: {
        invulnerables: 'Vec<AccountId32>',
      },
      force_unstake: {
        stash: 'AccountId32',
        numSlashingSpans: 'u32',
      },
      force_new_era_always: 'Null',
      cancel_deferred_slash: {
        era: 'u32',
        slashIndices: 'Vec<u32>',
      },
      payout_stakers: {
        validatorStash: 'AccountId32',
        era: 'u32',
      },
      rebond: {
        value: 'Compact<u128>',
      },
      set_history_depth: {
        newHistoryDepth: 'Compact<u32>',
        eraItemsDeleted: 'Compact<u32>',
      },
      reap_stash: {
        stash: 'AccountId32',
        numSlashingSpans: 'u32',
      },
      submit_election_solution: {
        _alias: {
          size_: 'size',
        },
        winners: 'Vec<u16>',
        compact: 'PalletStakingCompactAssignments',
        score: 'SpNposElectionsElectionScore',
        era: 'u32',
        size_: 'PalletStakingElectionSize',
      },
      submit_election_solution_unsigned: {
        _alias: {
          size_: 'size',
        },
        winners: 'Vec<u16>',
        compact: 'PalletStakingCompactAssignments',
        score: 'SpNposElectionsElectionScore',
        era: 'u32',
        size_: 'PalletStakingElectionSize',
      },
      payout_stakers_by_system: {
        validatorStash: 'AccountId32',
        era: 'u32',
      },
      change_slashing_allowed_for: {
        slashingSwitch: 'PalletStakingSlashingSwitch',
      },
      update_permissioned_validator_intended_count: {
        identity: 'PolymeshPrimitivesIdentityId',
        newIntendedCount: 'u32'
      }
    }
  },
  /**
   * Lookup391: pallet_staking::RewardDestination<sp_core::crypto::AccountId32>
   **/
  PalletStakingRewardDestination: {
    _enum: {
      Staked: 'Null',
      Stash: 'Null',
      Controller: 'Null',
      Account: 'AccountId32'
    }
  },
  /**
   * Lookup392: pallet_staking::ValidatorPrefs
   **/
  PalletStakingValidatorPrefs: {
    commission: 'Compact<Perbill>',
    blocked: 'bool'
  },
  /**
   * Lookup398: pallet_staking::CompactAssignments
   **/
  PalletStakingCompactAssignments: {
    votes1: 'Vec<(Compact<u32>,Compact<u16>)>',
    votes2: 'Vec<(Compact<u32>,(Compact<u16>,Compact<PerU16>),Compact<u16>)>',
    votes3: 'Vec<(Compact<u32>,[(Compact<u16>,Compact<PerU16>);2],Compact<u16>)>',
    votes4: 'Vec<(Compact<u32>,[(Compact<u16>,Compact<PerU16>);3],Compact<u16>)>',
    votes5: 'Vec<(Compact<u32>,[(Compact<u16>,Compact<PerU16>);4],Compact<u16>)>',
    votes6: 'Vec<(Compact<u32>,[(Compact<u16>,Compact<PerU16>);5],Compact<u16>)>',
    votes7: 'Vec<(Compact<u32>,[(Compact<u16>,Compact<PerU16>);6],Compact<u16>)>',
    votes8: 'Vec<(Compact<u32>,[(Compact<u16>,Compact<PerU16>);7],Compact<u16>)>',
    votes9: 'Vec<(Compact<u32>,[(Compact<u16>,Compact<PerU16>);8],Compact<u16>)>',
    votes10: 'Vec<(Compact<u32>,[(Compact<u16>,Compact<PerU16>);9],Compact<u16>)>',
    votes11: 'Vec<(Compact<u32>,[(Compact<u16>,Compact<PerU16>);10],Compact<u16>)>',
    votes12: 'Vec<(Compact<u32>,[(Compact<u16>,Compact<PerU16>);11],Compact<u16>)>',
    votes13: 'Vec<(Compact<u32>,[(Compact<u16>,Compact<PerU16>);12],Compact<u16>)>',
    votes14: 'Vec<(Compact<u32>,[(Compact<u16>,Compact<PerU16>);13],Compact<u16>)>',
    votes15: 'Vec<(Compact<u32>,[(Compact<u16>,Compact<PerU16>);14],Compact<u16>)>',
    votes16: 'Vec<(Compact<u32>,[(Compact<u16>,Compact<PerU16>);15],Compact<u16>)>'
  },
  /**
   * Lookup449: sp_npos_elections::ElectionScore
   **/
  SpNposElectionsElectionScore: {
    minimalStake: 'u128',
    sumStake: 'u128',
    sumStakeSquared: 'u128'
  },
  /**
   * Lookup450: pallet_staking::ElectionSize
   **/
  PalletStakingElectionSize: {
    validators: 'Compact<u16>',
    nominators: 'Compact<u32>'
  },
  /**
   * Lookup451: pallet_session::pallet::Call<T>
   **/
  PalletSessionCall: {
    _enum: {
      set_keys: {
        _alias: {
          keys_: 'keys',
        },
        keys_: 'PolymeshRuntimeDevelopRuntimeSessionKeys',
        proof: 'Bytes',
      },
      purge_keys: 'Null'
    }
  },
  /**
   * Lookup452: polymesh_runtime_develop::runtime::SessionKeys
   **/
  PolymeshRuntimeDevelopRuntimeSessionKeys: {
    grandpa: 'SpFinalityGrandpaAppPublic',
    babe: 'SpConsensusBabeAppPublic',
    imOnline: 'PalletImOnlineSr25519AppSr25519Public',
    authorityDiscovery: 'SpAuthorityDiscoveryAppPublic'
  },
  /**
   * Lookup453: sp_authority_discovery::app::Public
   **/
  SpAuthorityDiscoveryAppPublic: 'SpCoreSr25519Public',
  /**
   * Lookup454: pallet_grandpa::pallet::Call<T>
   **/
  PalletGrandpaCall: {
    _enum: {
      report_equivocation: {
        equivocationProof: 'SpFinalityGrandpaEquivocationProof',
        keyOwnerProof: 'SpSessionMembershipProof',
      },
      report_equivocation_unsigned: {
        equivocationProof: 'SpFinalityGrandpaEquivocationProof',
        keyOwnerProof: 'SpSessionMembershipProof',
      },
      note_stalled: {
        delay: 'u32',
        bestFinalizedBlockNumber: 'u32'
      }
    }
  },
  /**
   * Lookup455: sp_finality_grandpa::EquivocationProof<primitive_types::H256, N>
   **/
  SpFinalityGrandpaEquivocationProof: {
    setId: 'u64',
    equivocation: 'SpFinalityGrandpaEquivocation'
  },
  /**
   * Lookup456: sp_finality_grandpa::Equivocation<primitive_types::H256, N>
   **/
  SpFinalityGrandpaEquivocation: {
    _enum: {
      Prevote: 'FinalityGrandpaEquivocationPrevote',
      Precommit: 'FinalityGrandpaEquivocationPrecommit'
    }
  },
  /**
   * Lookup457: finality_grandpa::Equivocation<sp_finality_grandpa::app::Public, finality_grandpa::Prevote<primitive_types::H256, N>, sp_finality_grandpa::app::Signature>
   **/
  FinalityGrandpaEquivocationPrevote: {
    roundNumber: 'u64',
    identity: 'SpFinalityGrandpaAppPublic',
    first: '(FinalityGrandpaPrevote,SpFinalityGrandpaAppSignature)',
    second: '(FinalityGrandpaPrevote,SpFinalityGrandpaAppSignature)'
  },
  /**
   * Lookup458: finality_grandpa::Prevote<primitive_types::H256, N>
   **/
  FinalityGrandpaPrevote: {
    targetHash: 'H256',
    targetNumber: 'u32'
  },
  /**
   * Lookup459: sp_finality_grandpa::app::Signature
   **/
  SpFinalityGrandpaAppSignature: 'SpCoreEd25519Signature',
  /**
   * Lookup460: sp_core::ed25519::Signature
   **/
  SpCoreEd25519Signature: '[u8;64]',
  /**
   * Lookup462: finality_grandpa::Equivocation<sp_finality_grandpa::app::Public, finality_grandpa::Precommit<primitive_types::H256, N>, sp_finality_grandpa::app::Signature>
   **/
  FinalityGrandpaEquivocationPrecommit: {
    roundNumber: 'u64',
    identity: 'SpFinalityGrandpaAppPublic',
    first: '(FinalityGrandpaPrecommit,SpFinalityGrandpaAppSignature)',
    second: '(FinalityGrandpaPrecommit,SpFinalityGrandpaAppSignature)'
  },
  /**
   * Lookup463: finality_grandpa::Precommit<primitive_types::H256, N>
   **/
  FinalityGrandpaPrecommit: {
    targetHash: 'H256',
    targetNumber: 'u32'
  },
  /**
   * Lookup465: pallet_im_online::pallet::Call<T>
   **/
  PalletImOnlineCall: {
    _enum: {
      heartbeat: {
        heartbeat: 'PalletImOnlineHeartbeat',
        signature: 'PalletImOnlineSr25519AppSr25519Signature'
      }
    }
  },
  /**
   * Lookup466: pallet_im_online::Heartbeat<BlockNumber>
   **/
  PalletImOnlineHeartbeat: {
    blockNumber: 'u32',
    networkState: 'SpCoreOffchainOpaqueNetworkState',
    sessionIndex: 'u32',
    authorityIndex: 'u32',
    validatorsLen: 'u32'
  },
  /**
   * Lookup467: sp_core::offchain::OpaqueNetworkState
   **/
  SpCoreOffchainOpaqueNetworkState: {
    peerId: 'Bytes',
    externalAddresses: 'Vec<Bytes>'
  },
  /**
   * Lookup471: pallet_im_online::sr25519::app_sr25519::Signature
   **/
  PalletImOnlineSr25519AppSr25519Signature: 'SpCoreSr25519Signature',
  /**
   * Lookup472: sp_core::sr25519::Signature
   **/
  SpCoreSr25519Signature: '[u8;64]',
  /**
   * Lookup473: pallet_sudo::Call<T>
   **/
  PalletSudoCall: {
    _enum: {
      sudo: {
        call: 'Call',
      },
      sudo_unchecked_weight: {
        call: 'Call',
        weight: 'u64',
      },
      set_key: {
        _alias: {
          new_: 'new',
        },
        new_: 'MultiAddress',
      },
      sudo_as: {
        who: 'MultiAddress',
        call: 'Call'
      }
    }
  },
  /**
   * Lookup474: pallet_asset::Call<T>
   **/
  PalletAssetCall: {
    _enum: {
      register_ticker: {
        ticker: 'PolymeshPrimitivesTicker',
      },
      accept_ticker_transfer: {
        authId: 'u64',
      },
      accept_asset_ownership_transfer: {
        authId: 'u64',
      },
      create_asset: {
        name: 'Bytes',
        ticker: 'PolymeshPrimitivesTicker',
        divisible: 'bool',
        assetType: 'PolymeshPrimitivesAssetAssetType',
        identifiers: 'Vec<PolymeshPrimitivesAssetIdentifier>',
        fundingRound: 'Option<Bytes>',
        disableIu: 'bool',
      },
      freeze: {
        ticker: 'PolymeshPrimitivesTicker',
      },
      unfreeze: {
        ticker: 'PolymeshPrimitivesTicker',
      },
      rename_asset: {
        ticker: 'PolymeshPrimitivesTicker',
        name: 'Bytes',
      },
      issue: {
        ticker: 'PolymeshPrimitivesTicker',
        amount: 'u128',
      },
      redeem: {
        ticker: 'PolymeshPrimitivesTicker',
        value: 'u128',
      },
      make_divisible: {
        ticker: 'PolymeshPrimitivesTicker',
      },
      add_documents: {
        docs: 'Vec<PolymeshPrimitivesDocument>',
        ticker: 'PolymeshPrimitivesTicker',
      },
      remove_documents: {
        ids: 'Vec<u32>',
        ticker: 'PolymeshPrimitivesTicker',
      },
      set_funding_round: {
        ticker: 'PolymeshPrimitivesTicker',
        name: 'Bytes',
      },
      update_identifiers: {
        ticker: 'PolymeshPrimitivesTicker',
        identifiers: 'Vec<PolymeshPrimitivesAssetIdentifier>',
      },
      claim_classic_ticker: {
        ticker: 'PolymeshPrimitivesTicker',
        ethereumSignature: 'PolymeshPrimitivesEthereumEcdsaSignature',
      },
      reserve_classic_ticker: {
        classicTickerImport: 'PalletAssetClassicTickerImport',
        contractDid: 'PolymeshPrimitivesIdentityId',
        config: 'PalletAssetTickerRegistrationConfig',
      },
      controller_transfer: {
        ticker: 'PolymeshPrimitivesTicker',
        value: 'u128',
        fromPortfolio: 'PolymeshPrimitivesIdentityIdPortfolioId',
      },
      register_custom_asset_type: {
        ty: 'Bytes',
      },
      create_asset_with_custom_type: {
        name: 'Bytes',
        ticker: 'PolymeshPrimitivesTicker',
        divisible: 'bool',
        customAssetType: 'Bytes',
        identifiers: 'Vec<PolymeshPrimitivesAssetIdentifier>',
        fundingRound: 'Option<Bytes>',
        disableIu: 'bool',
      },
      set_asset_metadata: {
        ticker: 'PolymeshPrimitivesTicker',
        key: 'PolymeshPrimitivesAssetMetadataAssetMetadataKey',
        value: 'Bytes',
        detail: 'Option<PolymeshPrimitivesAssetMetadataAssetMetadataValueDetail>',
      },
      set_asset_metadata_details: {
        ticker: 'PolymeshPrimitivesTicker',
        key: 'PolymeshPrimitivesAssetMetadataAssetMetadataKey',
        detail: 'PolymeshPrimitivesAssetMetadataAssetMetadataValueDetail',
      },
      register_and_set_local_asset_metadata: {
        ticker: 'PolymeshPrimitivesTicker',
        name: 'Bytes',
        spec: 'PolymeshPrimitivesAssetMetadataAssetMetadataSpec',
        value: 'Bytes',
        detail: 'Option<PolymeshPrimitivesAssetMetadataAssetMetadataValueDetail>',
      },
      register_asset_metadata_local_type: {
        ticker: 'PolymeshPrimitivesTicker',
        name: 'Bytes',
        spec: 'PolymeshPrimitivesAssetMetadataAssetMetadataSpec',
      },
      register_asset_metadata_global_type: {
        name: 'Bytes',
        spec: 'PolymeshPrimitivesAssetMetadataAssetMetadataSpec'
      }
    }
  },
  /**
   * Lookup477: polymesh_primitives::ethereum::EcdsaSignature
   **/
  PolymeshPrimitivesEthereumEcdsaSignature: '[u8;65]',
  /**
   * Lookup479: pallet_asset::ClassicTickerImport
   **/
  PalletAssetClassicTickerImport: {
    ethOwner: 'PolymeshPrimitivesEthereumEthereumAddress',
    ticker: 'PolymeshPrimitivesTicker',
    isContract: 'bool',
    isCreated: 'bool'
  },
  /**
   * Lookup480: pallet_asset::TickerRegistrationConfig<U>
   **/
  PalletAssetTickerRegistrationConfig: {
    maxTickerLength: 'u8',
    registrationLength: 'Option<u64>'
  },
  /**
   * Lookup481: polymesh_primitives::asset_metadata::AssetMetadataKey
   **/
  PolymeshPrimitivesAssetMetadataAssetMetadataKey: {
    _enum: {
      Global: 'u64',
      Local: 'u64'
    }
  },
  /**
   * Lookup482: pallet_corporate_actions::distribution::Call<T>
   **/
  PalletCorporateActionsDistributionCall: {
    _enum: {
      distribute: {
        caId: 'PalletCorporateActionsCaId',
        portfolio: 'Option<u64>',
        currency: 'PolymeshPrimitivesTicker',
        perShare: 'u128',
        amount: 'u128',
        paymentAt: 'u64',
        expiresAt: 'Option<u64>',
      },
      claim: {
        caId: 'PalletCorporateActionsCaId',
      },
      push_benefit: {
        caId: 'PalletCorporateActionsCaId',
        holder: 'PolymeshPrimitivesIdentityId',
      },
      reclaim: {
        caId: 'PalletCorporateActionsCaId',
      },
      remove_distribution: {
        caId: 'PalletCorporateActionsCaId'
      }
    }
  },
  /**
   * Lookup484: pallet_asset::checkpoint::Call<T>
   **/
  PalletAssetCheckpointCall: {
    _enum: {
      create_checkpoint: {
        ticker: 'PolymeshPrimitivesTicker',
      },
      set_schedules_max_complexity: {
        maxComplexity: 'u64',
      },
      create_schedule: {
        ticker: 'PolymeshPrimitivesTicker',
        schedule: 'PalletAssetCheckpointScheduleSpec',
      },
      remove_schedule: {
        ticker: 'PolymeshPrimitivesTicker',
        id: 'u64'
      }
    }
  },
  /**
   * Lookup485: pallet_asset::checkpoint::ScheduleSpec
   **/
  PalletAssetCheckpointScheduleSpec: {
    start: 'Option<u64>',
    period: 'PolymeshPrimitivesCalendarCalendarPeriod',
    remaining: 'u32'
  },
  /**
   * Lookup486: pallet_compliance_manager::Call<T>
   **/
  PalletComplianceManagerCall: {
    _enum: {
      add_compliance_requirement: {
        ticker: 'PolymeshPrimitivesTicker',
        senderConditions: 'Vec<PolymeshPrimitivesCondition>',
        receiverConditions: 'Vec<PolymeshPrimitivesCondition>',
      },
      remove_compliance_requirement: {
        ticker: 'PolymeshPrimitivesTicker',
        id: 'u32',
      },
      replace_asset_compliance: {
        ticker: 'PolymeshPrimitivesTicker',
        assetCompliance: 'Vec<PolymeshPrimitivesComplianceManagerComplianceRequirement>',
      },
      reset_asset_compliance: {
        ticker: 'PolymeshPrimitivesTicker',
      },
      pause_asset_compliance: {
        ticker: 'PolymeshPrimitivesTicker',
      },
      resume_asset_compliance: {
        ticker: 'PolymeshPrimitivesTicker',
      },
      add_default_trusted_claim_issuer: {
        ticker: 'PolymeshPrimitivesTicker',
        issuer: 'PolymeshPrimitivesConditionTrustedIssuer',
      },
      remove_default_trusted_claim_issuer: {
        ticker: 'PolymeshPrimitivesTicker',
        issuer: 'PolymeshPrimitivesIdentityId',
      },
      change_compliance_requirement: {
        ticker: 'PolymeshPrimitivesTicker',
        newReq: 'PolymeshPrimitivesComplianceManagerComplianceRequirement'
      }
    }
  },
  /**
   * Lookup487: pallet_corporate_actions::Call<T>
   **/
  PalletCorporateActionsCall: {
    _enum: {
      set_max_details_length: {
        length: 'u32',
      },
      set_default_targets: {
        ticker: 'PolymeshPrimitivesTicker',
        targets: 'PalletCorporateActionsTargetIdentities',
      },
      set_default_withholding_tax: {
        ticker: 'PolymeshPrimitivesTicker',
        tax: 'Permill',
      },
      set_did_withholding_tax: {
        ticker: 'PolymeshPrimitivesTicker',
        taxedDid: 'PolymeshPrimitivesIdentityId',
        tax: 'Option<Permill>',
      },
      initiate_corporate_action: {
        ticker: 'PolymeshPrimitivesTicker',
        kind: 'PalletCorporateActionsCaKind',
        declDate: 'u64',
        recordDate: 'Option<PalletCorporateActionsRecordDateSpec>',
        details: 'Bytes',
        targets: 'Option<PalletCorporateActionsTargetIdentities>',
        defaultWithholdingTax: 'Option<Permill>',
        withholdingTax: 'Option<Vec<(PolymeshPrimitivesIdentityId,Permill)>>',
      },
      link_ca_doc: {
        id: 'PalletCorporateActionsCaId',
        docs: 'Vec<u32>',
      },
      remove_ca: {
        caId: 'PalletCorporateActionsCaId',
      },
      change_record_date: {
        caId: 'PalletCorporateActionsCaId',
        recordDate: 'Option<PalletCorporateActionsRecordDateSpec>',
      },
      initiate_corporate_action_and_distribute: {
        caArgs: 'PalletCorporateActionsInitiateCorporateActionArgs',
        portfolio: 'Option<u64>',
        currency: 'PolymeshPrimitivesTicker',
        perShare: 'u128',
        amount: 'u128',
        paymentAt: 'u64',
        expiresAt: 'Option<u64>'
      }
    }
  },
  /**
   * Lookup489: pallet_corporate_actions::RecordDateSpec
   **/
  PalletCorporateActionsRecordDateSpec: {
    _enum: {
      Scheduled: 'u64',
      ExistingSchedule: 'u64',
      Existing: 'u64'
    }
  },
  /**
   * Lookup492: pallet_corporate_actions::InitiateCorporateActionArgs
   **/
  PalletCorporateActionsInitiateCorporateActionArgs: {
    ticker: 'PolymeshPrimitivesTicker',
    kind: 'PalletCorporateActionsCaKind',
    declDate: 'u64',
    recordDate: 'Option<PalletCorporateActionsRecordDateSpec>',
    details: 'Bytes',
    targets: 'Option<PalletCorporateActionsTargetIdentities>',
    defaultWithholdingTax: 'Option<Permill>',
    withholdingTax: 'Option<Vec<(PolymeshPrimitivesIdentityId,Permill)>>'
  },
  /**
   * Lookup493: pallet_corporate_actions::ballot::Call<T>
   **/
  PalletCorporateActionsBallotCall: {
    _enum: {
      attach_ballot: {
        caId: 'PalletCorporateActionsCaId',
        range: 'PalletCorporateActionsBallotBallotTimeRange',
        meta: 'PalletCorporateActionsBallotBallotMeta',
        rcv: 'bool',
      },
      vote: {
        caId: 'PalletCorporateActionsCaId',
        votes: 'Vec<PalletCorporateActionsBallotBallotVote>',
      },
      change_end: {
        caId: 'PalletCorporateActionsCaId',
        end: 'u64',
      },
      change_meta: {
        caId: 'PalletCorporateActionsCaId',
        meta: 'PalletCorporateActionsBallotBallotMeta',
      },
      change_rcv: {
        caId: 'PalletCorporateActionsCaId',
        rcv: 'bool',
      },
      remove_ballot: {
        caId: 'PalletCorporateActionsCaId'
      }
    }
  },
  /**
   * Lookup494: pallet_pips::Call<T>
   **/
  PalletPipsCall: {
    _enum: {
      set_prune_historical_pips: {
        prune: 'bool',
      },
      set_min_proposal_deposit: {
        deposit: 'u128',
      },
      set_default_enactment_period: {
        duration: 'u32',
      },
      set_pending_pip_expiry: {
        expiry: 'PolymeshCommonUtilitiesMaybeBlock',
      },
      set_max_pip_skip_count: {
        max: 'u8',
      },
      set_active_pip_limit: {
        limit: 'u32',
      },
      propose: {
        proposal: 'Call',
        deposit: 'u128',
        url: 'Option<Bytes>',
        description: 'Option<Bytes>',
      },
      vote: {
        id: 'u32',
        ayeOrNay: 'bool',
        deposit: 'u128',
      },
      approve_committee_proposal: {
        id: 'u32',
      },
      reject_proposal: {
        id: 'u32',
      },
      prune_proposal: {
        id: 'u32',
      },
      reschedule_execution: {
        id: 'u32',
        until: 'Option<u32>',
      },
      clear_snapshot: 'Null',
      snapshot: 'Null',
      enact_snapshot_results: {
        results: 'Vec<(u32,PalletPipsSnapshotResult)>',
      },
      execute_scheduled_pip: {
        id: 'u32',
      },
      expire_scheduled_pip: {
        did: 'PolymeshPrimitivesIdentityId',
        id: 'u32'
      }
    }
  },
  /**
   * Lookup497: pallet_pips::SnapshotResult
   **/
  PalletPipsSnapshotResult: {
    _enum: ['Approve', 'Reject', 'Skip']
  },
  /**
   * Lookup498: pallet_portfolio::Call<T>
   **/
  PalletPortfolioCall: {
    _enum: {
      create_portfolio: {
        name: 'Bytes',
      },
      delete_portfolio: {
        num: 'u64',
      },
      move_portfolio_funds: {
        from: 'PolymeshPrimitivesIdentityIdPortfolioId',
        to: 'PolymeshPrimitivesIdentityIdPortfolioId',
        items: 'Vec<PalletPortfolioMovePortfolioItem>',
      },
      rename_portfolio: {
        num: 'u64',
        toName: 'Bytes',
      },
      quit_portfolio_custody: {
        pid: 'PolymeshPrimitivesIdentityIdPortfolioId',
      },
      accept_portfolio_custody: {
        authId: 'u64'
      }
    }
  },
  /**
   * Lookup500: pallet_portfolio::MovePortfolioItem
   **/
  PalletPortfolioMovePortfolioItem: {
    ticker: 'PolymeshPrimitivesTicker',
    amount: 'u128',
    memo: 'Option<PolymeshCommonUtilitiesBalancesMemo>'
  },
  /**
   * Lookup501: pallet_protocol_fee::Call<T>
   **/
  PalletProtocolFeeCall: {
    _enum: {
      change_coefficient: {
        coefficient: 'PolymeshPrimitivesPosRatio',
      },
      change_base_fee: {
        op: 'PolymeshCommonUtilitiesProtocolFeeProtocolOp',
        baseFee: 'u128'
      }
    }
  },
  /**
   * Lookup502: polymesh_common_utilities::protocol_fee::ProtocolOp
   **/
  PolymeshCommonUtilitiesProtocolFeeProtocolOp: {
    _enum: ['AssetRegisterTicker', 'AssetIssue', 'AssetAddDocuments', 'AssetCreateAsset', 'CheckpointCreateSchedule', 'ComplianceManagerAddComplianceRequirement', 'IdentityCddRegisterDid', 'IdentityAddClaim', 'IdentityAddSecondaryKeysWithAuthorization', 'PipsPropose', 'ContractsPutCode', 'CorporateBallotAttachBallot', 'CapitalDistributionDistribute']
  },
  /**
   * Lookup503: pallet_scheduler::pallet::Call<T>
   **/
  PalletSchedulerCall: {
    _enum: {
      schedule: {
        when: 'u32',
        maybePeriodic: 'Option<(u32,u32)>',
        priority: 'u8',
        call: 'FrameSupportScheduleMaybeHashed',
      },
      cancel: {
        when: 'u32',
        index: 'u32',
      },
      schedule_named: {
        id: 'Bytes',
        when: 'u32',
        maybePeriodic: 'Option<(u32,u32)>',
        priority: 'u8',
        call: 'FrameSupportScheduleMaybeHashed',
      },
      cancel_named: {
        id: 'Bytes',
      },
      schedule_after: {
        after: 'u32',
        maybePeriodic: 'Option<(u32,u32)>',
        priority: 'u8',
        call: 'FrameSupportScheduleMaybeHashed',
      },
      schedule_named_after: {
        id: 'Bytes',
        after: 'u32',
        maybePeriodic: 'Option<(u32,u32)>',
        priority: 'u8',
        call: 'FrameSupportScheduleMaybeHashed'
      }
    }
  },
  /**
   * Lookup505: frame_support::traits::schedule::MaybeHashed<polymesh_runtime_develop::runtime::Call, primitive_types::H256>
   **/
  FrameSupportScheduleMaybeHashed: {
    _enum: {
      Value: 'Call',
      Hash: 'H256'
    }
  },
  /**
   * Lookup506: pallet_settlement::Call<T>
   **/
  PalletSettlementCall: {
    _enum: {
      create_venue: {
        details: 'Bytes',
        signers: 'Vec<AccountId32>',
        typ: 'PalletSettlementVenueType',
      },
      update_venue_details: {
        id: 'u64',
        details: 'Bytes',
      },
      update_venue_type: {
        id: 'u64',
        typ: 'PalletSettlementVenueType',
      },
      add_instruction: {
        venueId: 'u64',
        settlementType: 'PalletSettlementSettlementType',
        tradeDate: 'Option<u64>',
        valueDate: 'Option<u64>',
        legs: 'Vec<PalletSettlementLeg>',
      },
      add_and_affirm_instruction: {
        venueId: 'u64',
        settlementType: 'PalletSettlementSettlementType',
        tradeDate: 'Option<u64>',
        valueDate: 'Option<u64>',
        legs: 'Vec<PalletSettlementLeg>',
        portfolios: 'Vec<PolymeshPrimitivesIdentityIdPortfolioId>',
      },
      affirm_instruction: {
        id: 'u64',
        portfolios: 'Vec<PolymeshPrimitivesIdentityIdPortfolioId>',
        maxLegsCount: 'u32',
      },
      withdraw_affirmation: {
        id: 'u64',
        portfolios: 'Vec<PolymeshPrimitivesIdentityIdPortfolioId>',
        maxLegsCount: 'u32',
      },
      reject_instruction: {
        id: 'u64',
        portfolio: 'PolymeshPrimitivesIdentityIdPortfolioId',
        numOfLegs: 'u32',
      },
      affirm_with_receipts: {
        id: 'u64',
        receiptDetails: 'Vec<PalletSettlementReceiptDetails>',
        portfolios: 'Vec<PolymeshPrimitivesIdentityIdPortfolioId>',
        maxLegsCount: 'u32',
      },
      claim_receipt: {
        id: 'u64',
        receiptDetails: 'PalletSettlementReceiptDetails',
      },
      unclaim_receipt: {
        instructionId: 'u64',
        legId: 'u64',
      },
      set_venue_filtering: {
        ticker: 'PolymeshPrimitivesTicker',
        enabled: 'bool',
      },
      allow_venues: {
        ticker: 'PolymeshPrimitivesTicker',
        venues: 'Vec<u64>',
      },
      disallow_venues: {
        ticker: 'PolymeshPrimitivesTicker',
        venues: 'Vec<u64>',
      },
      change_receipt_validity: {
        receiptUid: 'u64',
        validity: 'bool',
      },
      execute_scheduled_instruction: {
        id: 'u64',
        legsCount: 'u32',
      },
      reschedule_instruction: {
        id: 'u64'
      }
    }
  },
  /**
   * Lookup508: pallet_settlement::ReceiptDetails<sp_core::crypto::AccountId32, sp_runtime::MultiSignature>
   **/
  PalletSettlementReceiptDetails: {
    receiptUid: 'u64',
    legId: 'u64',
    signer: 'AccountId32',
    signature: 'SpRuntimeMultiSignature',
    metadata: 'Bytes'
  },
  /**
   * Lookup509: sp_runtime::MultiSignature
   **/
  SpRuntimeMultiSignature: {
    _enum: {
      Ed25519: 'SpCoreEd25519Signature',
      Sr25519: 'SpCoreSr25519Signature',
      Ecdsa: 'SpCoreEcdsaSignature'
    }
  },
  /**
   * Lookup510: sp_core::ecdsa::Signature
   **/
  SpCoreEcdsaSignature: '[u8;65]',
  /**
   * Lookup511: pallet_statistics::Call<T>
   **/
  PalletStatisticsCall: {
    _enum: {
      set_active_asset_stats: {
        asset: 'PolymeshPrimitivesStatisticsAssetScope',
        statTypes: 'BTreeSetStatType',
      },
      batch_update_asset_stats: {
        asset: 'PolymeshPrimitivesStatisticsAssetScope',
        statType: 'PolymeshPrimitivesStatisticsStatType',
        values: 'BTreeSetStatUpdate',
      },
      set_asset_transfer_compliance: {
        asset: 'PolymeshPrimitivesStatisticsAssetScope',
        transferConditions: 'BTreeSetTransferCondition',
      },
      set_entities_exempt: {
        isExempt: 'bool',
        exemptKey: 'PolymeshPrimitivesTransferComplianceTransferConditionExemptKey',
        entities: 'BTreeSetIdentityId'
      }
    }
  },
  /**
   * Lookup512: BTreeSet<polymesh_primitives::statistics::StatType>
   **/
  BTreeSetStatType: 'Vec<PolymeshPrimitivesStatisticsStatType>',
  /**
   * Lookup513: BTreeSet<polymesh_primitives::statistics::StatUpdate>
   **/
  BTreeSetStatUpdate: 'Vec<PolymeshPrimitivesStatisticsStatUpdate>',
  /**
   * Lookup514: BTreeSet<polymesh_primitives::transfer_compliance::TransferCondition>
   **/
  BTreeSetTransferCondition: 'Vec<PolymeshPrimitivesTransferComplianceTransferCondition>',
  /**
   * Lookup515: BTreeSet<polymesh_primitives::identity_id::IdentityId>
   **/
  BTreeSetIdentityId: 'Vec<PolymeshPrimitivesIdentityId>',
  /**
   * Lookup516: pallet_sto::Call<T>
   **/
  PalletStoCall: {
    _enum: {
      create_fundraiser: {
        offeringPortfolio: 'PolymeshPrimitivesIdentityIdPortfolioId',
        offeringAsset: 'PolymeshPrimitivesTicker',
        raisingPortfolio: 'PolymeshPrimitivesIdentityIdPortfolioId',
        raisingAsset: 'PolymeshPrimitivesTicker',
        tiers: 'Vec<PalletStoPriceTier>',
        venueId: 'u64',
        start: 'Option<u64>',
        end: 'Option<u64>',
        minimumInvestment: 'u128',
        fundraiserName: 'Bytes',
      },
      invest: {
        investmentPortfolio: 'PolymeshPrimitivesIdentityIdPortfolioId',
        fundingPortfolio: 'PolymeshPrimitivesIdentityIdPortfolioId',
        offeringAsset: 'PolymeshPrimitivesTicker',
        id: 'u64',
        purchaseAmount: 'u128',
        maxPrice: 'Option<u128>',
        receipt: 'Option<PalletSettlementReceiptDetails>',
      },
      freeze_fundraiser: {
        offeringAsset: 'PolymeshPrimitivesTicker',
        id: 'u64',
      },
      unfreeze_fundraiser: {
        offeringAsset: 'PolymeshPrimitivesTicker',
        id: 'u64',
      },
      modify_fundraiser_window: {
        offeringAsset: 'PolymeshPrimitivesTicker',
        id: 'u64',
        start: 'u64',
        end: 'Option<u64>',
      },
      stop: {
        offeringAsset: 'PolymeshPrimitivesTicker',
        id: 'u64'
      }
    }
  },
  /**
   * Lookup518: pallet_sto::PriceTier
   **/
  PalletStoPriceTier: {
    total: 'u128',
    price: 'u128'
  },
  /**
   * Lookup520: pallet_treasury::Call<T>
   **/
  PalletTreasuryCall: {
    _enum: {
      disbursement: {
        beneficiaries: 'Vec<PolymeshPrimitivesBeneficiary>',
      },
      reimbursement: {
        amount: 'u128'
      }
    }
  },
  /**
   * Lookup522: polymesh_primitives::Beneficiary<Balance>
   **/
  PolymeshPrimitivesBeneficiary: {
    id: 'PolymeshPrimitivesIdentityId',
    amount: 'u128'
  },
  /**
   * Lookup523: pallet_utility::Call<T>
   **/
  PalletUtilityCall: {
    _enum: {
      batch: {
        calls: 'Vec<Call>',
      },
      batch_atomic: {
        calls: 'Vec<Call>',
      },
      batch_optimistic: {
        calls: 'Vec<Call>',
      },
      relay_tx: {
        target: 'AccountId32',
        signature: 'SpRuntimeMultiSignature',
        call: 'PalletUtilityUniqueCall'
      }
    }
  },
  /**
   * Lookup525: pallet_utility::UniqueCall<polymesh_runtime_develop::runtime::Call>
   **/
  PalletUtilityUniqueCall: {
    nonce: 'u64',
    call: 'Call'
  },
  /**
   * Lookup526: pallet_base::Call<T>
   **/
  PalletBaseCall: 'Null',
  /**
   * Lookup527: pallet_external_agents::Call<T>
   **/
  PalletExternalAgentsCall: {
    _enum: {
      create_group: {
        ticker: 'PolymeshPrimitivesTicker',
        perms: 'PolymeshPrimitivesSubsetSubsetRestrictionPalletPermissions',
      },
      set_group_permissions: {
        ticker: 'PolymeshPrimitivesTicker',
        id: 'u32',
        perms: 'PolymeshPrimitivesSubsetSubsetRestrictionPalletPermissions',
      },
      remove_agent: {
        ticker: 'PolymeshPrimitivesTicker',
        agent: 'PolymeshPrimitivesIdentityId',
      },
      abdicate: {
        ticker: 'PolymeshPrimitivesTicker',
      },
      change_group: {
        ticker: 'PolymeshPrimitivesTicker',
        agent: 'PolymeshPrimitivesIdentityId',
        group: 'PolymeshPrimitivesAgentAgentGroup',
      },
      accept_become_agent: {
        authId: 'u64',
      },
      create_group_and_add_auth: {
        ticker: 'PolymeshPrimitivesTicker',
        perms: 'PolymeshPrimitivesSubsetSubsetRestrictionPalletPermissions',
        target: 'PolymeshPrimitivesSecondaryKeySignatory',
      },
      create_and_change_custom_group: {
        ticker: 'PolymeshPrimitivesTicker',
        perms: 'PolymeshPrimitivesSubsetSubsetRestrictionPalletPermissions',
        agent: 'PolymeshPrimitivesIdentityId'
      }
    }
  },
  /**
   * Lookup528: pallet_relayer::Call<T>
   **/
  PalletRelayerCall: {
    _enum: {
      set_paying_key: {
        userKey: 'AccountId32',
        polyxLimit: 'u128',
      },
      accept_paying_key: {
        authId: 'u64',
      },
      remove_paying_key: {
        userKey: 'AccountId32',
        payingKey: 'AccountId32',
      },
      update_polyx_limit: {
        userKey: 'AccountId32',
        polyxLimit: 'u128',
      },
      increase_polyx_limit: {
        userKey: 'AccountId32',
        amount: 'u128',
      },
      decrease_polyx_limit: {
        userKey: 'AccountId32',
        amount: 'u128'
      }
    }
  },
  /**
   * Lookup529: pallet_rewards::Call<T>
   **/
  PalletRewardsCall: {
    _enum: {
      claim_itn_reward: {
        rewardAddress: 'AccountId32',
        itnAddress: 'AccountId32',
        signature: 'SpRuntimeMultiSignature',
      },
      set_itn_reward_status: {
        itnAddress: 'AccountId32',
        status: 'PalletRewardsItnRewardStatus'
      }
    }
  },
  /**
   * Lookup530: pallet_rewards::ItnRewardStatus
   **/
  PalletRewardsItnRewardStatus: {
    _enum: {
      Unclaimed: 'u128',
      Claimed: 'Null'
    }
  },
  /**
   * Lookup531: polymesh_contracts::Call<T>
   **/
  PolymeshContractsCall: {
    _enum: {
      call: {
        contract: 'AccountId32',
        value: 'u128',
        gasLimit: 'u64',
        storageDepositLimit: 'Option<u128>',
        data: 'Bytes',
      },
      instantiate_with_code: {
        endowment: 'u128',
        gasLimit: 'u64',
        storageDepositLimit: 'Option<u128>',
        code: 'Bytes',
        data: 'Bytes',
        salt: 'Bytes',
        perms: 'PolymeshPrimitivesSecondaryKeyPermissions',
      },
      instantiate_with_hash: {
        endowment: 'u128',
        gasLimit: 'u64',
        storageDepositLimit: 'Option<u128>',
        codeHash: 'H256',
        data: 'Bytes',
        salt: 'Bytes',
        perms: 'PolymeshPrimitivesSecondaryKeyPermissions'
      }
    }
  },
  /**
   * Lookup532: pallet_preimage::pallet::Call<T>
   **/
  PalletPreimageCall: {
    _enum: {
      note_preimage: {
        bytes: 'Bytes',
      },
      unnote_preimage: {
        _alias: {
          hash_: 'hash',
        },
        hash_: 'H256',
      },
      request_preimage: {
        _alias: {
          hash_: 'hash',
        },
        hash_: 'H256',
      },
      unrequest_preimage: {
        _alias: {
          hash_: 'hash',
        },
        hash_: 'H256'
      }
    }
  },
  /**
   * Lookup533: pallet_test_utils::Call<T>
   **/
  PalletTestUtilsCall: {
    _enum: {
      register_did: {
        uid: 'PolymeshPrimitivesCddIdInvestorUid',
        secondaryKeys: 'Vec<PolymeshPrimitivesSecondaryKey>',
      },
      mock_cdd_register_did: {
        targetAccount: 'AccountId32',
      },
      get_my_did: 'Null',
      get_cdd_of: {
        of: 'AccountId32'
      }
    }
  },
  /**
   * Lookup534: pallet_committee::PolymeshVotes<BlockNumber>
   **/
  PalletCommitteePolymeshVotes: {
    index: 'u32',
    ayes: 'Vec<PolymeshPrimitivesIdentityId>',
    nays: 'Vec<PolymeshPrimitivesIdentityId>',
    expiry: 'PolymeshCommonUtilitiesMaybeBlock'
  },
  /**
   * Lookup536: pallet_committee::Error<T, I>
   **/
  PalletCommitteeError: {
    _enum: ['DuplicateVote', 'NotAMember', 'NoSuchProposal', 'ProposalExpired', 'DuplicateProposal', 'MismatchedVotingIndex', 'InvalidProportion', 'FirstVoteReject', 'ProposalsLimitReached']
  },
  /**
   * Lookup546: pallet_multisig::ProposalDetails<T>
   **/
  PalletMultisigProposalDetails: {
    approvals: 'u64',
    rejections: 'u64',
    status: 'PalletMultisigProposalStatus',
    expiry: 'Option<u64>',
    autoClose: 'bool'
  },
  /**
   * Lookup547: pallet_multisig::ProposalStatus
   **/
  PalletMultisigProposalStatus: {
    _enum: ['Invalid', 'ActiveOrExpired', 'ExecutionSuccessful', 'ExecutionFailed', 'Rejected']
  },
  /**
   * Lookup549: pallet_multisig::Error<T>
   **/
  PalletMultisigError: {
    _enum: ['CddMissing', 'ProposalMissing', 'DecodingError', 'NoSigners', 'RequiredSignaturesOutOfBounds', 'NotASigner', 'NoSuchMultisig', 'NotEnoughSigners', 'NonceOverflow', 'AlreadyVoted', 'AlreadyASigner', 'FailedToChargeFee', 'IdentityNotCreator', 'ChangeNotAllowed', 'SignerAlreadyLinkedToMultisig', 'SignerAlreadyLinkedToIdentity', 'MultisigNotAllowedToLinkToItself', 'MissingCurrentIdentity', 'NotPrimaryKey', 'ProposalAlreadyRejected', 'ProposalExpired', 'ProposalAlreadyExecuted', 'MultisigMissingIdentity', 'FailedToSchedule', 'TooManySigners']
  },
  /**
   * Lookup551: pallet_bridge::BridgeTxDetail<BlockNumber>
   **/
  PalletBridgeBridgeTxDetail: {
    amount: 'u128',
    status: 'PalletBridgeBridgeTxStatus',
    executionBlock: 'u32',
    txHash: 'H256'
  },
  /**
   * Lookup552: pallet_bridge::BridgeTxStatus
   **/
  PalletBridgeBridgeTxStatus: {
    _enum: {
      Absent: 'Null',
      Pending: 'u8',
      Frozen: 'Null',
      Timelocked: 'Null',
      Handled: 'Null'
    }
  },
  /**
   * Lookup555: pallet_bridge::Error<T>
   **/
  PalletBridgeError: {
    _enum: ['ControllerNotSet', 'BadCaller', 'BadAdmin', 'NoValidCdd', 'ProposalAlreadyHandled', 'Unauthorized', 'Frozen', 'NotFrozen', 'FrozenTx', 'BridgeLimitReached', 'Overflow', 'DivisionByZero', 'TimelockedTx']
  },
  /**
   * Lookup556: pallet_staking::StakingLedger<sp_core::crypto::AccountId32, Balance>
   **/
  PalletStakingStakingLedger: {
    stash: 'AccountId32',
    total: 'Compact<u128>',
    active: 'Compact<u128>',
    unlocking: 'Vec<PalletStakingUnlockChunk>',
    claimedRewards: 'Vec<u32>'
  },
  /**
   * Lookup558: pallet_staking::UnlockChunk<Balance>
   **/
  PalletStakingUnlockChunk: {
    value: 'Compact<u128>',
    era: 'Compact<u32>'
  },
  /**
   * Lookup559: pallet_staking::Nominations<sp_core::crypto::AccountId32>
   **/
  PalletStakingNominations: {
    targets: 'Vec<AccountId32>',
    submittedIn: 'u32',
    suppressed: 'bool'
  },
  /**
   * Lookup560: pallet_staking::ActiveEraInfo
   **/
  PalletStakingActiveEraInfo: {
    index: 'u32',
    start: 'Option<u64>'
  },
  /**
   * Lookup562: pallet_staking::EraRewardPoints<sp_core::crypto::AccountId32>
   **/
  PalletStakingEraRewardPoints: {
    total: 'u32',
    individual: 'BTreeMap<AccountId32, u32>'
  },
  /**
   * Lookup565: pallet_staking::Forcing
   **/
  PalletStakingForcing: {
    _enum: ['NotForcing', 'ForceNew', 'ForceNone', 'ForceAlways']
  },
  /**
   * Lookup567: pallet_staking::UnappliedSlash<sp_core::crypto::AccountId32, Balance>
   **/
  PalletStakingUnappliedSlash: {
    validator: 'AccountId32',
    own: 'u128',
    others: 'Vec<(AccountId32,u128)>',
    reporters: 'Vec<AccountId32>',
    payout: 'u128'
  },
  /**
   * Lookup571: pallet_staking::slashing::SlashingSpans
   **/
  PalletStakingSlashingSlashingSpans: {
    spanIndex: 'u32',
    lastStart: 'u32',
    lastNonzeroSlash: 'u32',
    prior: 'Vec<u32>'
  },
  /**
   * Lookup572: pallet_staking::slashing::SpanRecord<Balance>
   **/
  PalletStakingSlashingSpanRecord: {
    slashed: 'u128',
    paidOut: 'u128'
  },
  /**
   * Lookup573: pallet_staking::ElectionResult<sp_core::crypto::AccountId32, Balance>
   **/
  PalletStakingElectionResult: {
    electedStashes: 'Vec<AccountId32>',
    exposures: 'Vec<(AccountId32,PalletStakingExposure)>',
    compute: 'PalletStakingElectionCompute'
  },
  /**
   * Lookup574: pallet_staking::ElectionStatus<BlockNumber>
   **/
  PalletStakingElectionStatus: {
    _enum: {
      Closed: 'Null',
      Open: 'u32'
    }
  },
  /**
   * Lookup575: pallet_staking::PermissionedIdentityPrefs
   **/
  PalletStakingPermissionedIdentityPrefs: {
    intendedCount: 'u32',
    runningCount: 'u32'
  },
  /**
   * Lookup576: pallet_staking::Releases
   **/
  PalletStakingReleases: {
    _enum: ['V1_0_0Ancient', 'V2_0_0', 'V3_0_0', 'V4_0_0', 'V5_0_0', 'V6_0_0', 'V6_0_1', 'V7_0_0']
  },
  /**
   * Lookup577: pallet_staking::Error<T>
   **/
  PalletStakingError: {
    _enum: ['NotController', 'NotStash', 'AlreadyBonded', 'AlreadyPaired', 'EmptyTargets', 'InvalidSlashIndex', 'InsufficientValue', 'NoMoreChunks', 'NoUnlockChunk', 'FundedTarget', 'InvalidEraToReward', 'NotSortedAndUnique', 'AlreadyClaimed', 'OffchainElectionEarlySubmission', 'OffchainElectionWeakSubmission', 'SnapshotUnavailable', 'OffchainElectionBogusWinnerCount', 'OffchainElectionBogusWinner', 'OffchainElectionBogusCompact', 'OffchainElectionBogusNominator', 'OffchainElectionBogusNomination', 'OffchainElectionSlashedNomination', 'OffchainElectionBogusSelfVote', 'OffchainElectionBogusEdge', 'OffchainElectionBogusScore', 'OffchainElectionBogusElectionSize', 'CallNotAllowed', 'IncorrectSlashingSpans', 'AlreadyExists', 'NotExists', 'NoChange', 'InvalidValidatorIdentity', 'InvalidValidatorCommission', 'StashIdentityDoesNotExist', 'StashIdentityNotPermissioned', 'StashIdentityNotCDDed', 'HitIntendedValidatorCount', 'IntendedCountIsExceedingConsensusLimit', 'BondTooSmall', 'BadState', 'TooManyTargets', 'BadTarget']
  },
  /**
   * Lookup578: sp_staking::offence::OffenceDetails<sp_core::crypto::AccountId32, Offender>
   **/
  SpStakingOffenceOffenceDetails: {
    offender: '(AccountId32,PalletStakingExposure)',
    reporters: 'Vec<AccountId32>'
  },
  /**
   * Lookup583: sp_core::crypto::KeyTypeId
   **/
  SpCoreCryptoKeyTypeId: '[u8;4]',
  /**
   * Lookup584: pallet_session::pallet::Error<T>
   **/
  PalletSessionError: {
    _enum: ['InvalidProof', 'NoAssociatedValidatorId', 'DuplicatedKey', 'NoKeys', 'NoAccount']
  },
  /**
   * Lookup585: pallet_grandpa::StoredState<N>
   **/
  PalletGrandpaStoredState: {
    _enum: {
      Live: 'Null',
      PendingPause: {
        scheduledAt: 'u32',
        delay: 'u32',
      },
      Paused: 'Null',
      PendingResume: {
        scheduledAt: 'u32',
        delay: 'u32'
      }
    }
  },
  /**
   * Lookup586: pallet_grandpa::StoredPendingChange<N, Limit>
   **/
  PalletGrandpaStoredPendingChange: {
    scheduledAt: 'u32',
    delay: 'u32',
    nextAuthorities: 'Vec<(SpFinalityGrandpaAppPublic,u64)>',
    forced: 'Option<u32>'
  },
  /**
   * Lookup588: pallet_grandpa::pallet::Error<T>
   **/
  PalletGrandpaError: {
    _enum: ['PauseFailed', 'ResumeFailed', 'ChangePending', 'TooSoon', 'InvalidKeyOwnershipProof', 'InvalidEquivocationProof', 'DuplicateOffenceReport']
  },
  /**
   * Lookup592: pallet_im_online::BoundedOpaqueNetworkState<PeerIdEncodingLimit, MultiAddrEncodingLimit, AddressesLimit>
   **/
  PalletImOnlineBoundedOpaqueNetworkState: {
    peerId: 'Bytes',
    externalAddresses: 'Vec<Bytes>'
  },
  /**
   * Lookup596: pallet_im_online::pallet::Error<T>
   **/
  PalletImOnlineError: {
    _enum: ['InvalidKey', 'DuplicatedHeartbeat']
  },
  /**
   * Lookup598: pallet_sudo::Error<T>
   **/
  PalletSudoError: {
    _enum: ['RequireSudo']
  },
  /**
   * Lookup599: pallet_asset::TickerRegistration<U>
   **/
  PalletAssetTickerRegistration: {
    owner: 'PolymeshPrimitivesIdentityId',
    expiry: 'Option<u64>'
  },
  /**
   * Lookup600: pallet_asset::SecurityToken
   **/
  PalletAssetSecurityToken: {
    totalSupply: 'u128',
    ownerDid: 'PolymeshPrimitivesIdentityId',
    divisible: 'bool',
    assetType: 'PolymeshPrimitivesAssetAssetType'
  },
  /**
   * Lookup604: pallet_asset::AssetOwnershipRelation
   **/
  PalletAssetAssetOwnershipRelation: {
    _enum: ['NotOwned', 'TickerOwned', 'AssetOwned']
  },
  /**
   * Lookup606: pallet_asset::ClassicTickerRegistration
   **/
  PalletAssetClassicTickerRegistration: {
    ethOwner: 'PolymeshPrimitivesEthereumEthereumAddress',
    isCreated: 'bool'
  },
  /**
   * Lookup612: pallet_asset::Error<T>
   **/
  PalletAssetError: {
    _enum: ['Unauthorized', 'AlreadyArchived', 'AlreadyUnArchived', 'ExtensionAlreadyPresent', 'AssetAlreadyCreated', 'TickerTooLong', 'TickerNotAscii', 'TickerAlreadyRegistered', 'TotalSupplyAboveLimit', 'NoSuchAsset', 'AlreadyFrozen', 'NotAnOwner', 'BalanceOverflow', 'TotalSupplyOverflow', 'InvalidGranularity', 'NotFrozen', 'InvalidTransfer', 'InsufficientBalance', 'AssetAlreadyDivisible', 'MaximumTMExtensionLimitReached', 'IncompatibleExtensionVersion', 'InvalidEthereumSignature', 'NoSuchClassicTicker', 'TickerRegistrationExpired', 'SenderSameAsReceiver', 'NoSuchDoc', 'MaxLengthOfAssetNameExceeded', 'FundingRoundNameMaxLengthExceeded', 'InvalidAssetIdentifier', 'InvestorUniquenessClaimNotAllowed', 'InvalidCustomAssetTypeId', 'AssetMetadataNameMaxLengthExceeded', 'AssetMetadataValueMaxLengthExceeded', 'AssetMetadataTypeDefMaxLengthExceeded', 'AssetMetadataKeyIsMissing', 'AssetMetadataValueIsLocked', 'AssetMetadataLocalKeyAlreadyExists', 'AssetMetadataGlobalKeyAlreadyExists']
  },
  /**
   * Lookup615: pallet_corporate_actions::distribution::Error<T>
   **/
  PalletCorporateActionsDistributionError: {
    _enum: ['CANotBenefit', 'AlreadyExists', 'ExpiryBeforePayment', 'HolderAlreadyPaid', 'NoSuchDistribution', 'CannotClaimBeforeStart', 'CannotClaimAfterExpiry', 'BalancePerShareProductOverflowed', 'NotDistributionCreator', 'AlreadyReclaimed', 'NotExpired', 'DistributionStarted', 'InsufficientRemainingAmount', 'DistributionAmountIsZero', 'DistributionPerShareIsZero']
  },
  /**
   * Lookup622: pallet_asset::checkpoint::Error<T>
   **/
  PalletAssetCheckpointError: {
    _enum: ['NoSuchSchedule', 'ScheduleNotRemovable', 'FailedToComputeNextCheckpoint', 'ScheduleDurationTooShort', 'SchedulesTooComplex']
  },
  /**
   * Lookup623: polymesh_primitives::compliance_manager::AssetCompliance
   **/
  PolymeshPrimitivesComplianceManagerAssetCompliance: {
    paused: 'bool',
    requirements: 'Vec<PolymeshPrimitivesComplianceManagerComplianceRequirement>'
  },
  /**
   * Lookup625: pallet_compliance_manager::Error<T>
   **/
  PalletComplianceManagerError: {
    _enum: ['Unauthorized', 'DidNotExist', 'InvalidComplianceRequirementId', 'IncorrectOperationOnTrustedIssuer', 'DuplicateComplianceRequirements', 'ComplianceRequirementTooComplex']
  },
  /**
   * Lookup628: pallet_corporate_actions::Error<T>
   **/
  PalletCorporateActionsError: {
    _enum: ['AuthNotCAATransfer', 'DetailsTooLong', 'DuplicateDidTax', 'TooManyDidTaxes', 'TooManyTargetIds', 'NoSuchCheckpointId', 'NoSuchCA', 'NoRecordDate', 'RecordDateAfterStart', 'DeclDateAfterRecordDate', 'DeclDateInFuture', 'NotTargetedByCA']
  },
  /**
   * Lookup630: pallet_corporate_actions::ballot::Error<T>
   **/
  PalletCorporateActionsBallotError: {
    _enum: ['CANotNotice', 'AlreadyExists', 'NoSuchBallot', 'StartAfterEnd', 'NowAfterEnd', 'NumberOfChoicesOverflow', 'VotingAlreadyStarted', 'VotingNotStarted', 'VotingAlreadyEnded', 'WrongVoteCount', 'InsufficientVotes', 'NoSuchRCVFallback', 'RCVSelfCycle', 'RCVNotAllowed']
  },
  /**
   * Lookup631: pallet_permissions::Error<T>
   **/
  PalletPermissionsError: {
    _enum: ['UnauthorizedCaller']
  },
  /**
   * Lookup632: pallet_pips::PipsMetadata<BlockNumber>
   **/
  PalletPipsPipsMetadata: {
    id: 'u32',
    url: 'Option<Bytes>',
    description: 'Option<Bytes>',
    createdAt: 'u32',
    transactionVersion: 'u32',
    expiry: 'PolymeshCommonUtilitiesMaybeBlock'
  },
  /**
   * Lookup634: pallet_pips::DepositInfo<sp_core::crypto::AccountId32>
   **/
  PalletPipsDepositInfo: {
    owner: 'AccountId32',
    amount: 'u128'
  },
  /**
   * Lookup635: pallet_pips::Pip<polymesh_runtime_develop::runtime::Call, sp_core::crypto::AccountId32>
   **/
  PalletPipsPip: {
    id: 'u32',
    proposal: 'Call',
    state: 'PalletPipsProposalState',
    proposer: 'PalletPipsProposer'
  },
  /**
   * Lookup636: pallet_pips::VotingResult
   **/
  PalletPipsVotingResult: {
    ayesCount: 'u32',
    ayesStake: 'u128',
    naysCount: 'u32',
    naysStake: 'u128'
  },
  /**
   * Lookup637: pallet_pips::Vote
   **/
  PalletPipsVote: '(bool,u128)',
  /**
   * Lookup638: pallet_pips::SnapshotMetadata<BlockNumber, sp_core::crypto::AccountId32>
   **/
  PalletPipsSnapshotMetadata: {
    createdAt: 'u32',
    madeBy: 'AccountId32',
    id: 'u32'
  },
  /**
   * Lookup640: pallet_pips::Error<T>
   **/
  PalletPipsError: {
    _enum: ['RescheduleNotByReleaseCoordinator', 'NotFromCommunity', 'NotByCommittee', 'TooManyActivePips', 'IncorrectDeposit', 'InsufficientDeposit', 'NoSuchProposal', 'NotACommitteeMember', 'InvalidFutureBlockNumber', 'NumberOfVotesExceeded', 'StakeAmountOfVotesExceeded', 'MissingCurrentIdentity', 'IncorrectProposalState', 'CannotSkipPip', 'SnapshotResultTooLarge', 'SnapshotIdMismatch', 'ScheduledProposalDoesntExist', 'ProposalNotInScheduledState']
  },
  /**
   * Lookup646: pallet_portfolio::Error<T>
   **/
  PalletPortfolioError: {
    _enum: ['PortfolioDoesNotExist', 'InsufficientPortfolioBalance', 'DestinationIsSamePortfolio', 'PortfolioNameAlreadyInUse', 'SecondaryKeyNotAuthorizedForPortfolio', 'UnauthorizedCustodian', 'InsufficientTokensLocked', 'PortfolioNotEmpty', 'DifferentIdentityPortfolios']
  },
  /**
   * Lookup647: pallet_protocol_fee::Error<T>
   **/
  PalletProtocolFeeError: {
    _enum: ['InsufficientAccountBalance', 'UnHandledImbalances', 'InsufficientSubsidyBalance']
  },
  /**
   * Lookup650: pallet_scheduler::ScheduledV3<frame_support::traits::schedule::MaybeHashed<polymesh_runtime_develop::runtime::Call, primitive_types::H256>, BlockNumber, polymesh_runtime_develop::runtime::OriginCaller, sp_core::crypto::AccountId32>
   **/
  PalletSchedulerScheduledV3: {
    maybeId: 'Option<Bytes>',
    priority: 'u8',
    call: 'FrameSupportScheduleMaybeHashed',
    maybePeriodic: 'Option<(u32,u32)>',
    origin: 'PolymeshRuntimeDevelopRuntimeOriginCaller'
  },
  /**
   * Lookup651: polymesh_runtime_develop::runtime::OriginCaller
   **/
  PolymeshRuntimeDevelopRuntimeOriginCaller: {
    _enum: {
      system: 'FrameSupportDispatchRawOrigin',
      __Unused1: 'Null',
      __Unused2: 'Null',
      __Unused3: 'Null',
      Void: 'SpCoreVoid',
      __Unused5: 'Null',
      __Unused6: 'Null',
      __Unused7: 'Null',
      __Unused8: 'Null',
      PolymeshCommittee: 'PalletCommitteeRawOriginInstance1',
      __Unused10: 'Null',
      TechnicalCommittee: 'PalletCommitteeRawOriginInstance3',
      __Unused12: 'Null',
      UpgradeCommittee: 'PalletCommitteeRawOriginInstance4'
    }
  },
  /**
   * Lookup652: frame_support::dispatch::RawOrigin<sp_core::crypto::AccountId32>
   **/
  FrameSupportDispatchRawOrigin: {
    _enum: {
      Root: 'Null',
      Signed: 'AccountId32',
      None: 'Null'
    }
  },
  /**
   * Lookup653: pallet_committee::RawOrigin<sp_core::crypto::AccountId32, pallet_committee::Instance1>
   **/
  PalletCommitteeRawOriginInstance1: {
    _enum: ['Endorsed']
  },
  /**
   * Lookup654: pallet_committee::RawOrigin<sp_core::crypto::AccountId32, pallet_committee::Instance3>
   **/
  PalletCommitteeRawOriginInstance3: {
    _enum: ['Endorsed']
  },
  /**
   * Lookup655: pallet_committee::RawOrigin<sp_core::crypto::AccountId32, pallet_committee::Instance4>
   **/
  PalletCommitteeRawOriginInstance4: {
    _enum: ['Endorsed']
  },
  /**
   * Lookup656: sp_core::Void
   **/
  SpCoreVoid: 'Null',
  /**
   * Lookup657: pallet_scheduler::pallet::Error<T>
   **/
  PalletSchedulerError: {
    _enum: ['FailedToSchedule', 'NotFound', 'TargetBlockNumberInPast', 'RescheduleNoChange']
  },
  /**
   * Lookup658: pallet_settlement::Venue
   **/
  PalletSettlementVenue: {
    creator: 'PolymeshPrimitivesIdentityId',
    venueType: 'PalletSettlementVenueType'
  },
  /**
   * Lookup661: pallet_settlement::Instruction<Moment, BlockNumber>
   **/
  PalletSettlementInstruction: {
    instructionId: 'u64',
    venueId: 'u64',
    status: 'PalletSettlementInstructionStatus',
    settlementType: 'PalletSettlementSettlementType',
    createdAt: 'Option<u64>',
    tradeDate: 'Option<u64>',
    valueDate: 'Option<u64>'
  },
  /**
   * Lookup662: pallet_settlement::InstructionStatus
   **/
  PalletSettlementInstructionStatus: {
    _enum: ['Unknown', 'Pending', 'Failed']
  },
  /**
   * Lookup664: pallet_settlement::LegStatus<sp_core::crypto::AccountId32>
   **/
  PalletSettlementLegStatus: {
    _enum: {
      PendingTokenLock: 'Null',
      ExecutionPending: 'Null',
      ExecutionToBeSkipped: '(AccountId32,u64)'
    }
  },
  /**
   * Lookup666: pallet_settlement::AffirmationStatus
   **/
  PalletSettlementAffirmationStatus: {
    _enum: ['Unknown', 'Pending', 'Affirmed']
  },
  /**
   * Lookup670: pallet_settlement::Error<T>
   **/
  PalletSettlementError: {
    _enum: ['InvalidVenue', 'Unauthorized', 'NoPendingAffirm', 'InstructionNotAffirmed', 'InstructionNotPending', 'InstructionNotFailed', 'LegNotPending', 'UnauthorizedSigner', 'ReceiptAlreadyClaimed', 'ReceiptNotClaimed', 'UnauthorizedVenue', 'FailedToLockTokens', 'InstructionFailed', 'InstructionDatesInvalid', 'InstructionSettleBlockPassed', 'InvalidSignature', 'SameSenderReceiver', 'PortfolioMismatch', 'SettleOnPastBlock', 'NoPortfolioProvided', 'UnexpectedAffirmationStatus', 'FailedToSchedule', 'LegCountTooSmall', 'UnknownInstruction', 'InstructionHasTooManyLegs']
  },
  /**
   * Lookup672: polymesh_primitives::statistics::Stat1stKey
   **/
  PolymeshPrimitivesStatisticsStat1stKey: {
    asset: 'PolymeshPrimitivesStatisticsAssetScope',
    statType: 'PolymeshPrimitivesStatisticsStatType'
  },
  /**
   * Lookup673: polymesh_primitives::transfer_compliance::AssetTransferCompliance
   **/
  PolymeshPrimitivesTransferComplianceAssetTransferCompliance: {
    paused: 'bool',
    requirements: 'BTreeSetTransferCondition'
  },
  /**
   * Lookup676: pallet_statistics::Error<T>
   **/
  PalletStatisticsError: {
    _enum: ['InvalidTransfer', 'StatTypeMissing', 'StatTypeNeededByTransferCondition', 'CannotRemoveStatTypeInUse', 'StatTypeLimitReached', 'TransferConditionLimitReached']
  },
  /**
   * Lookup678: pallet_sto::Error<T>
   **/
  PalletStoError: {
    _enum: ['Unauthorized', 'Overflow', 'InsufficientTokensRemaining', 'FundraiserNotFound', 'FundraiserNotLive', 'FundraiserClosed', 'FundraiserExpired', 'InvalidVenue', 'InvalidPriceTiers', 'InvalidOfferingWindow', 'MaxPriceExceeded', 'InvestmentAmountTooLow']
  },
  /**
   * Lookup679: pallet_treasury::Error<T>
   **/
  PalletTreasuryError: {
    _enum: ['InsufficientBalance', 'InvalidIdentity']
  },
  /**
   * Lookup680: pallet_utility::Error<T>
   **/
  PalletUtilityError: {
    _enum: ['InvalidSignature', 'TargetCddMissing', 'InvalidNonce']
  },
  /**
   * Lookup681: pallet_base::Error<T>
   **/
  PalletBaseError: {
    _enum: ['TooLong', 'CounterOverflow']
  },
  /**
   * Lookup683: pallet_external_agents::Error<T>
   **/
  PalletExternalAgentsError: {
    _enum: ['NoSuchAG', 'UnauthorizedAgent', 'AlreadyAnAgent', 'NotAnAgent', 'RemovingLastFullAgent', 'SecondaryKeyNotAuthorizedForAsset']
  },
  /**
   * Lookup684: pallet_relayer::Subsidy<sp_core::crypto::AccountId32>
   **/
  PalletRelayerSubsidy: {
    payingKey: 'AccountId32',
    remaining: 'u128'
  },
  /**
   * Lookup685: pallet_relayer::Error<T>
   **/
  PalletRelayerError: {
    _enum: ['UserKeyCddMissing', 'PayingKeyCddMissing', 'NoPayingKey', 'NotPayingKey', 'NotAuthorizedForPayingKey', 'NotAuthorizedForUserKey', 'Overflow']
  },
  /**
   * Lookup686: pallet_rewards::Error<T>
   **/
  PalletRewardsError: {
    _enum: ['UnknownItnAddress', 'ItnRewardAlreadyClaimed', 'InvalidSignature', 'UnableToCovertBalance']
  },
  /**
   * Lookup687: pallet_contracts::wasm::PrefabWasmModule<T>
   **/
  PalletContractsWasmPrefabWasmModule: {
    instructionWeightsVersion: 'Compact<u32>',
    initial: 'Compact<u32>',
    maximum: 'Compact<u32>',
    code: 'Bytes'
  },
  /**
   * Lookup688: pallet_contracts::wasm::OwnerInfo<T>
   **/
  PalletContractsWasmOwnerInfo: {
    owner: 'AccountId32',
    deposit: 'Compact<u128>',
    refcount: 'Compact<u64>'
  },
  /**
   * Lookup689: pallet_contracts::storage::RawContractInfo<primitive_types::H256, Balance>
   **/
  PalletContractsStorageRawContractInfo: {
    trieId: 'Bytes',
    codeHash: 'H256',
    storageDeposit: 'u128'
  },
  /**
   * Lookup691: pallet_contracts::storage::DeletedContract
   **/
  PalletContractsStorageDeletedContract: {
    trieId: 'Bytes'
  },
  /**
   * Lookup692: pallet_contracts::schedule::Schedule<T>
   **/
  PalletContractsSchedule: {
    limits: 'PalletContractsScheduleLimits',
    instructionWeights: 'PalletContractsScheduleInstructionWeights',
    hostFnWeights: 'PalletContractsScheduleHostFnWeights'
  },
  /**
   * Lookup693: pallet_contracts::schedule::Limits
   **/
  PalletContractsScheduleLimits: {
    eventTopics: 'u32',
    stackHeight: 'Option<u32>',
    globals: 'u32',
    parameters: 'u32',
    memoryPages: 'u32',
    tableSize: 'u32',
    brTableSize: 'u32',
    subjectLen: 'u32',
    callDepth: 'u32',
    payloadLen: 'u32',
    codeLen: 'u32'
  },
  /**
   * Lookup694: pallet_contracts::schedule::InstructionWeights<T>
   **/
  PalletContractsScheduleInstructionWeights: {
    _alias: {
      r_if: 'r#if'
    },
    version: 'u32',
    i64const: 'u32',
    i64load: 'u32',
    i64store: 'u32',
    select: 'u32',
    r_if: 'u32',
    br: 'u32',
    brIf: 'u32',
    brTable: 'u32',
    brTablePerEntry: 'u32',
    call: 'u32',
    callIndirect: 'u32',
    callIndirectPerParam: 'u32',
    localGet: 'u32',
    localSet: 'u32',
    localTee: 'u32',
    globalGet: 'u32',
    globalSet: 'u32',
    memoryCurrent: 'u32',
    memoryGrow: 'u32',
    i64clz: 'u32',
    i64ctz: 'u32',
    i64popcnt: 'u32',
    i64eqz: 'u32',
    i64extendsi32: 'u32',
    i64extendui32: 'u32',
    i32wrapi64: 'u32',
    i64eq: 'u32',
    i64ne: 'u32',
    i64lts: 'u32',
    i64ltu: 'u32',
    i64gts: 'u32',
    i64gtu: 'u32',
    i64les: 'u32',
    i64leu: 'u32',
    i64ges: 'u32',
    i64geu: 'u32',
    i64add: 'u32',
    i64sub: 'u32',
    i64mul: 'u32',
    i64divs: 'u32',
    i64divu: 'u32',
    i64rems: 'u32',
    i64remu: 'u32',
    i64and: 'u32',
    i64or: 'u32',
    i64xor: 'u32',
    i64shl: 'u32',
    i64shrs: 'u32',
    i64shru: 'u32',
    i64rotl: 'u32',
    i64rotr: 'u32'
  },
  /**
   * Lookup695: pallet_contracts::schedule::HostFnWeights<T>
   **/
  PalletContractsScheduleHostFnWeights: {
    _alias: {
      r_return: 'r#return'
    },
    caller: 'u64',
    isContract: 'u64',
    codeHash: 'u64',
    ownCodeHash: 'u64',
    callerIsOrigin: 'u64',
    address: 'u64',
    gasLeft: 'u64',
    balance: 'u64',
    valueTransferred: 'u64',
    minimumBalance: 'u64',
    blockNumber: 'u64',
    now: 'u64',
    weightToFee: 'u64',
    gas: 'u64',
    input: 'u64',
    inputPerByte: 'u64',
    r_return: 'u64',
    returnPerByte: 'u64',
    terminate: 'u64',
    random: 'u64',
    depositEvent: 'u64',
    depositEventPerTopic: 'u64',
    depositEventPerByte: 'u64',
    debugMessage: 'u64',
    setStorage: 'u64',
    setStoragePerNewByte: 'u64',
    setStoragePerOldByte: 'u64',
    setCodeHash: 'u64',
    clearStorage: 'u64',
    clearStoragePerByte: 'u64',
    containsStorage: 'u64',
    containsStoragePerByte: 'u64',
    getStorage: 'u64',
    getStoragePerByte: 'u64',
    takeStorage: 'u64',
    takeStoragePerByte: 'u64',
    transfer: 'u64',
    call: 'u64',
    delegateCall: 'u64',
    callTransferSurcharge: 'u64',
    callPerClonedByte: 'u64',
    instantiate: 'u64',
    instantiateTransferSurcharge: 'u64',
    instantiatePerSaltByte: 'u64',
    hashSha2256: 'u64',
    hashSha2256PerByte: 'u64',
    hashKeccak256: 'u64',
    hashKeccak256PerByte: 'u64',
    hashBlake2256: 'u64',
    hashBlake2256PerByte: 'u64',
    hashBlake2128: 'u64',
    hashBlake2128PerByte: 'u64',
    ecdsaRecover: 'u64',
    ecdsaToEthAddress: 'u64'
  },
  /**
   * Lookup696: pallet_contracts::pallet::Error<T>
   **/
  PalletContractsError: {
    _enum: ['InvalidScheduleVersion', 'InvalidCallFlags', 'OutOfGas', 'OutputBufferTooSmall', 'TransferFailed', 'MaxCallDepthReached', 'ContractNotFound', 'CodeTooLarge', 'CodeNotFound', 'OutOfBounds', 'DecodingFailed', 'ContractTrapped', 'ValueTooLarge', 'TerminatedWhileReentrant', 'InputForwarded', 'RandomSubjectTooLong', 'TooManyTopics', 'DuplicateTopics', 'NoChainExtension', 'DeletionQueueFull', 'DuplicateContract', 'TerminatedInConstructor', 'DebugMessageInvalidUTF8', 'ReentranceDenied', 'StorageDepositNotEnoughFunds', 'StorageDepositLimitExhausted', 'CodeInUse', 'ContractReverted', 'CodeRejected']
  },
  /**
   * Lookup697: polymesh_contracts::Error<T>
   **/
  PolymeshContractsError: {
    _enum: ['RuntimeCallNotFound', 'DataLeftAfterDecoding', 'InLenTooLarge', 'InstantiatorWithNoIdentity']
  },
  /**
   * Lookup698: pallet_preimage::RequestStatus<sp_core::crypto::AccountId32, Balance>
   **/
  PalletPreimageRequestStatus: {
    _enum: {
      Unrequested: 'Option<(AccountId32,u128)>',
      Requested: 'u32'
    }
  },
  /**
   * Lookup701: pallet_preimage::pallet::Error<T>
   **/
  PalletPreimageError: {
    _enum: ['TooLarge', 'AlreadyNoted', 'NotAuthorized', 'NotNoted', 'Requested', 'NotRequested']
  },
  /**
   * Lookup702: pallet_test_utils::Error<T>
   **/
  PalletTestUtilsError: 'Null',
  /**
   * Lookup705: frame_system::extensions::check_spec_version::CheckSpecVersion<T>
   **/
  FrameSystemExtensionsCheckSpecVersion: 'Null',
  /**
   * Lookup706: frame_system::extensions::check_tx_version::CheckTxVersion<T>
   **/
  FrameSystemExtensionsCheckTxVersion: 'Null',
  /**
   * Lookup707: frame_system::extensions::check_genesis::CheckGenesis<T>
   **/
  FrameSystemExtensionsCheckGenesis: 'Null',
  /**
   * Lookup710: frame_system::extensions::check_nonce::CheckNonce<T>
   **/
  FrameSystemExtensionsCheckNonce: 'Compact<u32>',
  /**
   * Lookup711: polymesh_extensions::check_weight::CheckWeight<T>
   **/
  PolymeshExtensionsCheckWeight: 'FrameSystemExtensionsCheckWeight',
  /**
   * Lookup712: frame_system::extensions::check_weight::CheckWeight<T>
   **/
  FrameSystemExtensionsCheckWeight: 'Null',
  /**
   * Lookup713: pallet_transaction_payment::ChargeTransactionPayment<T>
   **/
  PalletTransactionPaymentChargeTransactionPayment: 'Compact<u128>',
  /**
   * Lookup714: pallet_permissions::StoreCallMetadata<T>
   **/
  PalletPermissionsStoreCallMetadata: 'Null',
  /**
   * Lookup715: polymesh_runtime_develop::runtime::Runtime
   **/
  PolymeshRuntimeDevelopRuntime: 'Null'
};
