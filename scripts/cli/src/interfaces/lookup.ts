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
   * Lookup7: frame_support::dispatch::PerDispatchClass<sp_weights::weight_v2::Weight>
   **/
  FrameSupportDispatchPerDispatchClassWeight: {
    normal: 'SpWeightsWeightV2Weight',
    operational: 'SpWeightsWeightV2Weight',
    mandatory: 'SpWeightsWeightV2Weight'
  },
  /**
   * Lookup8: sp_weights::weight_v2::Weight
   **/
  SpWeightsWeightV2Weight: {
    refTime: 'Compact<u64>',
    proofSize: 'Compact<u64>'
  },
  /**
   * Lookup13: sp_runtime::generic::digest::Digest
   **/
  SpRuntimeDigest: {
    logs: 'Vec<SpRuntimeDigestDigestItem>'
  },
  /**
   * Lookup15: sp_runtime::generic::digest::DigestItem
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
   * Lookup18: frame_system::EventRecord<polymesh_runtime_develop::runtime::RuntimeEvent, primitive_types::H256>
   **/
  FrameSystemEventRecord: {
    phase: 'FrameSystemPhase',
    event: 'Event',
    topics: 'Vec<H256>'
  },
  /**
   * Lookup20: frame_system::pallet::Event<T>
   **/
  FrameSystemEvent: {
    _enum: {
      ExtrinsicSuccess: {
        dispatchInfo: 'FrameSupportDispatchDispatchInfo',
      },
      ExtrinsicFailed: {
        dispatchError: 'SpRuntimeDispatchError',
        dispatchInfo: 'FrameSupportDispatchDispatchInfo',
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
   * Lookup21: frame_support::dispatch::DispatchInfo
   **/
  FrameSupportDispatchDispatchInfo: {
    weight: 'SpWeightsWeightV2Weight',
    class: 'FrameSupportDispatchDispatchClass',
    paysFee: 'FrameSupportDispatchPays'
  },
  /**
   * Lookup22: frame_support::dispatch::DispatchClass
   **/
  FrameSupportDispatchDispatchClass: {
    _enum: ['Normal', 'Operational', 'Mandatory']
  },
  /**
   * Lookup23: frame_support::dispatch::Pays
   **/
  FrameSupportDispatchPays: {
    _enum: ['Yes', 'No']
  },
  /**
   * Lookup24: sp_runtime::DispatchError
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
      Arithmetic: 'SpArithmeticArithmeticError',
      Transactional: 'SpRuntimeTransactionalError',
      Exhausted: 'Null',
      Corruption: 'Null',
      Unavailable: 'Null'
    }
  },
  /**
   * Lookup25: sp_runtime::ModuleError
   **/
  SpRuntimeModuleError: {
    index: 'u8',
    error: '[u8;4]'
  },
  /**
   * Lookup26: sp_runtime::TokenError
   **/
  SpRuntimeTokenError: {
    _enum: ['NoFunds', 'WouldDie', 'BelowMinimum', 'CannotCreate', 'UnknownAsset', 'Frozen', 'Unsupported']
  },
  /**
   * Lookup27: sp_arithmetic::ArithmeticError
   **/
  SpArithmeticArithmeticError: {
    _enum: ['Underflow', 'Overflow', 'DivisionByZero']
  },
  /**
   * Lookup28: sp_runtime::TransactionalError
   **/
  SpRuntimeTransactionalError: {
    _enum: ['LimitReached', 'NoLayer']
  },
  /**
   * Lookup29: pallet_indices::pallet::Event<T>
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
   * Lookup30: polymesh_common_utilities::traits::balances::RawEvent<sp_core::crypto::AccountId32>
   **/
  PolymeshCommonUtilitiesBalancesRawEvent: {
    _enum: {
      Endowed: '(Option<PolymeshPrimitivesIdentityId>,AccountId32,u128)',
      Transfer: '(Option<PolymeshPrimitivesIdentityId>,AccountId32,Option<PolymeshPrimitivesIdentityId>,AccountId32,u128,Option<PolymeshPrimitivesMemo>)',
      BalanceSet: '(PolymeshPrimitivesIdentityId,AccountId32,u128,u128)',
      AccountBalanceBurned: '(PolymeshPrimitivesIdentityId,AccountId32,u128)',
      Reserved: '(AccountId32,u128)',
      Unreserved: '(AccountId32,u128)',
      ReserveRepatriated: '(AccountId32,AccountId32,u128,FrameSupportTokensMiscBalanceStatus)'
    }
  },
  /**
   * Lookup32: polymesh_primitives::identity_id::IdentityId
   **/
  PolymeshPrimitivesIdentityId: '[u8;32]',
  /**
   * Lookup34: polymesh_primitives::Memo
   **/
  PolymeshPrimitivesMemo: '[u8;32]',
  /**
   * Lookup35: frame_support::traits::tokens::misc::BalanceStatus
   **/
  FrameSupportTokensMiscBalanceStatus: {
    _enum: ['Free', 'Reserved']
  },
  /**
   * Lookup36: pallet_transaction_payment::RawEvent<Balance, sp_core::crypto::AccountId32>
   **/
  PalletTransactionPaymentRawEvent: {
    _enum: {
      TransactionFeePaid: {
        who: 'AccountId32',
        actualFee: 'u128',
        tip: 'u128'
      }
    }
  },
  /**
   * Lookup37: polymesh_common_utilities::traits::identity::RawEvent<sp_core::crypto::AccountId32, Moment>
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
      AuthorizationRetryLimitReached: '(Option<PolymeshPrimitivesIdentityId>,Option<AccountId32>,u64)',
      CddRequirementForPrimaryKeyUpdated: 'bool',
      CddClaimsInvalidated: '(PolymeshPrimitivesIdentityId,u64)',
      SecondaryKeysFrozen: 'PolymeshPrimitivesIdentityId',
      SecondaryKeysUnfrozen: 'PolymeshPrimitivesIdentityId',
      CustomClaimTypeAdded: '(PolymeshPrimitivesIdentityId,u32,Bytes)',
      ChildDidCreated: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesIdentityId,AccountId32)',
      ChildDidUnlinked: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesIdentityId,PolymeshPrimitivesIdentityId)'
    }
  },
  /**
   * Lookup39: polymesh_primitives::secondary_key::SecondaryKey<sp_core::crypto::AccountId32>
   **/
  PolymeshPrimitivesSecondaryKey: {
    key: 'AccountId32',
    permissions: 'PolymeshPrimitivesSecondaryKeyPermissions'
  },
  /**
   * Lookup40: polymesh_primitives::secondary_key::Permissions
   **/
  PolymeshPrimitivesSecondaryKeyPermissions: {
    asset: 'PolymeshPrimitivesSubsetSubsetRestrictionTicker',
    extrinsic: 'PolymeshPrimitivesSubsetSubsetRestrictionPalletPermissions',
    portfolio: 'PolymeshPrimitivesSubsetSubsetRestrictionPortfolioId'
  },
  /**
   * Lookup41: polymesh_primitives::subset::SubsetRestriction<polymesh_primitives::ticker::Ticker>
   **/
  PolymeshPrimitivesSubsetSubsetRestrictionTicker: {
    _enum: {
      Whole: 'Null',
      These: 'BTreeSet<PolymeshPrimitivesTicker>',
      Except: 'BTreeSet<PolymeshPrimitivesTicker>'
    }
  },
  /**
   * Lookup42: polymesh_primitives::ticker::Ticker
   **/
  PolymeshPrimitivesTicker: '[u8;12]',
  /**
   * Lookup46: polymesh_primitives::subset::SubsetRestriction<polymesh_primitives::secondary_key::PalletPermissions>
   **/
  PolymeshPrimitivesSubsetSubsetRestrictionPalletPermissions: {
    _enum: {
      Whole: 'Null',
      These: 'BTreeSet<PolymeshPrimitivesSecondaryKeyPalletPermissions>',
      Except: 'BTreeSet<PolymeshPrimitivesSecondaryKeyPalletPermissions>'
    }
  },
  /**
   * Lookup47: polymesh_primitives::secondary_key::PalletPermissions
   **/
  PolymeshPrimitivesSecondaryKeyPalletPermissions: {
    palletName: 'Bytes',
    dispatchableNames: 'PolymeshPrimitivesSubsetSubsetRestrictionDispatchableName'
  },
  /**
   * Lookup49: polymesh_primitives::subset::SubsetRestriction<polymesh_primitives::DispatchableName>
   **/
  PolymeshPrimitivesSubsetSubsetRestrictionDispatchableName: {
    _enum: {
      Whole: 'Null',
      These: 'BTreeSet<Bytes>',
      Except: 'BTreeSet<Bytes>'
    }
  },
  /**
   * Lookup55: polymesh_primitives::subset::SubsetRestriction<polymesh_primitives::identity_id::PortfolioId>
   **/
  PolymeshPrimitivesSubsetSubsetRestrictionPortfolioId: {
    _enum: {
      Whole: 'Null',
      These: 'BTreeSet<PolymeshPrimitivesIdentityIdPortfolioId>',
      Except: 'BTreeSet<PolymeshPrimitivesIdentityIdPortfolioId>'
    }
  },
  /**
   * Lookup56: polymesh_primitives::identity_id::PortfolioId
   **/
  PolymeshPrimitivesIdentityIdPortfolioId: {
    did: 'PolymeshPrimitivesIdentityId',
    kind: 'PolymeshPrimitivesIdentityIdPortfolioKind'
  },
  /**
   * Lookup57: polymesh_primitives::identity_id::PortfolioKind
   **/
  PolymeshPrimitivesIdentityIdPortfolioKind: {
    _enum: {
      Default: 'Null',
      User: 'u64'
    }
  },
  /**
   * Lookup62: polymesh_primitives::identity_claim::IdentityClaim
   **/
  PolymeshPrimitivesIdentityClaim: {
    claimIssuer: 'PolymeshPrimitivesIdentityId',
    issuanceDate: 'u64',
    lastUpdateDate: 'u64',
    expiry: 'Option<u64>',
    claim: 'PolymeshPrimitivesIdentityClaimClaim'
  },
  /**
   * Lookup64: polymesh_primitives::identity_claim::Claim
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
      InvestorUniquenessV2: 'PolymeshPrimitivesCddId',
      Custom: '(u32,Option<PolymeshPrimitivesIdentityClaimScope>)'
    }
  },
  /**
   * Lookup65: polymesh_primitives::identity_claim::Scope
   **/
  PolymeshPrimitivesIdentityClaimScope: {
    _enum: {
      Identity: 'PolymeshPrimitivesIdentityId',
      Ticker: 'PolymeshPrimitivesTicker',
      Custom: 'Bytes'
    }
  },
  /**
   * Lookup66: polymesh_primitives::cdd_id::CddId
   **/
  PolymeshPrimitivesCddId: '[u8;32]',
  /**
   * Lookup67: polymesh_primitives::jurisdiction::CountryCode
   **/
  PolymeshPrimitivesJurisdictionCountryCode: {
    _enum: ['AF', 'AX', 'AL', 'DZ', 'AS', 'AD', 'AO', 'AI', 'AQ', 'AG', 'AR', 'AM', 'AW', 'AU', 'AT', 'AZ', 'BS', 'BH', 'BD', 'BB', 'BY', 'BE', 'BZ', 'BJ', 'BM', 'BT', 'BO', 'BA', 'BW', 'BV', 'BR', 'VG', 'IO', 'BN', 'BG', 'BF', 'BI', 'KH', 'CM', 'CA', 'CV', 'KY', 'CF', 'TD', 'CL', 'CN', 'HK', 'MO', 'CX', 'CC', 'CO', 'KM', 'CG', 'CD', 'CK', 'CR', 'CI', 'HR', 'CU', 'CY', 'CZ', 'DK', 'DJ', 'DM', 'DO', 'EC', 'EG', 'SV', 'GQ', 'ER', 'EE', 'ET', 'FK', 'FO', 'FJ', 'FI', 'FR', 'GF', 'PF', 'TF', 'GA', 'GM', 'GE', 'DE', 'GH', 'GI', 'GR', 'GL', 'GD', 'GP', 'GU', 'GT', 'GG', 'GN', 'GW', 'GY', 'HT', 'HM', 'VA', 'HN', 'HU', 'IS', 'IN', 'ID', 'IR', 'IQ', 'IE', 'IM', 'IL', 'IT', 'JM', 'JP', 'JE', 'JO', 'KZ', 'KE', 'KI', 'KP', 'KR', 'KW', 'KG', 'LA', 'LV', 'LB', 'LS', 'LR', 'LY', 'LI', 'LT', 'LU', 'MK', 'MG', 'MW', 'MY', 'MV', 'ML', 'MT', 'MH', 'MQ', 'MR', 'MU', 'YT', 'MX', 'FM', 'MD', 'MC', 'MN', 'ME', 'MS', 'MA', 'MZ', 'MM', 'NA', 'NR', 'NP', 'NL', 'AN', 'NC', 'NZ', 'NI', 'NE', 'NG', 'NU', 'NF', 'MP', 'NO', 'OM', 'PK', 'PW', 'PS', 'PA', 'PG', 'PY', 'PE', 'PH', 'PN', 'PL', 'PT', 'PR', 'QA', 'RE', 'RO', 'RU', 'RW', 'BL', 'SH', 'KN', 'LC', 'MF', 'PM', 'VC', 'WS', 'SM', 'ST', 'SA', 'SN', 'RS', 'SC', 'SL', 'SG', 'SK', 'SI', 'SB', 'SO', 'ZA', 'GS', 'SS', 'ES', 'LK', 'SD', 'SR', 'SJ', 'SZ', 'SE', 'CH', 'SY', 'TW', 'TJ', 'TZ', 'TH', 'TL', 'TG', 'TK', 'TO', 'TT', 'TN', 'TR', 'TM', 'TC', 'TV', 'UG', 'UA', 'AE', 'GB', 'US', 'UM', 'UY', 'UZ', 'VU', 'VE', 'VN', 'VI', 'WF', 'EH', 'YE', 'ZM', 'ZW', 'BQ', 'CW', 'SX']
  },
  /**
   * Lookup71: polymesh_primitives::authorization::AuthorizationData<sp_core::crypto::AccountId32>
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
   * Lookup72: polymesh_primitives::agent::AgentGroup
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
   * Lookup75: polymesh_common_utilities::traits::group::RawEvent<sp_core::crypto::AccountId32, polymesh_runtime_develop::runtime::RuntimeEvent, pallet_group::Instance2>
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
   * Lookup76: pallet_group::Instance2
   **/
  PalletGroupInstance2: 'Null',
  /**
   * Lookup78: pallet_committee::RawEvent<primitive_types::H256, BlockNumber, pallet_committee::Instance1>
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
   * Lookup79: pallet_committee::Instance1
   **/
  PalletCommitteeInstance1: 'Null',
  /**
   * Lookup82: polymesh_common_utilities::MaybeBlock<BlockNumber>
   **/
  PolymeshCommonUtilitiesMaybeBlock: {
    _enum: {
      Some: 'u32',
      None: 'Null'
    }
  },
  /**
   * Lookup83: polymesh_common_utilities::traits::group::RawEvent<sp_core::crypto::AccountId32, polymesh_runtime_develop::runtime::RuntimeEvent, pallet_group::Instance1>
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
   * Lookup84: pallet_group::Instance1
   **/
  PalletGroupInstance1: 'Null',
  /**
   * Lookup85: pallet_committee::RawEvent<primitive_types::H256, BlockNumber, pallet_committee::Instance3>
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
   * Lookup86: pallet_committee::Instance3
   **/
  PalletCommitteeInstance3: 'Null',
  /**
   * Lookup87: polymesh_common_utilities::traits::group::RawEvent<sp_core::crypto::AccountId32, polymesh_runtime_develop::runtime::RuntimeEvent, pallet_group::Instance3>
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
   * Lookup88: pallet_group::Instance3
   **/
  PalletGroupInstance3: 'Null',
  /**
   * Lookup89: pallet_committee::RawEvent<primitive_types::H256, BlockNumber, pallet_committee::Instance4>
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
   * Lookup90: pallet_committee::Instance4
   **/
  PalletCommitteeInstance4: 'Null',
  /**
   * Lookup91: polymesh_common_utilities::traits::group::RawEvent<sp_core::crypto::AccountId32, polymesh_runtime_develop::runtime::RuntimeEvent, pallet_group::Instance4>
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
   * Lookup92: pallet_group::Instance4
   **/
  PalletGroupInstance4: 'Null',
  /**
   * Lookup93: polymesh_common_utilities::traits::multisig::RawEvent<sp_core::crypto::AccountId32>
   **/
  PolymeshCommonUtilitiesMultisigRawEvent: {
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
   * Lookup95: polymesh_primitives::secondary_key::Signatory<sp_core::crypto::AccountId32>
   **/
  PolymeshPrimitivesSecondaryKeySignatory: {
    _enum: {
      Identity: 'PolymeshPrimitivesIdentityId',
      Account: 'AccountId32'
    }
  },
  /**
   * Lookup96: pallet_bridge::RawEvent<sp_core::crypto::AccountId32, BlockNumber>
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
      TxRemoved: '(PolymeshPrimitivesIdentityId,PalletBridgeBridgeTx)',
      BridgeTxFailed: '(PolymeshPrimitivesIdentityId,PalletBridgeBridgeTx,SpRuntimeDispatchError)'
    }
  },
  /**
   * Lookup97: pallet_bridge::BridgeTx<sp_core::crypto::AccountId32>
   **/
  PalletBridgeBridgeTx: {
    nonce: 'u32',
    recipient: 'AccountId32',
    amount: 'u128',
    txHash: 'H256'
  },
  /**
   * Lookup100: pallet_bridge::HandledTxStatus
   **/
  PalletBridgeHandledTxStatus: {
    _enum: {
      Success: 'Null',
      Error: 'Bytes'
    }
  },
  /**
   * Lookup101: pallet_staking::RawEvent<Balance, sp_core::crypto::AccountId32>
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
   * Lookup102: pallet_staking::ElectionCompute
   **/
  PalletStakingElectionCompute: {
    _enum: ['OnChain', 'Signed', 'Unsigned']
  },
  /**
   * Lookup104: pallet_staking::SlashingSwitch
   **/
  PalletStakingSlashingSwitch: {
    _enum: ['Validator', 'ValidatorAndNominator', 'None']
  },
  /**
   * Lookup105: pallet_offences::pallet::Event
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
   * Lookup107: pallet_session::pallet::Event
   **/
  PalletSessionEvent: {
    _enum: {
      NewSession: {
        sessionIndex: 'u32'
      }
    }
  },
  /**
   * Lookup108: pallet_grandpa::pallet::Event
   **/
  PalletGrandpaEvent: {
    _enum: {
      NewAuthorities: {
        authoritySet: 'Vec<(SpConsensusGrandpaAppPublic,u64)>',
      },
      Paused: 'Null',
      Resumed: 'Null'
    }
  },
  /**
   * Lookup111: sp_consensus_grandpa::app::Public
   **/
  SpConsensusGrandpaAppPublic: 'SpCoreEd25519Public',
  /**
   * Lookup112: sp_core::ed25519::Public
   **/
  SpCoreEd25519Public: '[u8;32]',
  /**
   * Lookup113: pallet_im_online::pallet::Event<T>
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
   * Lookup114: pallet_im_online::sr25519::app_sr25519::Public
   **/
  PalletImOnlineSr25519AppSr25519Public: 'SpCoreSr25519Public',
  /**
   * Lookup115: sp_core::sr25519::Public
   **/
  SpCoreSr25519Public: '[u8;32]',
  /**
   * Lookup118: pallet_staking::Exposure<sp_core::crypto::AccountId32, Balance>
   **/
  PalletStakingExposure: {
    total: 'Compact<u128>',
    own: 'Compact<u128>',
    others: 'Vec<PalletStakingIndividualExposure>'
  },
  /**
   * Lookup121: pallet_staking::IndividualExposure<sp_core::crypto::AccountId32, Balance>
   **/
  PalletStakingIndividualExposure: {
    who: 'AccountId32',
    value: 'Compact<u128>'
  },
  /**
   * Lookup122: pallet_sudo::RawEvent<sp_core::crypto::AccountId32>
   **/
  PalletSudoRawEvent: {
    _enum: {
      Sudid: 'Result<Null, SpRuntimeDispatchError>',
      KeyChanged: 'Option<AccountId32>',
      SudoAsDone: 'Result<Null, SpRuntimeDispatchError>'
    }
  },
  /**
   * Lookup123: polymesh_common_utilities::traits::asset::RawEvent<Moment, sp_core::crypto::AccountId32>
   **/
  PolymeshCommonUtilitiesAssetRawEvent: {
    _enum: {
      AssetCreated: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesTicker,bool,PolymeshPrimitivesAssetAssetType,PolymeshPrimitivesIdentityId,bool,Bytes,Vec<PolymeshPrimitivesAssetIdentifier>,Option<Bytes>)',
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
      ControllerTransfer: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesTicker,PolymeshPrimitivesIdentityIdPortfolioId,u128)',
      CustomAssetTypeExists: '(PolymeshPrimitivesIdentityId,u32,Bytes)',
      CustomAssetTypeRegistered: '(PolymeshPrimitivesIdentityId,u32,Bytes)',
      SetAssetMetadataValue: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesTicker,Bytes,Option<PolymeshPrimitivesAssetMetadataAssetMetadataValueDetail>)',
      SetAssetMetadataValueDetails: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesTicker,PolymeshPrimitivesAssetMetadataAssetMetadataValueDetail)',
      RegisterAssetMetadataLocalType: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesTicker,Bytes,u64,PolymeshPrimitivesAssetMetadataAssetMetadataSpec)',
      RegisterAssetMetadataGlobalType: '(Bytes,u64,PolymeshPrimitivesAssetMetadataAssetMetadataSpec)',
      AssetTypeChanged: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesTicker,PolymeshPrimitivesAssetAssetType)',
      LocalMetadataKeyDeleted: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesTicker,u64)',
      MetadataValueDeleted: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesTicker,PolymeshPrimitivesAssetMetadataAssetMetadataKey)',
      AssetBalanceUpdated: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesTicker,u128,Option<PolymeshPrimitivesIdentityIdPortfolioId>,Option<PolymeshPrimitivesIdentityIdPortfolioId>,PolymeshPrimitivesPortfolioPortfolioUpdateReason)'
    }
  },
  /**
   * Lookup124: polymesh_primitives::asset::AssetType
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
      StableCoin: 'Null',
      NonFungible: 'PolymeshPrimitivesAssetNonFungibleType'
    }
  },
  /**
   * Lookup126: polymesh_primitives::asset::NonFungibleType
   **/
  PolymeshPrimitivesAssetNonFungibleType: {
    _enum: {
      Derivative: 'Null',
      FixedIncome: 'Null',
      Invoice: 'Null',
      Custom: 'u32'
    }
  },
  /**
   * Lookup129: polymesh_primitives::asset_identifier::AssetIdentifier
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
   * Lookup135: polymesh_primitives::document::Document
   **/
  PolymeshPrimitivesDocument: {
    uri: 'Bytes',
    contentHash: 'PolymeshPrimitivesDocumentHash',
    name: 'Bytes',
    docType: 'Option<Bytes>',
    filingDate: 'Option<u64>'
  },
  /**
   * Lookup137: polymesh_primitives::document_hash::DocumentHash
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
   * Lookup148: polymesh_primitives::asset_metadata::AssetMetadataValueDetail<Moment>
   **/
  PolymeshPrimitivesAssetMetadataAssetMetadataValueDetail: {
    expire: 'Option<u64>',
    lockStatus: 'PolymeshPrimitivesAssetMetadataAssetMetadataLockStatus'
  },
  /**
   * Lookup149: polymesh_primitives::asset_metadata::AssetMetadataLockStatus<Moment>
   **/
  PolymeshPrimitivesAssetMetadataAssetMetadataLockStatus: {
    _enum: {
      Unlocked: 'Null',
      Locked: 'Null',
      LockedUntil: 'u64'
    }
  },
  /**
   * Lookup152: polymesh_primitives::asset_metadata::AssetMetadataSpec
   **/
  PolymeshPrimitivesAssetMetadataAssetMetadataSpec: {
    url: 'Option<Bytes>',
    description: 'Option<Bytes>',
    typeDef: 'Option<Bytes>'
  },
  /**
   * Lookup159: polymesh_primitives::asset_metadata::AssetMetadataKey
   **/
  PolymeshPrimitivesAssetMetadataAssetMetadataKey: {
    _enum: {
      Global: 'u64',
      Local: 'u64'
    }
  },
  /**
   * Lookup161: polymesh_primitives::portfolio::PortfolioUpdateReason
   **/
  PolymeshPrimitivesPortfolioPortfolioUpdateReason: {
    _enum: {
      Issued: {
        fundingRoundName: 'Option<Bytes>',
      },
      Redeemed: 'Null',
      Transferred: {
        instructionId: 'Option<u64>',
        instructionMemo: 'Option<PolymeshPrimitivesMemo>'
      }
    }
  },
  /**
   * Lookup164: pallet_corporate_actions::distribution::Event
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
   * Lookup165: polymesh_primitives::event_only::EventOnly<polymesh_primitives::identity_id::IdentityId>
   **/
  PolymeshPrimitivesEventOnly: 'PolymeshPrimitivesIdentityId',
  /**
   * Lookup166: pallet_corporate_actions::CAId
   **/
  PalletCorporateActionsCaId: {
    ticker: 'PolymeshPrimitivesTicker',
    localId: 'u32'
  },
  /**
   * Lookup168: pallet_corporate_actions::distribution::Distribution
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
   * Lookup170: polymesh_common_utilities::traits::checkpoint::Event
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
   * Lookup173: polymesh_common_utilities::traits::checkpoint::StoredSchedule
   **/
  PolymeshCommonUtilitiesCheckpointStoredSchedule: {
    schedule: 'PolymeshPrimitivesCalendarCheckpointSchedule',
    id: 'u64',
    at: 'u64',
    remaining: 'u32'
  },
  /**
   * Lookup174: polymesh_primitives::calendar::CheckpointSchedule
   **/
  PolymeshPrimitivesCalendarCheckpointSchedule: {
    start: 'u64',
    period: 'PolymeshPrimitivesCalendarCalendarPeriod'
  },
  /**
   * Lookup175: polymesh_primitives::calendar::CalendarPeriod
   **/
  PolymeshPrimitivesCalendarCalendarPeriod: {
    unit: 'PolymeshPrimitivesCalendarCalendarUnit',
    amount: 'u64'
  },
  /**
   * Lookup176: polymesh_primitives::calendar::CalendarUnit
   **/
  PolymeshPrimitivesCalendarCalendarUnit: {
    _enum: ['Second', 'Minute', 'Hour', 'Day', 'Week', 'Month', 'Year']
  },
  /**
   * Lookup178: polymesh_common_utilities::traits::compliance_manager::Event
   **/
  PolymeshCommonUtilitiesComplianceManagerEvent: {
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
   * Lookup179: polymesh_primitives::compliance_manager::ComplianceRequirement
   **/
  PolymeshPrimitivesComplianceManagerComplianceRequirement: {
    senderConditions: 'Vec<PolymeshPrimitivesCondition>',
    receiverConditions: 'Vec<PolymeshPrimitivesCondition>',
    id: 'u32'
  },
  /**
   * Lookup181: polymesh_primitives::condition::Condition
   **/
  PolymeshPrimitivesCondition: {
    conditionType: 'PolymeshPrimitivesConditionConditionType',
    issuers: 'Vec<PolymeshPrimitivesConditionTrustedIssuer>'
  },
  /**
   * Lookup182: polymesh_primitives::condition::ConditionType
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
   * Lookup184: polymesh_primitives::condition::TargetIdentity
   **/
  PolymeshPrimitivesConditionTargetIdentity: {
    _enum: {
      ExternalAgent: 'Null',
      Specific: 'PolymeshPrimitivesIdentityId'
    }
  },
  /**
   * Lookup186: polymesh_primitives::condition::TrustedIssuer
   **/
  PolymeshPrimitivesConditionTrustedIssuer: {
    issuer: 'PolymeshPrimitivesIdentityId',
    trustedFor: 'PolymeshPrimitivesConditionTrustedFor'
  },
  /**
   * Lookup187: polymesh_primitives::condition::TrustedFor
   **/
  PolymeshPrimitivesConditionTrustedFor: {
    _enum: {
      Any: 'Null',
      Specific: 'Vec<PolymeshPrimitivesIdentityClaimClaimType>'
    }
  },
  /**
   * Lookup189: polymesh_primitives::identity_claim::ClaimType
   **/
  PolymeshPrimitivesIdentityClaimClaimType: {
    _enum: {
      Accredited: 'Null',
      Affiliate: 'Null',
      BuyLockup: 'Null',
      SellLockup: 'Null',
      CustomerDueDiligence: 'Null',
      KnowYourCustomer: 'Null',
      Jurisdiction: 'Null',
      Exempted: 'Null',
      Blocked: 'Null',
      InvestorUniqueness: 'Null',
      NoType: 'Null',
      InvestorUniquenessV2: 'Null',
      Custom: 'u32'
    }
  },
  /**
   * Lookup191: pallet_corporate_actions::Event
   **/
  PalletCorporateActionsEvent: {
    _enum: {
      MaxDetailsLengthChanged: '(PolymeshPrimitivesIdentityId,u32)',
      DefaultTargetIdentitiesChanged: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesTicker,PalletCorporateActionsTargetIdentities)',
      DefaultWithholdingTaxChanged: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesTicker,Permill)',
      DidWithholdingTaxChanged: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesTicker,PolymeshPrimitivesIdentityId,Option<Permill>)',
      CAInitiated: '(PolymeshPrimitivesEventOnly,PalletCorporateActionsCaId,PalletCorporateActionsCorporateAction,Bytes)',
      CALinkedToDoc: '(PolymeshPrimitivesIdentityId,PalletCorporateActionsCaId,Vec<u32>)',
      CARemoved: '(PolymeshPrimitivesEventOnly,PalletCorporateActionsCaId)',
      RecordDateChanged: '(PolymeshPrimitivesEventOnly,PalletCorporateActionsCaId,PalletCorporateActionsCorporateAction)'
    }
  },
  /**
   * Lookup192: pallet_corporate_actions::TargetIdentities
   **/
  PalletCorporateActionsTargetIdentities: {
    identities: 'Vec<PolymeshPrimitivesIdentityId>',
    treatment: 'PalletCorporateActionsTargetTreatment'
  },
  /**
   * Lookup193: pallet_corporate_actions::TargetTreatment
   **/
  PalletCorporateActionsTargetTreatment: {
    _enum: ['Include', 'Exclude']
  },
  /**
   * Lookup195: pallet_corporate_actions::CorporateAction
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
   * Lookup196: pallet_corporate_actions::CAKind
   **/
  PalletCorporateActionsCaKind: {
    _enum: ['PredictableBenefit', 'UnpredictableBenefit', 'IssuerNotice', 'Reorganization', 'Other']
  },
  /**
   * Lookup198: pallet_corporate_actions::RecordDate
   **/
  PalletCorporateActionsRecordDate: {
    date: 'u64',
    checkpoint: 'PalletCorporateActionsCaCheckpoint'
  },
  /**
   * Lookup199: pallet_corporate_actions::CACheckpoint
   **/
  PalletCorporateActionsCaCheckpoint: {
    _enum: {
      Scheduled: '(u64,u64)',
      Existing: 'u64'
    }
  },
  /**
   * Lookup204: pallet_corporate_actions::ballot::Event
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
   * Lookup205: pallet_corporate_actions::ballot::BallotTimeRange
   **/
  PalletCorporateActionsBallotBallotTimeRange: {
    start: 'u64',
    end: 'u64'
  },
  /**
   * Lookup206: pallet_corporate_actions::ballot::BallotMeta
   **/
  PalletCorporateActionsBallotBallotMeta: {
    title: 'Bytes',
    motions: 'Vec<PalletCorporateActionsBallotMotion>'
  },
  /**
   * Lookup209: pallet_corporate_actions::ballot::Motion
   **/
  PalletCorporateActionsBallotMotion: {
    title: 'Bytes',
    infoLink: 'Bytes',
    choices: 'Vec<Bytes>'
  },
  /**
   * Lookup215: pallet_corporate_actions::ballot::BallotVote
   **/
  PalletCorporateActionsBallotBallotVote: {
    power: 'u128',
    fallback: 'Option<u16>'
  },
  /**
   * Lookup218: pallet_pips::RawEvent<sp_core::crypto::AccountId32, BlockNumber>
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
   * Lookup219: pallet_pips::Proposer<sp_core::crypto::AccountId32>
   **/
  PalletPipsProposer: {
    _enum: {
      Community: 'AccountId32',
      Committee: 'PalletPipsCommittee'
    }
  },
  /**
   * Lookup220: pallet_pips::Committee
   **/
  PalletPipsCommittee: {
    _enum: ['Technical', 'Upgrade']
  },
  /**
   * Lookup224: pallet_pips::ProposalData
   **/
  PalletPipsProposalData: {
    _enum: {
      Hash: 'H256',
      Proposal: 'Bytes'
    }
  },
  /**
   * Lookup225: pallet_pips::ProposalState
   **/
  PalletPipsProposalState: {
    _enum: ['Pending', 'Rejected', 'Scheduled', 'Failed', 'Executed', 'Expired']
  },
  /**
   * Lookup228: pallet_pips::SnapshottedPip
   **/
  PalletPipsSnapshottedPip: {
    id: 'u32',
    weight: '(bool,u128)'
  },
  /**
   * Lookup234: polymesh_common_utilities::traits::portfolio::Event
   **/
  PolymeshCommonUtilitiesPortfolioEvent: {
    _enum: {
      PortfolioCreated: '(PolymeshPrimitivesIdentityId,u64,Bytes)',
      PortfolioDeleted: '(PolymeshPrimitivesIdentityId,u64)',
      PortfolioRenamed: '(PolymeshPrimitivesIdentityId,u64,Bytes)',
      UserPortfolios: '(PolymeshPrimitivesIdentityId,Vec<(u64,Bytes)>)',
      PortfolioCustodianChanged: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesIdentityIdPortfolioId,PolymeshPrimitivesIdentityId)',
      FundsMovedBetweenPortfolios: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesIdentityIdPortfolioId,PolymeshPrimitivesIdentityIdPortfolioId,PolymeshPrimitivesPortfolioFundDescription,Option<PolymeshPrimitivesMemo>)'
    }
  },
  /**
   * Lookup238: polymesh_primitives::portfolio::FundDescription
   **/
  PolymeshPrimitivesPortfolioFundDescription: {
    _enum: {
      Fungible: {
        ticker: 'PolymeshPrimitivesTicker',
        amount: 'u128',
      },
      NonFungible: 'PolymeshPrimitivesNftNfTs'
    }
  },
  /**
   * Lookup239: polymesh_primitives::nft::NFTs
   **/
  PolymeshPrimitivesNftNfTs: {
    ticker: 'PolymeshPrimitivesTicker',
    ids: 'Vec<u64>'
  },
  /**
   * Lookup242: pallet_protocol_fee::RawEvent<sp_core::crypto::AccountId32>
   **/
  PalletProtocolFeeRawEvent: {
    _enum: {
      FeeSet: '(PolymeshPrimitivesIdentityId,u128)',
      CoefficientSet: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesPosRatio)',
      FeeCharged: '(AccountId32,u128)'
    }
  },
  /**
   * Lookup243: polymesh_primitives::PosRatio
   **/
  PolymeshPrimitivesPosRatio: '(u32,u32)',
  /**
   * Lookup244: pallet_scheduler::pallet::Event<T>
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
        id: 'Option<[u8;32]>',
        result: 'Result<Null, SpRuntimeDispatchError>',
      },
      CallUnavailable: {
        task: '(u32,u32)',
        id: 'Option<[u8;32]>',
      },
      PeriodicFailed: {
        task: '(u32,u32)',
        id: 'Option<[u8;32]>',
      },
      PermanentlyOverweight: {
        task: '(u32,u32)',
        id: 'Option<[u8;32]>'
      }
    }
  },
  /**
   * Lookup247: polymesh_common_utilities::traits::settlement::RawEvent<Moment, BlockNumber, sp_core::crypto::AccountId32>
   **/
  PolymeshCommonUtilitiesSettlementRawEvent: {
    _enum: {
      VenueCreated: '(PolymeshPrimitivesIdentityId,u64,Bytes,PolymeshPrimitivesSettlementVenueType)',
      VenueDetailsUpdated: '(PolymeshPrimitivesIdentityId,u64,Bytes)',
      VenueTypeUpdated: '(PolymeshPrimitivesIdentityId,u64,PolymeshPrimitivesSettlementVenueType)',
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
      InstructionRescheduled: '(PolymeshPrimitivesIdentityId,u64)',
      VenueSignersUpdated: '(PolymeshPrimitivesIdentityId,u64,Vec<AccountId32>,bool)',
      SettlementManuallyExecuted: '(PolymeshPrimitivesIdentityId,u64)',
      InstructionCreated: '(PolymeshPrimitivesIdentityId,u64,u64,PolymeshPrimitivesSettlementSettlementType,Option<u64>,Option<u64>,Vec<PolymeshPrimitivesSettlementLeg>,Option<PolymeshPrimitivesMemo>)',
      FailedToExecuteInstruction: '(u64,SpRuntimeDispatchError)'
    }
  },
  /**
   * Lookup250: polymesh_primitives::settlement::VenueType
   **/
  PolymeshPrimitivesSettlementVenueType: {
    _enum: ['Other', 'Distribution', 'Sto', 'Exchange']
  },
  /**
   * Lookup254: polymesh_primitives::settlement::SettlementType<BlockNumber>
   **/
  PolymeshPrimitivesSettlementSettlementType: {
    _enum: {
      SettleOnAffirmation: 'Null',
      SettleOnBlock: 'u32',
      SettleManual: 'u32'
    }
  },
  /**
   * Lookup256: polymesh_primitives::settlement::Leg
   **/
  PolymeshPrimitivesSettlementLeg: {
    _enum: {
      Fungible: {
        sender: 'PolymeshPrimitivesIdentityIdPortfolioId',
        receiver: 'PolymeshPrimitivesIdentityIdPortfolioId',
        ticker: 'PolymeshPrimitivesTicker',
        amount: 'u128',
      },
      NonFungible: {
        sender: 'PolymeshPrimitivesIdentityIdPortfolioId',
        receiver: 'PolymeshPrimitivesIdentityIdPortfolioId',
        nfts: 'PolymeshPrimitivesNftNfTs',
      },
      OffChain: {
        senderIdentity: 'PolymeshPrimitivesIdentityId',
        receiverIdentity: 'PolymeshPrimitivesIdentityId',
        ticker: 'PolymeshPrimitivesTicker',
        amount: 'u128'
      }
    }
  },
  /**
   * Lookup257: polymesh_common_utilities::traits::statistics::Event
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
   * Lookup258: polymesh_primitives::statistics::AssetScope
   **/
  PolymeshPrimitivesStatisticsAssetScope: {
    _enum: {
      Ticker: 'PolymeshPrimitivesTicker'
    }
  },
  /**
   * Lookup260: polymesh_primitives::statistics::StatType
   **/
  PolymeshPrimitivesStatisticsStatType: {
    op: 'PolymeshPrimitivesStatisticsStatOpType',
    claimIssuer: 'Option<(PolymeshPrimitivesIdentityClaimClaimType,PolymeshPrimitivesIdentityId)>'
  },
  /**
   * Lookup261: polymesh_primitives::statistics::StatOpType
   **/
  PolymeshPrimitivesStatisticsStatOpType: {
    _enum: ['Count', 'Balance']
  },
  /**
   * Lookup265: polymesh_primitives::statistics::StatUpdate
   **/
  PolymeshPrimitivesStatisticsStatUpdate: {
    key2: 'PolymeshPrimitivesStatisticsStat2ndKey',
    value: 'Option<u128>'
  },
  /**
   * Lookup266: polymesh_primitives::statistics::Stat2ndKey
   **/
  PolymeshPrimitivesStatisticsStat2ndKey: {
    _enum: {
      NoClaimStat: 'Null',
      Claim: 'PolymeshPrimitivesStatisticsStatClaim'
    }
  },
  /**
   * Lookup267: polymesh_primitives::statistics::StatClaim
   **/
  PolymeshPrimitivesStatisticsStatClaim: {
    _enum: {
      Accredited: 'bool',
      Affiliate: 'bool',
      Jurisdiction: 'Option<PolymeshPrimitivesJurisdictionCountryCode>'
    }
  },
  /**
   * Lookup271: polymesh_primitives::transfer_compliance::TransferCondition
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
   * Lookup272: polymesh_primitives::transfer_compliance::TransferConditionExemptKey
   **/
  PolymeshPrimitivesTransferComplianceTransferConditionExemptKey: {
    asset: 'PolymeshPrimitivesStatisticsAssetScope',
    op: 'PolymeshPrimitivesStatisticsStatOpType',
    claimType: 'Option<PolymeshPrimitivesIdentityClaimClaimType>'
  },
  /**
   * Lookup274: pallet_sto::RawEvent<Moment>
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
   * Lookup277: pallet_sto::Fundraiser<Moment>
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
   * Lookup279: pallet_sto::FundraiserTier
   **/
  PalletStoFundraiserTier: {
    total: 'u128',
    price: 'u128',
    remaining: 'u128'
  },
  /**
   * Lookup280: pallet_sto::FundraiserStatus
   **/
  PalletStoFundraiserStatus: {
    _enum: ['Live', 'Frozen', 'Closed', 'ClosedEarly']
  },
  /**
   * Lookup281: pallet_treasury::RawEvent<Balance, sp_core::crypto::AccountId32>
   **/
  PalletTreasuryRawEvent: {
    _enum: {
      TreasuryDisbursement: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesIdentityId,AccountId32,u128)',
      TreasuryDisbursementFailed: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesIdentityId,AccountId32,u128)',
      TreasuryReimbursement: '(PolymeshPrimitivesIdentityId,u128)'
    }
  },
  /**
   * Lookup282: pallet_utility::pallet::Event<T>
   **/
  PalletUtilityEvent: {
    _enum: {
      BatchInterrupted: {
        index: 'u32',
        error: 'SpRuntimeDispatchError',
      },
      BatchCompleted: 'Null',
      BatchCompletedWithErrors: 'Null',
      ItemCompleted: 'Null',
      ItemFailed: {
        error: 'SpRuntimeDispatchError',
      },
      DispatchedAs: {
        result: 'Result<Null, SpRuntimeDispatchError>',
      },
      RelayedTx: {
        callerDid: 'PolymeshPrimitivesIdentityId',
        target: 'AccountId32',
        result: 'Result<Null, SpRuntimeDispatchError>',
      },
      BatchInterruptedOld: '(Vec<u32>,(u32,SpRuntimeDispatchError))',
      BatchOptimisticFailed: '(Vec<u32>,Vec<(u32,SpRuntimeDispatchError)>)',
      BatchCompletedOld: 'Vec<u32>'
    }
  },
  /**
   * Lookup286: polymesh_common_utilities::traits::base::Event
   **/
  PolymeshCommonUtilitiesBaseEvent: {
    _enum: {
      UnexpectedError: 'Option<SpRuntimeDispatchError>'
    }
  },
  /**
   * Lookup288: polymesh_common_utilities::traits::external_agents::Event
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
   * Lookup289: polymesh_common_utilities::traits::relayer::RawEvent<sp_core::crypto::AccountId32>
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
   * Lookup290: pallet_rewards::RawEvent<sp_core::crypto::AccountId32>
   **/
  PalletRewardsRawEvent: {
    _enum: {
      ItnRewardClaimed: '(AccountId32,u128)'
    }
  },
  /**
   * Lookup291: pallet_contracts::pallet::Event<T>
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
        oldCodeHash: 'H256',
      },
      Called: {
        caller: 'AccountId32',
        contract: 'AccountId32',
      },
      DelegateCalled: {
        contract: 'AccountId32',
        codeHash: 'H256'
      }
    }
  },
  /**
   * Lookup292: polymesh_contracts::Event
   **/
  PolymeshContractsEvent: 'Null',
  /**
   * Lookup293: pallet_preimage::pallet::Event<T>
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
   * Lookup294: polymesh_common_utilities::traits::nft::Event
   **/
  PolymeshCommonUtilitiesNftEvent: {
    _enum: {
      NftCollectionCreated: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesTicker,u64)',
      NFTPortfolioUpdated: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesNftNfTs,Option<PolymeshPrimitivesIdentityIdPortfolioId>,Option<PolymeshPrimitivesIdentityIdPortfolioId>,PolymeshPrimitivesPortfolioPortfolioUpdateReason)'
    }
  },
  /**
   * Lookup296: pallet_test_utils::RawEvent<sp_core::crypto::AccountId32>
   **/
  PalletTestUtilsRawEvent: {
    _enum: {
      MockInvestorUIDCreated: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesCddIdInvestorUid)',
      DidStatus: '(PolymeshPrimitivesIdentityId,AccountId32)',
      CddStatus: '(Option<PolymeshPrimitivesIdentityId>,AccountId32,bool)'
    }
  },
  /**
   * Lookup297: polymesh_primitives::cdd_id::InvestorUid
   **/
  PolymeshPrimitivesCddIdInvestorUid: '[u8;16]',
  /**
   * Lookup298: frame_system::Phase
   **/
  FrameSystemPhase: {
    _enum: {
      ApplyExtrinsic: 'u32',
      Finalization: 'Null',
      Initialization: 'Null'
    }
  },
  /**
   * Lookup301: frame_system::LastRuntimeUpgradeInfo
   **/
  FrameSystemLastRuntimeUpgradeInfo: {
    specVersion: 'Compact<u32>',
    specName: 'Text'
  },
  /**
   * Lookup304: frame_system::pallet::Call<T>
   **/
  FrameSystemCall: {
    _enum: {
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
   * Lookup308: frame_system::limits::BlockWeights
   **/
  FrameSystemLimitsBlockWeights: {
    baseBlock: 'SpWeightsWeightV2Weight',
    maxBlock: 'SpWeightsWeightV2Weight',
    perClass: 'FrameSupportDispatchPerDispatchClassWeightsPerClass'
  },
  /**
   * Lookup309: frame_support::dispatch::PerDispatchClass<frame_system::limits::WeightsPerClass>
   **/
  FrameSupportDispatchPerDispatchClassWeightsPerClass: {
    normal: 'FrameSystemLimitsWeightsPerClass',
    operational: 'FrameSystemLimitsWeightsPerClass',
    mandatory: 'FrameSystemLimitsWeightsPerClass'
  },
  /**
   * Lookup310: frame_system::limits::WeightsPerClass
   **/
  FrameSystemLimitsWeightsPerClass: {
    baseExtrinsic: 'SpWeightsWeightV2Weight',
    maxExtrinsic: 'Option<SpWeightsWeightV2Weight>',
    maxTotal: 'Option<SpWeightsWeightV2Weight>',
    reserved: 'Option<SpWeightsWeightV2Weight>'
  },
  /**
   * Lookup312: frame_system::limits::BlockLength
   **/
  FrameSystemLimitsBlockLength: {
    max: 'FrameSupportDispatchPerDispatchClassU32'
  },
  /**
   * Lookup313: frame_support::dispatch::PerDispatchClass<T>
   **/
  FrameSupportDispatchPerDispatchClassU32: {
    normal: 'u32',
    operational: 'u32',
    mandatory: 'u32'
  },
  /**
   * Lookup314: sp_weights::RuntimeDbWeight
   **/
  SpWeightsRuntimeDbWeight: {
    read: 'u64',
    write: 'u64'
  },
  /**
   * Lookup315: sp_version::RuntimeVersion
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
   * Lookup320: frame_system::pallet::Error<T>
   **/
  FrameSystemError: {
    _enum: ['InvalidSpecName', 'SpecVersionNeedsToIncrease', 'FailedToExtractRuntimeVersion', 'NonDefaultComposite', 'NonZeroRefCount', 'CallFiltered']
  },
  /**
   * Lookup323: sp_consensus_babe::app::Public
   **/
  SpConsensusBabeAppPublic: 'SpCoreSr25519Public',
  /**
   * Lookup326: sp_consensus_babe::digests::NextConfigDescriptor
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
   * Lookup328: sp_consensus_babe::AllowedSlots
   **/
  SpConsensusBabeAllowedSlots: {
    _enum: ['PrimarySlots', 'PrimaryAndSecondaryPlainSlots', 'PrimaryAndSecondaryVRFSlots']
  },
  /**
   * Lookup332: sp_consensus_babe::digests::PreDigest
   **/
  SpConsensusBabeDigestsPreDigest: {
    _enum: {
      __Unused0: 'Null',
      Primary: 'SpConsensusBabeDigestsPrimaryPreDigest',
      SecondaryPlain: 'SpConsensusBabeDigestsSecondaryPlainPreDigest',
      SecondaryVRF: 'SpConsensusBabeDigestsSecondaryVRFPreDigest'
    }
  },
  /**
   * Lookup333: sp_consensus_babe::digests::PrimaryPreDigest
   **/
  SpConsensusBabeDigestsPrimaryPreDigest: {
    authorityIndex: 'u32',
    slot: 'u64',
    vrfOutput: '[u8;32]',
    vrfProof: '[u8;64]'
  },
  /**
   * Lookup334: sp_consensus_babe::digests::SecondaryPlainPreDigest
   **/
  SpConsensusBabeDigestsSecondaryPlainPreDigest: {
    authorityIndex: 'u32',
    slot: 'u64'
  },
  /**
   * Lookup335: sp_consensus_babe::digests::SecondaryVRFPreDigest
   **/
  SpConsensusBabeDigestsSecondaryVRFPreDigest: {
    authorityIndex: 'u32',
    slot: 'u64',
    vrfOutput: '[u8;32]',
    vrfProof: '[u8;64]'
  },
  /**
   * Lookup336: sp_consensus_babe::BabeEpochConfiguration
   **/
  SpConsensusBabeBabeEpochConfiguration: {
    c: '(u64,u64)',
    allowedSlots: 'SpConsensusBabeAllowedSlots'
  },
  /**
   * Lookup340: pallet_babe::pallet::Call<T>
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
   * Lookup341: sp_consensus_slots::EquivocationProof<sp_runtime::generic::header::Header<Number, sp_runtime::traits::BlakeTwo256>, sp_consensus_babe::app::Public>
   **/
  SpConsensusSlotsEquivocationProof: {
    offender: 'SpConsensusBabeAppPublic',
    slot: 'u64',
    firstHeader: 'SpRuntimeHeader',
    secondHeader: 'SpRuntimeHeader'
  },
  /**
   * Lookup342: sp_runtime::generic::header::Header<Number, sp_runtime::traits::BlakeTwo256>
   **/
  SpRuntimeHeader: {
    parentHash: 'H256',
    number: 'Compact<u32>',
    stateRoot: 'H256',
    extrinsicsRoot: 'H256',
    digest: 'SpRuntimeDigest'
  },
  /**
   * Lookup343: sp_runtime::traits::BlakeTwo256
   **/
  SpRuntimeBlakeTwo256: 'Null',
  /**
   * Lookup344: sp_session::MembershipProof
   **/
  SpSessionMembershipProof: {
    session: 'u32',
    trieNodes: 'Vec<Bytes>',
    validatorCount: 'u32'
  },
  /**
   * Lookup345: pallet_babe::pallet::Error<T>
   **/
  PalletBabeError: {
    _enum: ['InvalidEquivocationProof', 'InvalidKeyOwnershipProof', 'DuplicateOffenceReport', 'InvalidConfiguration']
  },
  /**
   * Lookup346: pallet_timestamp::pallet::Call<T>
   **/
  PalletTimestampCall: {
    _enum: {
      set: {
        now: 'Compact<u64>'
      }
    }
  },
  /**
   * Lookup348: pallet_indices::pallet::Call<T>
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
        new_: 'MultiAddress',
        index: 'u32',
      },
      free: {
        index: 'u32',
      },
      force_transfer: {
        _alias: {
          new_: 'new',
        },
        new_: 'MultiAddress',
        index: 'u32',
        freeze: 'bool',
      },
      freeze: {
        index: 'u32'
      }
    }
  },
  /**
   * Lookup350: pallet_indices::pallet::Error<T>
   **/
  PalletIndicesError: {
    _enum: ['NotAssigned', 'NotOwner', 'InUse', 'NotTransfer', 'Permanent']
  },
  /**
   * Lookup352: pallet_balances::BalanceLock<Balance>
   **/
  PalletBalancesBalanceLock: {
    id: '[u8;8]',
    amount: 'u128',
    reasons: 'PolymeshCommonUtilitiesBalancesReasons'
  },
  /**
   * Lookup353: polymesh_common_utilities::traits::balances::Reasons
   **/
  PolymeshCommonUtilitiesBalancesReasons: {
    _enum: ['Fee', 'Misc', 'All']
  },
  /**
   * Lookup354: pallet_balances::Call<T>
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
        memo: 'Option<PolymeshPrimitivesMemo>',
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
   * Lookup355: pallet_balances::Error<T>
   **/
  PalletBalancesError: {
    _enum: ['LiquidityRestrictions', 'Overflow', 'InsufficientBalance', 'ExistentialDeposit', 'ReceiverCddMissing']
  },
  /**
   * Lookup357: pallet_transaction_payment::Releases
   **/
  PalletTransactionPaymentReleases: {
    _enum: ['V1Ancient', 'V2']
  },
  /**
   * Lookup359: sp_weights::WeightToFeeCoefficient<Balance>
   **/
  SpWeightsWeightToFeeCoefficient: {
    coeffInteger: 'u128',
    coeffFrac: 'Perbill',
    negative: 'bool',
    degree: 'u8'
  },
  /**
   * Lookup360: polymesh_primitives::identity::DidRecord<sp_core::crypto::AccountId32>
   **/
  PolymeshPrimitivesIdentityDidRecord: {
    primaryKey: 'Option<AccountId32>'
  },
  /**
   * Lookup362: pallet_identity::types::Claim1stKey
   **/
  PalletIdentityClaim1stKey: {
    target: 'PolymeshPrimitivesIdentityId',
    claimType: 'PolymeshPrimitivesIdentityClaimClaimType'
  },
  /**
   * Lookup363: pallet_identity::types::Claim2ndKey
   **/
  PalletIdentityClaim2ndKey: {
    issuer: 'PolymeshPrimitivesIdentityId',
    scope: 'Option<PolymeshPrimitivesIdentityClaimScope>'
  },
  /**
   * Lookup364: polymesh_primitives::secondary_key::KeyRecord<sp_core::crypto::AccountId32>
   **/
  PolymeshPrimitivesSecondaryKeyKeyRecord: {
    _enum: {
      PrimaryKey: 'PolymeshPrimitivesIdentityId',
      SecondaryKey: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesSecondaryKeyPermissions)',
      MultiSigSignerKey: 'AccountId32'
    }
  },
  /**
   * Lookup367: polymesh_primitives::authorization::Authorization<sp_core::crypto::AccountId32, Moment>
   **/
  PolymeshPrimitivesAuthorization: {
    authorizationData: 'PolymeshPrimitivesAuthorizationAuthorizationData',
    authorizedBy: 'PolymeshPrimitivesIdentityId',
    expiry: 'Option<u64>',
    authId: 'u64',
    count: 'u32'
  },
  /**
   * Lookup370: pallet_identity::Call<T>
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
      add_investor_uniqueness_claim: {
        target: 'PolymeshPrimitivesIdentityId',
        claim: 'PolymeshPrimitivesIdentityClaimClaim',
        proof: '[u8;64]',
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
        proof: 'ConfidentialIdentityV2ClaimProofsScopeClaimProof',
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
        keysToRemove: 'Vec<AccountId32>',
      },
      register_custom_claim_type: {
        ty: 'Bytes',
      },
      cdd_register_did_with_cdd: {
        targetAccount: 'AccountId32',
        secondaryKeys: 'Vec<PolymeshPrimitivesSecondaryKey>',
        expiry: 'Option<u64>',
      },
      create_child_identity: {
        secondaryKey: 'AccountId32',
      },
      create_child_identities: {
        childKeys: 'Vec<PolymeshCommonUtilitiesIdentityCreateChildIdentityWithAuth>',
        expiresAt: 'u64',
      },
      unlink_child_identity: {
        childDid: 'PolymeshPrimitivesIdentityId'
      }
    }
  },
  /**
   * Lookup372: confidential_identity_v2::claim_proofs::ScopeClaimProof
   **/
  ConfidentialIdentityV2ClaimProofsScopeClaimProof: {
    proofScopeIdWellformed: 'ConfidentialIdentityV2SignSignature',
    proofScopeIdCddIdMatch: 'ConfidentialIdentityV2ClaimProofsZkProofData',
    scopeId: '[u8;32]'
  },
  /**
   * Lookup373: confidential_identity_v2::sign::Signature
   **/
  ConfidentialIdentityV2SignSignature: {
    r: '[u8;32]',
    s: '[u8;32]'
  },
  /**
   * Lookup374: confidential_identity_v2::claim_proofs::ZkProofData
   **/
  ConfidentialIdentityV2ClaimProofsZkProofData: {
    challengeResponses: '[[u8;32];2]',
    subtractExpressionsRes: '[u8;32]',
    blindedScopeDidHash: '[u8;32]'
  },
  /**
   * Lookup377: polymesh_common_utilities::traits::identity::SecondaryKeyWithAuth<sp_core::crypto::AccountId32>
   **/
  PolymeshCommonUtilitiesIdentitySecondaryKeyWithAuth: {
    secondaryKey: 'PolymeshPrimitivesSecondaryKey',
    authSignature: 'H512'
  },
  /**
   * Lookup380: polymesh_common_utilities::traits::identity::CreateChildIdentityWithAuth<sp_core::crypto::AccountId32>
   **/
  PolymeshCommonUtilitiesIdentityCreateChildIdentityWithAuth: {
    key: 'AccountId32',
    authSignature: 'H512'
  },
  /**
   * Lookup381: pallet_identity::Error<T>
   **/
  PalletIdentityError: {
    _enum: ['AlreadyLinked', 'MissingCurrentIdentity', 'Unauthorized', 'InvalidAccountKey', 'UnAuthorizedCddProvider', 'InvalidAuthorizationFromOwner', 'InvalidAuthorizationFromCddProvider', 'NotCddProviderAttestation', 'AuthorizationsNotForSameDids', 'DidMustAlreadyExist', 'CurrentIdentityCannotBeForwarded', 'AuthorizationExpired', 'TargetHasNoCdd', 'AuthorizationHasBeenRevoked', 'InvalidAuthorizationSignature', 'KeyNotAllowed', 'NotPrimaryKey', 'DidDoesNotExist', 'DidAlreadyExists', 'SecondaryKeysContainPrimaryKey', 'FailedToChargeFee', 'NotASigner', 'CannotDecodeSignerAccountId', 'MultiSigHasBalance', 'ConfidentialScopeClaimNotAllowed', 'InvalidScopeClaim', 'ClaimVariantNotAllowed', 'TargetHasNonZeroBalanceAtScopeId', 'CDDIdNotUniqueForIdentity', 'InvalidCDDId', 'ClaimAndProofVersionsDoNotMatch', 'AccountKeyIsBeingUsed', 'CustomScopeTooLong', 'CustomClaimTypeAlreadyExists', 'CustomClaimTypeDoesNotExist', 'ClaimDoesNotExist', 'IsChildIdentity', 'NoParentIdentity', 'NotParentOrChildIdentity', 'DuplicateKey', 'ExceptNotAllowedForExtrinsics']
  },
  /**
   * Lookup383: polymesh_common_utilities::traits::group::InactiveMember<Moment>
   **/
  PolymeshCommonUtilitiesGroupInactiveMember: {
    id: 'PolymeshPrimitivesIdentityId',
    deactivatedAt: 'u64',
    expiry: 'Option<u64>'
  },
  /**
   * Lookup384: pallet_group::Call<T, I>
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
   * Lookup385: pallet_group::Error<T, I>
   **/
  PalletGroupError: {
    _enum: ['OnlyPrimaryKeyAllowed', 'DuplicateMember', 'NoSuchMember', 'LastMemberCannotQuit', 'MissingCurrentIdentity', 'ActiveMembersLimitExceeded', 'ActiveMembersLimitOverflow']
  },
  /**
   * Lookup387: pallet_committee::Call<T, I>
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
   * Lookup393: pallet_multisig::Call<T>
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
        proposalWeight: 'SpWeightsWeightV2Weight',
      },
      change_sigs_required_via_creator: {
        multisigAccount: 'AccountId32',
        signaturesRequired: 'u64',
      },
      remove_creator_controls: {
        multisigAccount: 'AccountId32'
      }
    }
  },
  /**
   * Lookup394: pallet_bridge::Call<T>
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
   * Lookup398: pallet_staking::Call<T>
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
        newIntendedCount: 'u32',
      },
      chill_from_governance: {
        identity: 'PolymeshPrimitivesIdentityId',
        stashKeys: 'Vec<AccountId32>'
      }
    }
  },
  /**
   * Lookup399: pallet_staking::RewardDestination<sp_core::crypto::AccountId32>
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
   * Lookup400: pallet_staking::ValidatorPrefs
   **/
  PalletStakingValidatorPrefs: {
    commission: 'Compact<Perbill>',
    blocked: 'bool'
  },
  /**
   * Lookup406: pallet_staking::CompactAssignments
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
   * Lookup457: sp_npos_elections::ElectionScore
   **/
  SpNposElectionsElectionScore: {
    minimalStake: 'u128',
    sumStake: 'u128',
    sumStakeSquared: 'u128'
  },
  /**
   * Lookup458: pallet_staking::ElectionSize
   **/
  PalletStakingElectionSize: {
    validators: 'Compact<u16>',
    nominators: 'Compact<u32>'
  },
  /**
   * Lookup459: pallet_session::pallet::Call<T>
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
   * Lookup460: polymesh_runtime_develop::runtime::SessionKeys
   **/
  PolymeshRuntimeDevelopRuntimeSessionKeys: {
    grandpa: 'SpConsensusGrandpaAppPublic',
    babe: 'SpConsensusBabeAppPublic',
    imOnline: 'PalletImOnlineSr25519AppSr25519Public',
    authorityDiscovery: 'SpAuthorityDiscoveryAppPublic'
  },
  /**
   * Lookup461: sp_authority_discovery::app::Public
   **/
  SpAuthorityDiscoveryAppPublic: 'SpCoreSr25519Public',
  /**
   * Lookup462: pallet_grandpa::pallet::Call<T>
   **/
  PalletGrandpaCall: {
    _enum: {
      report_equivocation: {
        equivocationProof: 'SpConsensusGrandpaEquivocationProof',
        keyOwnerProof: 'SpSessionMembershipProof',
      },
      report_equivocation_unsigned: {
        equivocationProof: 'SpConsensusGrandpaEquivocationProof',
        keyOwnerProof: 'SpSessionMembershipProof',
      },
      note_stalled: {
        delay: 'u32',
        bestFinalizedBlockNumber: 'u32'
      }
    }
  },
  /**
   * Lookup463: sp_consensus_grandpa::EquivocationProof<primitive_types::H256, N>
   **/
  SpConsensusGrandpaEquivocationProof: {
    setId: 'u64',
    equivocation: 'SpConsensusGrandpaEquivocation'
  },
  /**
   * Lookup464: sp_consensus_grandpa::Equivocation<primitive_types::H256, N>
   **/
  SpConsensusGrandpaEquivocation: {
    _enum: {
      Prevote: 'FinalityGrandpaEquivocationPrevote',
      Precommit: 'FinalityGrandpaEquivocationPrecommit'
    }
  },
  /**
   * Lookup465: finality_grandpa::Equivocation<sp_consensus_grandpa::app::Public, finality_grandpa::Prevote<primitive_types::H256, N>, sp_consensus_grandpa::app::Signature>
   **/
  FinalityGrandpaEquivocationPrevote: {
    roundNumber: 'u64',
    identity: 'SpConsensusGrandpaAppPublic',
    first: '(FinalityGrandpaPrevote,SpConsensusGrandpaAppSignature)',
    second: '(FinalityGrandpaPrevote,SpConsensusGrandpaAppSignature)'
  },
  /**
   * Lookup466: finality_grandpa::Prevote<primitive_types::H256, N>
   **/
  FinalityGrandpaPrevote: {
    targetHash: 'H256',
    targetNumber: 'u32'
  },
  /**
   * Lookup467: sp_consensus_grandpa::app::Signature
   **/
  SpConsensusGrandpaAppSignature: 'SpCoreEd25519Signature',
  /**
   * Lookup468: sp_core::ed25519::Signature
   **/
  SpCoreEd25519Signature: '[u8;64]',
  /**
   * Lookup470: finality_grandpa::Equivocation<sp_consensus_grandpa::app::Public, finality_grandpa::Precommit<primitive_types::H256, N>, sp_consensus_grandpa::app::Signature>
   **/
  FinalityGrandpaEquivocationPrecommit: {
    roundNumber: 'u64',
    identity: 'SpConsensusGrandpaAppPublic',
    first: '(FinalityGrandpaPrecommit,SpConsensusGrandpaAppSignature)',
    second: '(FinalityGrandpaPrecommit,SpConsensusGrandpaAppSignature)'
  },
  /**
   * Lookup471: finality_grandpa::Precommit<primitive_types::H256, N>
   **/
  FinalityGrandpaPrecommit: {
    targetHash: 'H256',
    targetNumber: 'u32'
  },
  /**
   * Lookup473: pallet_im_online::pallet::Call<T>
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
   * Lookup474: pallet_im_online::Heartbeat<BlockNumber>
   **/
  PalletImOnlineHeartbeat: {
    blockNumber: 'u32',
    networkState: 'SpCoreOffchainOpaqueNetworkState',
    sessionIndex: 'u32',
    authorityIndex: 'u32',
    validatorsLen: 'u32'
  },
  /**
   * Lookup475: sp_core::offchain::OpaqueNetworkState
   **/
  SpCoreOffchainOpaqueNetworkState: {
    peerId: 'Bytes',
    externalAddresses: 'Vec<Bytes>'
  },
  /**
   * Lookup479: pallet_im_online::sr25519::app_sr25519::Signature
   **/
  PalletImOnlineSr25519AppSr25519Signature: 'SpCoreSr25519Signature',
  /**
   * Lookup480: sp_core::sr25519::Signature
   **/
  SpCoreSr25519Signature: '[u8;64]',
  /**
   * Lookup481: pallet_sudo::Call<T>
   **/
  PalletSudoCall: {
    _enum: {
      sudo: {
        call: 'Call',
      },
      sudo_unchecked_weight: {
        call: 'Call',
        weight: 'SpWeightsWeightV2Weight',
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
   * Lookup482: pallet_asset::Call<T>
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
        portfolioKind: 'PolymeshPrimitivesIdentityIdPortfolioKind',
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
        spec: 'PolymeshPrimitivesAssetMetadataAssetMetadataSpec',
      },
      redeem_from_portfolio: {
        ticker: 'PolymeshPrimitivesTicker',
        value: 'u128',
        portfolio: 'PolymeshPrimitivesIdentityIdPortfolioKind',
      },
      update_asset_type: {
        ticker: 'PolymeshPrimitivesTicker',
        assetType: 'PolymeshPrimitivesAssetAssetType',
      },
      remove_local_metadata_key: {
        ticker: 'PolymeshPrimitivesTicker',
        localKey: 'u64',
      },
      remove_metadata_value: {
        ticker: 'PolymeshPrimitivesTicker',
        metadataKey: 'PolymeshPrimitivesAssetMetadataAssetMetadataKey',
      },
      exempt_ticker_affirmation: {
        ticker: 'PolymeshPrimitivesTicker',
      },
      remove_ticker_affirmation_exemption: {
        ticker: 'PolymeshPrimitivesTicker',
      },
      pre_approve_ticker: {
        ticker: 'PolymeshPrimitivesTicker',
      },
      remove_ticker_pre_approval: {
        ticker: 'PolymeshPrimitivesTicker'
      }
    }
  },
  /**
   * Lookup484: pallet_corporate_actions::distribution::Call<T>
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
   * Lookup486: pallet_asset::checkpoint::Call<T>
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
   * Lookup487: pallet_asset::checkpoint::ScheduleSpec
   **/
  PalletAssetCheckpointScheduleSpec: {
    start: 'Option<u64>',
    period: 'PolymeshPrimitivesCalendarCalendarPeriod',
    remaining: 'u32'
  },
  /**
   * Lookup488: pallet_compliance_manager::Call<T>
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
   * Lookup489: pallet_corporate_actions::Call<T>
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
   * Lookup491: pallet_corporate_actions::RecordDateSpec
   **/
  PalletCorporateActionsRecordDateSpec: {
    _enum: {
      Scheduled: 'u64',
      ExistingSchedule: 'u64',
      Existing: 'u64'
    }
  },
  /**
   * Lookup494: pallet_corporate_actions::InitiateCorporateActionArgs
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
   * Lookup495: pallet_corporate_actions::ballot::Call<T>
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
   * Lookup496: pallet_pips::Call<T>
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
   * Lookup499: pallet_pips::SnapshotResult
   **/
  PalletPipsSnapshotResult: {
    _enum: ['Approve', 'Reject', 'Skip']
  },
  /**
   * Lookup500: pallet_portfolio::Call<T>
   **/
  PalletPortfolioCall: {
    _enum: {
      create_portfolio: {
        name: 'Bytes',
      },
      delete_portfolio: {
        num: 'u64',
      },
      rename_portfolio: {
        num: 'u64',
        toName: 'Bytes',
      },
      quit_portfolio_custody: {
        pid: 'PolymeshPrimitivesIdentityIdPortfolioId',
      },
      accept_portfolio_custody: {
        authId: 'u64',
      },
      move_portfolio_funds: {
        from: 'PolymeshPrimitivesIdentityIdPortfolioId',
        to: 'PolymeshPrimitivesIdentityIdPortfolioId',
        funds: 'Vec<PolymeshPrimitivesPortfolioFund>',
      },
      pre_approve_portfolio: {
        ticker: 'PolymeshPrimitivesTicker',
        portfolioId: 'PolymeshPrimitivesIdentityIdPortfolioId',
      },
      remove_portfolio_pre_approval: {
        ticker: 'PolymeshPrimitivesTicker',
        portfolioId: 'PolymeshPrimitivesIdentityIdPortfolioId'
      }
    }
  },
  /**
   * Lookup502: polymesh_primitives::portfolio::Fund
   **/
  PolymeshPrimitivesPortfolioFund: {
    description: 'PolymeshPrimitivesPortfolioFundDescription',
    memo: 'Option<PolymeshPrimitivesMemo>'
  },
  /**
   * Lookup503: pallet_protocol_fee::Call<T>
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
   * Lookup504: polymesh_common_utilities::protocol_fee::ProtocolOp
   **/
  PolymeshCommonUtilitiesProtocolFeeProtocolOp: {
    _enum: ['AssetRegisterTicker', 'AssetIssue', 'AssetAddDocuments', 'AssetCreateAsset', 'CheckpointCreateSchedule', 'ComplianceManagerAddComplianceRequirement', 'IdentityCddRegisterDid', 'IdentityAddClaim', 'IdentityAddSecondaryKeysWithAuthorization', 'PipsPropose', 'ContractsPutCode', 'CorporateBallotAttachBallot', 'CapitalDistributionDistribute', 'NFTCreateCollection', 'NFTMint']
  },
  /**
   * Lookup505: pallet_scheduler::pallet::Call<T>
   **/
  PalletSchedulerCall: {
    _enum: {
      schedule: {
        when: 'u32',
        maybePeriodic: 'Option<(u32,u32)>',
        priority: 'u8',
        call: 'Call',
      },
      cancel: {
        when: 'u32',
        index: 'u32',
      },
      schedule_named: {
        id: '[u8;32]',
        when: 'u32',
        maybePeriodic: 'Option<(u32,u32)>',
        priority: 'u8',
        call: 'Call',
      },
      cancel_named: {
        id: '[u8;32]',
      },
      schedule_after: {
        after: 'u32',
        maybePeriodic: 'Option<(u32,u32)>',
        priority: 'u8',
        call: 'Call',
      },
      schedule_named_after: {
        id: '[u8;32]',
        after: 'u32',
        maybePeriodic: 'Option<(u32,u32)>',
        priority: 'u8',
        call: 'Call'
      }
    }
  },
  /**
   * Lookup507: pallet_settlement::Call<T>
   **/
  PalletSettlementCall: {
    _enum: {
      create_venue: {
        details: 'Bytes',
        signers: 'Vec<AccountId32>',
        typ: 'PolymeshPrimitivesSettlementVenueType',
      },
      update_venue_details: {
        id: 'u64',
        details: 'Bytes',
      },
      update_venue_type: {
        id: 'u64',
        typ: 'PolymeshPrimitivesSettlementVenueType',
      },
      placeholder_add_instruction: 'Null',
      placeholder_add_and_affirm_instruction: 'Null',
      placeholder_affirm_instruction: 'Null',
      placeholder_withdraw_affirmation: 'Null',
      placeholder_reject_instruction: 'Null',
      affirm_with_receipts: {
        id: 'u64',
        receiptDetails: 'Vec<PolymeshPrimitivesSettlementReceiptDetails>',
        portfolios: 'Vec<PolymeshPrimitivesIdentityIdPortfolioId>',
      },
      placeholder_claim_receipt: 'Null',
      placeholder_unclaim_receipt: 'Null',
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
        id: 'u64',
      },
      update_venue_signers: {
        id: 'u64',
        signers: 'Vec<AccountId32>',
        addSigners: 'bool',
      },
      placeholder_add_instruction_with_memo: 'Null',
      placeholder_add_and_affirm_instruction_with_memo: 'Null',
      execute_manual_instruction: {
        id: 'u64',
        portfolio: 'Option<PolymeshPrimitivesIdentityIdPortfolioId>',
        fungibleTransfers: 'u32',
        nftsTransfers: 'u32',
        offchainTransfers: 'u32',
        weightLimit: 'Option<SpWeightsWeightV2Weight>',
      },
      add_instruction: {
        venueId: 'u64',
        settlementType: 'PolymeshPrimitivesSettlementSettlementType',
        tradeDate: 'Option<u64>',
        valueDate: 'Option<u64>',
        legs: 'Vec<PolymeshPrimitivesSettlementLeg>',
        instructionMemo: 'Option<PolymeshPrimitivesMemo>',
      },
      add_and_affirm_instruction: {
        venueId: 'u64',
        settlementType: 'PolymeshPrimitivesSettlementSettlementType',
        tradeDate: 'Option<u64>',
        valueDate: 'Option<u64>',
        legs: 'Vec<PolymeshPrimitivesSettlementLeg>',
        portfolios: 'Vec<PolymeshPrimitivesIdentityIdPortfolioId>',
        instructionMemo: 'Option<PolymeshPrimitivesMemo>',
      },
      affirm_instruction: {
        id: 'u64',
        portfolios: 'Vec<PolymeshPrimitivesIdentityIdPortfolioId>',
      },
      withdraw_affirmation: {
        id: 'u64',
        portfolios: 'Vec<PolymeshPrimitivesIdentityIdPortfolioId>',
      },
      reject_instruction: {
        id: 'u64',
        portfolio: 'PolymeshPrimitivesIdentityIdPortfolioId',
      },
      execute_scheduled_instruction_v2: {
        id: 'u64',
        fungibleTransfers: 'u32',
        nftsTransfers: 'u32',
      },
      execute_scheduled_instruction_v3: {
        id: 'u64',
        weightLimit: 'SpWeightsWeightV2Weight'
      }
    }
  },
  /**
   * Lookup509: polymesh_primitives::settlement::ReceiptDetails<sp_core::crypto::AccountId32, sp_runtime::MultiSignature>
   **/
  PolymeshPrimitivesSettlementReceiptDetails: {
    uid: 'u64',
    instructionId: 'u64',
    legId: 'u64',
    signer: 'AccountId32',
    signature: 'SpRuntimeMultiSignature',
    metadata: 'Bytes'
  },
  /**
   * Lookup510: sp_runtime::MultiSignature
   **/
  SpRuntimeMultiSignature: {
    _enum: {
      Ed25519: 'SpCoreEd25519Signature',
      Sr25519: 'SpCoreSr25519Signature',
      Ecdsa: 'SpCoreEcdsaSignature'
    }
  },
  /**
   * Lookup511: sp_core::ecdsa::Signature
   **/
  SpCoreEcdsaSignature: '[u8;65]',
  /**
   * Lookup513: pallet_statistics::Call<T>
   **/
  PalletStatisticsCall: {
    _enum: {
      set_active_asset_stats: {
        asset: 'PolymeshPrimitivesStatisticsAssetScope',
        statTypes: 'BTreeSet<PolymeshPrimitivesStatisticsStatType>',
      },
      batch_update_asset_stats: {
        asset: 'PolymeshPrimitivesStatisticsAssetScope',
        statType: 'PolymeshPrimitivesStatisticsStatType',
        values: 'BTreeSet<PolymeshPrimitivesStatisticsStatUpdate>',
      },
      set_asset_transfer_compliance: {
        asset: 'PolymeshPrimitivesStatisticsAssetScope',
        transferConditions: 'BTreeSet<PolymeshPrimitivesTransferComplianceTransferCondition>',
      },
      set_entities_exempt: {
        isExempt: 'bool',
        exemptKey: 'PolymeshPrimitivesTransferComplianceTransferConditionExemptKey',
        entities: 'BTreeSet<PolymeshPrimitivesIdentityId>'
      }
    }
  },
  /**
   * Lookup518: pallet_sto::Call<T>
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
        receipt: 'Option<PolymeshPrimitivesSettlementReceiptDetails>',
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
   * Lookup520: pallet_sto::PriceTier
   **/
  PalletStoPriceTier: {
    total: 'u128',
    price: 'u128'
  },
  /**
   * Lookup522: pallet_treasury::Call<T>
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
   * Lookup524: polymesh_primitives::Beneficiary<Balance>
   **/
  PolymeshPrimitivesBeneficiary: {
    id: 'PolymeshPrimitivesIdentityId',
    amount: 'u128'
  },
  /**
   * Lookup525: pallet_utility::pallet::Call<T>
   **/
  PalletUtilityCall: {
    _enum: {
      batch: {
        calls: 'Vec<Call>',
      },
      relay_tx: {
        target: 'AccountId32',
        signature: 'SpRuntimeMultiSignature',
        call: 'PalletUtilityUniqueCall',
      },
      batch_all: {
        calls: 'Vec<Call>',
      },
      dispatch_as: {
        asOrigin: 'PolymeshRuntimeDevelopRuntimeOriginCaller',
        call: 'Call',
      },
      force_batch: {
        calls: 'Vec<Call>',
      },
      with_weight: {
        call: 'Call',
        weight: 'SpWeightsWeightV2Weight',
      },
      batch_old: {
        calls: 'Vec<Call>',
      },
      batch_atomic: {
        calls: 'Vec<Call>',
      },
      batch_optimistic: {
        calls: 'Vec<Call>'
      }
    }
  },
  /**
   * Lookup527: pallet_utility::UniqueCall<polymesh_runtime_develop::runtime::RuntimeCall>
   **/
  PalletUtilityUniqueCall: {
    nonce: 'u64',
    call: 'Call'
  },
  /**
   * Lookup528: polymesh_runtime_develop::runtime::OriginCaller
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
   * Lookup529: frame_support::dispatch::RawOrigin<sp_core::crypto::AccountId32>
   **/
  FrameSupportDispatchRawOrigin: {
    _enum: {
      Root: 'Null',
      Signed: 'AccountId32',
      None: 'Null'
    }
  },
  /**
   * Lookup530: pallet_committee::RawOrigin<sp_core::crypto::AccountId32, pallet_committee::Instance1>
   **/
  PalletCommitteeRawOriginInstance1: {
    _enum: ['Endorsed']
  },
  /**
   * Lookup531: pallet_committee::RawOrigin<sp_core::crypto::AccountId32, pallet_committee::Instance3>
   **/
  PalletCommitteeRawOriginInstance3: {
    _enum: ['Endorsed']
  },
  /**
   * Lookup532: pallet_committee::RawOrigin<sp_core::crypto::AccountId32, pallet_committee::Instance4>
   **/
  PalletCommitteeRawOriginInstance4: {
    _enum: ['Endorsed']
  },
  /**
   * Lookup533: sp_core::Void
   **/
  SpCoreVoid: 'Null',
  /**
   * Lookup534: pallet_base::Call<T>
   **/
  PalletBaseCall: 'Null',
  /**
   * Lookup535: pallet_external_agents::Call<T>
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
        target: 'PolymeshPrimitivesIdentityId',
        expiry: 'Option<u64>',
      },
      create_and_change_custom_group: {
        ticker: 'PolymeshPrimitivesTicker',
        perms: 'PolymeshPrimitivesSubsetSubsetRestrictionPalletPermissions',
        agent: 'PolymeshPrimitivesIdentityId'
      }
    }
  },
  /**
   * Lookup536: pallet_relayer::Call<T>
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
   * Lookup537: pallet_rewards::Call<T>
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
   * Lookup538: pallet_rewards::ItnRewardStatus
   **/
  PalletRewardsItnRewardStatus: {
    _enum: {
      Unclaimed: 'u128',
      Claimed: 'Null'
    }
  },
  /**
   * Lookup539: pallet_contracts::pallet::Call<T>
   **/
  PalletContractsCall: {
    _enum: {
      call_old_weight: {
        dest: 'MultiAddress',
        value: 'Compact<u128>',
        gasLimit: 'Compact<u64>',
        storageDepositLimit: 'Option<Compact<u128>>',
        data: 'Bytes',
      },
      instantiate_with_code_old_weight: {
        value: 'Compact<u128>',
        gasLimit: 'Compact<u64>',
        storageDepositLimit: 'Option<Compact<u128>>',
        code: 'Bytes',
        data: 'Bytes',
        salt: 'Bytes',
      },
      instantiate_old_weight: {
        value: 'Compact<u128>',
        gasLimit: 'Compact<u64>',
        storageDepositLimit: 'Option<Compact<u128>>',
        codeHash: 'H256',
        data: 'Bytes',
        salt: 'Bytes',
      },
      upload_code: {
        code: 'Bytes',
        storageDepositLimit: 'Option<Compact<u128>>',
        determinism: 'PalletContractsWasmDeterminism',
      },
      remove_code: {
        codeHash: 'H256',
      },
      set_code: {
        dest: 'MultiAddress',
        codeHash: 'H256',
      },
      call: {
        dest: 'MultiAddress',
        value: 'Compact<u128>',
        gasLimit: 'SpWeightsWeightV2Weight',
        storageDepositLimit: 'Option<Compact<u128>>',
        data: 'Bytes',
      },
      instantiate_with_code: {
        value: 'Compact<u128>',
        gasLimit: 'SpWeightsWeightV2Weight',
        storageDepositLimit: 'Option<Compact<u128>>',
        code: 'Bytes',
        data: 'Bytes',
        salt: 'Bytes',
      },
      instantiate: {
        value: 'Compact<u128>',
        gasLimit: 'SpWeightsWeightV2Weight',
        storageDepositLimit: 'Option<Compact<u128>>',
        codeHash: 'H256',
        data: 'Bytes',
        salt: 'Bytes'
      }
    }
  },
  /**
   * Lookup543: pallet_contracts::wasm::Determinism
   **/
  PalletContractsWasmDeterminism: {
    _enum: ['Deterministic', 'AllowIndeterminism']
  },
  /**
   * Lookup544: polymesh_contracts::Call<T>
   **/
  PolymeshContractsCall: {
    _enum: {
      instantiate_with_code_perms: {
        endowment: 'u128',
        gasLimit: 'SpWeightsWeightV2Weight',
        storageDepositLimit: 'Option<u128>',
        code: 'Bytes',
        data: 'Bytes',
        salt: 'Bytes',
        perms: 'PolymeshPrimitivesSecondaryKeyPermissions',
      },
      instantiate_with_hash_perms: {
        endowment: 'u128',
        gasLimit: 'SpWeightsWeightV2Weight',
        storageDepositLimit: 'Option<u128>',
        codeHash: 'H256',
        data: 'Bytes',
        salt: 'Bytes',
        perms: 'PolymeshPrimitivesSecondaryKeyPermissions',
      },
      update_call_runtime_whitelist: {
        updates: 'Vec<(PolymeshContractsChainExtensionExtrinsicId,bool)>'
      }
    }
  },
  /**
   * Lookup547: polymesh_contracts::chain_extension::ExtrinsicId
   **/
  PolymeshContractsChainExtensionExtrinsicId: '(u8,u8)',
  /**
   * Lookup548: pallet_preimage::pallet::Call<T>
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
   * Lookup549: pallet_nft::Call<T>
   **/
  PalletNftCall: {
    _enum: {
      create_nft_collection: {
        ticker: 'PolymeshPrimitivesTicker',
        nftType: 'Option<PolymeshPrimitivesAssetNonFungibleType>',
        collectionKeys: 'PolymeshPrimitivesNftNftCollectionKeys',
      },
      issue_nft: {
        ticker: 'PolymeshPrimitivesTicker',
        nftMetadataAttributes: 'Vec<PolymeshPrimitivesNftNftMetadataAttribute>',
        portfolioKind: 'PolymeshPrimitivesIdentityIdPortfolioKind',
      },
      redeem_nft: {
        ticker: 'PolymeshPrimitivesTicker',
        nftId: 'u64',
        portfolioKind: 'PolymeshPrimitivesIdentityIdPortfolioKind'
      }
    }
  },
  /**
   * Lookup551: polymesh_primitives::nft::NFTCollectionKeys
   **/
  PolymeshPrimitivesNftNftCollectionKeys: 'Vec<PolymeshPrimitivesAssetMetadataAssetMetadataKey>',
  /**
   * Lookup554: polymesh_primitives::nft::NFTMetadataAttribute
   **/
  PolymeshPrimitivesNftNftMetadataAttribute: {
    key: 'PolymeshPrimitivesAssetMetadataAssetMetadataKey',
    value: 'Bytes'
  },
  /**
   * Lookup555: pallet_test_utils::Call<T>
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
   * Lookup556: pallet_committee::PolymeshVotes<BlockNumber>
   **/
  PalletCommitteePolymeshVotes: {
    index: 'u32',
    ayes: 'Vec<PolymeshPrimitivesIdentityId>',
    nays: 'Vec<PolymeshPrimitivesIdentityId>',
    expiry: 'PolymeshCommonUtilitiesMaybeBlock'
  },
  /**
   * Lookup558: pallet_committee::Error<T, I>
   **/
  PalletCommitteeError: {
    _enum: ['DuplicateVote', 'NotAMember', 'NoSuchProposal', 'ProposalExpired', 'DuplicateProposal', 'MismatchedVotingIndex', 'InvalidProportion', 'FirstVoteReject', 'ProposalsLimitReached']
  },
  /**
   * Lookup568: polymesh_primitives::multisig::ProposalDetails<T>
   **/
  PolymeshPrimitivesMultisigProposalDetails: {
    approvals: 'u64',
    rejections: 'u64',
    status: 'PolymeshPrimitivesMultisigProposalStatus',
    expiry: 'Option<u64>',
    autoClose: 'bool'
  },
  /**
   * Lookup569: polymesh_primitives::multisig::ProposalStatus
   **/
  PolymeshPrimitivesMultisigProposalStatus: {
    _enum: ['Invalid', 'ActiveOrExpired', 'ExecutionSuccessful', 'ExecutionFailed', 'Rejected']
  },
  /**
   * Lookup571: pallet_multisig::Error<T>
   **/
  PalletMultisigError: {
    _enum: ['CddMissing', 'ProposalMissing', 'DecodingError', 'NoSigners', 'RequiredSignaturesOutOfBounds', 'NotASigner', 'NoSuchMultisig', 'NotEnoughSigners', 'NonceOverflow', 'AlreadyVoted', 'AlreadyASigner', 'FailedToChargeFee', 'IdentityNotCreator', 'ChangeNotAllowed', 'SignerAlreadyLinkedToMultisig', 'SignerAlreadyLinkedToIdentity', 'MultisigNotAllowedToLinkToItself', 'MissingCurrentIdentity', 'NotPrimaryKey', 'ProposalAlreadyRejected', 'ProposalExpired', 'ProposalAlreadyExecuted', 'MultisigMissingIdentity', 'FailedToSchedule', 'TooManySigners', 'CreatorControlsHaveBeenRemoved']
  },
  /**
   * Lookup573: pallet_bridge::BridgeTxDetail<BlockNumber>
   **/
  PalletBridgeBridgeTxDetail: {
    amount: 'u128',
    status: 'PalletBridgeBridgeTxStatus',
    executionBlock: 'u32',
    txHash: 'H256'
  },
  /**
   * Lookup574: pallet_bridge::BridgeTxStatus
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
   * Lookup577: pallet_bridge::Error<T>
   **/
  PalletBridgeError: {
    _enum: ['ControllerNotSet', 'BadCaller', 'BadAdmin', 'NoValidCdd', 'ProposalAlreadyHandled', 'Unauthorized', 'Frozen', 'NotFrozen', 'FrozenTx', 'BridgeLimitReached', 'Overflow', 'DivisionByZero', 'TimelockedTx']
  },
  /**
   * Lookup578: pallet_staking::StakingLedger<sp_core::crypto::AccountId32, Balance>
   **/
  PalletStakingStakingLedger: {
    stash: 'AccountId32',
    total: 'Compact<u128>',
    active: 'Compact<u128>',
    unlocking: 'Vec<PalletStakingUnlockChunk>',
    claimedRewards: 'Vec<u32>'
  },
  /**
   * Lookup580: pallet_staking::UnlockChunk<Balance>
   **/
  PalletStakingUnlockChunk: {
    value: 'Compact<u128>',
    era: 'Compact<u32>'
  },
  /**
   * Lookup581: pallet_staking::Nominations<sp_core::crypto::AccountId32>
   **/
  PalletStakingNominations: {
    targets: 'Vec<AccountId32>',
    submittedIn: 'u32',
    suppressed: 'bool'
  },
  /**
   * Lookup582: pallet_staking::ActiveEraInfo
   **/
  PalletStakingActiveEraInfo: {
    index: 'u32',
    start: 'Option<u64>'
  },
  /**
   * Lookup584: pallet_staking::EraRewardPoints<sp_core::crypto::AccountId32>
   **/
  PalletStakingEraRewardPoints: {
    total: 'u32',
    individual: 'BTreeMap<AccountId32, u32>'
  },
  /**
   * Lookup587: pallet_staking::Forcing
   **/
  PalletStakingForcing: {
    _enum: ['NotForcing', 'ForceNew', 'ForceNone', 'ForceAlways']
  },
  /**
   * Lookup589: pallet_staking::UnappliedSlash<sp_core::crypto::AccountId32, Balance>
   **/
  PalletStakingUnappliedSlash: {
    validator: 'AccountId32',
    own: 'u128',
    others: 'Vec<(AccountId32,u128)>',
    reporters: 'Vec<AccountId32>',
    payout: 'u128'
  },
  /**
   * Lookup593: pallet_staking::slashing::SlashingSpans
   **/
  PalletStakingSlashingSlashingSpans: {
    spanIndex: 'u32',
    lastStart: 'u32',
    lastNonzeroSlash: 'u32',
    prior: 'Vec<u32>'
  },
  /**
   * Lookup594: pallet_staking::slashing::SpanRecord<Balance>
   **/
  PalletStakingSlashingSpanRecord: {
    slashed: 'u128',
    paidOut: 'u128'
  },
  /**
   * Lookup597: pallet_staking::ElectionResult<sp_core::crypto::AccountId32, Balance>
   **/
  PalletStakingElectionResult: {
    electedStashes: 'Vec<AccountId32>',
    exposures: 'Vec<(AccountId32,PalletStakingExposure)>',
    compute: 'PalletStakingElectionCompute'
  },
  /**
   * Lookup598: pallet_staking::ElectionStatus<BlockNumber>
   **/
  PalletStakingElectionStatus: {
    _enum: {
      Closed: 'Null',
      Open: 'u32'
    }
  },
  /**
   * Lookup599: pallet_staking::PermissionedIdentityPrefs
   **/
  PalletStakingPermissionedIdentityPrefs: {
    intendedCount: 'u32',
    runningCount: 'u32'
  },
  /**
   * Lookup600: pallet_staking::Releases
   **/
  PalletStakingReleases: {
    _enum: ['V1_0_0Ancient', 'V2_0_0', 'V3_0_0', 'V4_0_0', 'V5_0_0', 'V6_0_0', 'V6_0_1', 'V7_0_0']
  },
  /**
   * Lookup602: pallet_staking::Error<T>
   **/
  PalletStakingError: {
    _enum: ['NotController', 'NotStash', 'AlreadyBonded', 'AlreadyPaired', 'EmptyTargets', 'InvalidSlashIndex', 'InsufficientValue', 'NoMoreChunks', 'NoUnlockChunk', 'FundedTarget', 'InvalidEraToReward', 'NotSortedAndUnique', 'AlreadyClaimed', 'OffchainElectionEarlySubmission', 'OffchainElectionWeakSubmission', 'SnapshotUnavailable', 'OffchainElectionBogusWinnerCount', 'OffchainElectionBogusWinner', 'OffchainElectionBogusCompact', 'OffchainElectionBogusNominator', 'OffchainElectionBogusNomination', 'OffchainElectionSlashedNomination', 'OffchainElectionBogusSelfVote', 'OffchainElectionBogusEdge', 'OffchainElectionBogusScore', 'OffchainElectionBogusElectionSize', 'CallNotAllowed', 'IncorrectSlashingSpans', 'AlreadyExists', 'NotExists', 'NoChange', 'InvalidValidatorIdentity', 'InvalidValidatorCommission', 'StashIdentityDoesNotExist', 'StashIdentityNotPermissioned', 'StashIdentityNotCDDed', 'HitIntendedValidatorCount', 'IntendedCountIsExceedingConsensusLimit', 'BondTooSmall', 'BadState', 'TooManyTargets', 'BadTarget', 'InvalidValidatorUnbondAmount']
  },
  /**
   * Lookup603: sp_staking::offence::OffenceDetails<sp_core::crypto::AccountId32, Offender>
   **/
  SpStakingOffenceOffenceDetails: {
    offender: '(AccountId32,PalletStakingExposure)',
    reporters: 'Vec<AccountId32>'
  },
  /**
   * Lookup608: sp_core::crypto::KeyTypeId
   **/
  SpCoreCryptoKeyTypeId: '[u8;4]',
  /**
   * Lookup609: pallet_session::pallet::Error<T>
   **/
  PalletSessionError: {
    _enum: ['InvalidProof', 'NoAssociatedValidatorId', 'DuplicatedKey', 'NoKeys', 'NoAccount']
  },
  /**
   * Lookup610: pallet_grandpa::StoredState<N>
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
   * Lookup611: pallet_grandpa::StoredPendingChange<N, Limit>
   **/
  PalletGrandpaStoredPendingChange: {
    scheduledAt: 'u32',
    delay: 'u32',
    nextAuthorities: 'Vec<(SpConsensusGrandpaAppPublic,u64)>',
    forced: 'Option<u32>'
  },
  /**
   * Lookup613: pallet_grandpa::pallet::Error<T>
   **/
  PalletGrandpaError: {
    _enum: ['PauseFailed', 'ResumeFailed', 'ChangePending', 'TooSoon', 'InvalidKeyOwnershipProof', 'InvalidEquivocationProof', 'DuplicateOffenceReport']
  },
  /**
   * Lookup617: pallet_im_online::BoundedOpaqueNetworkState<PeerIdEncodingLimit, MultiAddrEncodingLimit, AddressesLimit>
   **/
  PalletImOnlineBoundedOpaqueNetworkState: {
    peerId: 'Bytes',
    externalAddresses: 'Vec<Bytes>'
  },
  /**
   * Lookup621: pallet_im_online::pallet::Error<T>
   **/
  PalletImOnlineError: {
    _enum: ['InvalidKey', 'DuplicatedHeartbeat']
  },
  /**
   * Lookup623: pallet_sudo::Error<T>
   **/
  PalletSudoError: {
    _enum: ['RequireSudo']
  },
  /**
   * Lookup624: pallet_asset::TickerRegistration<U>
   **/
  PalletAssetTickerRegistration: {
    owner: 'PolymeshPrimitivesIdentityId',
    expiry: 'Option<u64>'
  },
  /**
   * Lookup625: pallet_asset::TickerRegistrationConfig<U>
   **/
  PalletAssetTickerRegistrationConfig: {
    maxTickerLength: 'u8',
    registrationLength: 'Option<u64>'
  },
  /**
   * Lookup626: pallet_asset::SecurityToken
   **/
  PalletAssetSecurityToken: {
    totalSupply: 'u128',
    ownerDid: 'PolymeshPrimitivesIdentityId',
    divisible: 'bool',
    assetType: 'PolymeshPrimitivesAssetAssetType'
  },
  /**
   * Lookup630: pallet_asset::AssetOwnershipRelation
   **/
  PalletAssetAssetOwnershipRelation: {
    _enum: ['NotOwned', 'TickerOwned', 'AssetOwned']
  },
  /**
   * Lookup637: pallet_asset::Error<T>
   **/
  PalletAssetError: {
    _enum: ['Unauthorized', 'AssetAlreadyCreated', 'TickerTooLong', 'TickerNotAlphanumeric', 'TickerAlreadyRegistered', 'TotalSupplyAboveLimit', 'NoSuchAsset', 'AlreadyFrozen', 'NotAnOwner', 'BalanceOverflow', 'TotalSupplyOverflow', 'InvalidGranularity', 'NotFrozen', 'InvalidTransfer', 'InsufficientBalance', 'AssetAlreadyDivisible', 'InvalidEthereumSignature', 'TickerRegistrationExpired', 'SenderSameAsReceiver', 'NoSuchDoc', 'MaxLengthOfAssetNameExceeded', 'FundingRoundNameMaxLengthExceeded', 'InvalidAssetIdentifier', 'InvestorUniquenessClaimNotAllowed', 'InvalidCustomAssetTypeId', 'AssetMetadataNameMaxLengthExceeded', 'AssetMetadataValueMaxLengthExceeded', 'AssetMetadataTypeDefMaxLengthExceeded', 'AssetMetadataKeyIsMissing', 'AssetMetadataValueIsLocked', 'AssetMetadataLocalKeyAlreadyExists', 'AssetMetadataGlobalKeyAlreadyExists', 'TickerFirstByteNotValid', 'UnexpectedNonFungibleToken', 'IncompatibleAssetTypeUpdate', 'AssetMetadataKeyBelongsToNFTCollection', 'AssetMetadataValueIsEmpty', 'InvestorUniquenessNotAllowed']
  },
  /**
   * Lookup640: pallet_corporate_actions::distribution::Error<T>
   **/
  PalletCorporateActionsDistributionError: {
    _enum: ['CANotBenefit', 'AlreadyExists', 'ExpiryBeforePayment', 'HolderAlreadyPaid', 'NoSuchDistribution', 'CannotClaimBeforeStart', 'CannotClaimAfterExpiry', 'BalancePerShareProductOverflowed', 'NotDistributionCreator', 'AlreadyReclaimed', 'NotExpired', 'DistributionStarted', 'InsufficientRemainingAmount', 'DistributionAmountIsZero', 'DistributionPerShareIsZero']
  },
  /**
   * Lookup647: pallet_asset::checkpoint::Error<T>
   **/
  PalletAssetCheckpointError: {
    _enum: ['NoSuchSchedule', 'ScheduleNotRemovable', 'FailedToComputeNextCheckpoint', 'ScheduleDurationTooShort', 'SchedulesTooComplex']
  },
  /**
   * Lookup648: polymesh_primitives::compliance_manager::AssetCompliance
   **/
  PolymeshPrimitivesComplianceManagerAssetCompliance: {
    paused: 'bool',
    requirements: 'Vec<PolymeshPrimitivesComplianceManagerComplianceRequirement>'
  },
  /**
   * Lookup650: pallet_compliance_manager::Error<T>
   **/
  PalletComplianceManagerError: {
    _enum: ['Unauthorized', 'DidNotExist', 'InvalidComplianceRequirementId', 'IncorrectOperationOnTrustedIssuer', 'DuplicateComplianceRequirements', 'ComplianceRequirementTooComplex', 'WeightLimitExceeded']
  },
  /**
   * Lookup653: pallet_corporate_actions::Error<T>
   **/
  PalletCorporateActionsError: {
    _enum: ['DetailsTooLong', 'DuplicateDidTax', 'TooManyDidTaxes', 'TooManyTargetIds', 'NoSuchCheckpointId', 'NoSuchCA', 'NoRecordDate', 'RecordDateAfterStart', 'DeclDateAfterRecordDate', 'DeclDateInFuture', 'NotTargetedByCA']
  },
  /**
   * Lookup655: pallet_corporate_actions::ballot::Error<T>
   **/
  PalletCorporateActionsBallotError: {
    _enum: ['CANotNotice', 'AlreadyExists', 'NoSuchBallot', 'StartAfterEnd', 'NowAfterEnd', 'NumberOfChoicesOverflow', 'VotingAlreadyStarted', 'VotingNotStarted', 'VotingAlreadyEnded', 'WrongVoteCount', 'InsufficientVotes', 'NoSuchRCVFallback', 'RCVSelfCycle', 'RCVNotAllowed']
  },
  /**
   * Lookup656: pallet_permissions::Error<T>
   **/
  PalletPermissionsError: {
    _enum: ['UnauthorizedCaller']
  },
  /**
   * Lookup657: pallet_pips::PipsMetadata<BlockNumber>
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
   * Lookup659: pallet_pips::DepositInfo<sp_core::crypto::AccountId32>
   **/
  PalletPipsDepositInfo: {
    owner: 'AccountId32',
    amount: 'u128'
  },
  /**
   * Lookup660: pallet_pips::Pip<polymesh_runtime_develop::runtime::RuntimeCall, sp_core::crypto::AccountId32>
   **/
  PalletPipsPip: {
    id: 'u32',
    proposal: 'Call',
    proposer: 'PalletPipsProposer'
  },
  /**
   * Lookup661: pallet_pips::VotingResult
   **/
  PalletPipsVotingResult: {
    ayesCount: 'u32',
    ayesStake: 'u128',
    naysCount: 'u32',
    naysStake: 'u128'
  },
  /**
   * Lookup662: pallet_pips::Vote
   **/
  PalletPipsVote: '(bool,u128)',
  /**
   * Lookup663: pallet_pips::SnapshotMetadata<BlockNumber, sp_core::crypto::AccountId32>
   **/
  PalletPipsSnapshotMetadata: {
    createdAt: 'u32',
    madeBy: 'AccountId32',
    id: 'u32'
  },
  /**
   * Lookup665: pallet_pips::Error<T>
   **/
  PalletPipsError: {
    _enum: ['RescheduleNotByReleaseCoordinator', 'NotFromCommunity', 'NotByCommittee', 'TooManyActivePips', 'IncorrectDeposit', 'InsufficientDeposit', 'NoSuchProposal', 'NotACommitteeMember', 'InvalidFutureBlockNumber', 'NumberOfVotesExceeded', 'StakeAmountOfVotesExceeded', 'MissingCurrentIdentity', 'IncorrectProposalState', 'CannotSkipPip', 'SnapshotResultTooLarge', 'SnapshotIdMismatch', 'ScheduledProposalDoesntExist', 'ProposalNotInScheduledState']
  },
  /**
   * Lookup673: pallet_portfolio::Error<T>
   **/
  PalletPortfolioError: {
    _enum: ['PortfolioDoesNotExist', 'InsufficientPortfolioBalance', 'DestinationIsSamePortfolio', 'PortfolioNameAlreadyInUse', 'SecondaryKeyNotAuthorizedForPortfolio', 'UnauthorizedCustodian', 'InsufficientTokensLocked', 'PortfolioNotEmpty', 'DifferentIdentityPortfolios', 'NoDuplicateAssetsAllowed', 'NFTNotFoundInPortfolio', 'NFTAlreadyLocked', 'NFTNotLocked', 'InvalidTransferNFTNotOwned', 'InvalidTransferNFTIsLocked', 'EmptyTransfer']
  },
  /**
   * Lookup674: pallet_protocol_fee::Error<T>
   **/
  PalletProtocolFeeError: {
    _enum: ['InsufficientAccountBalance', 'UnHandledImbalances', 'InsufficientSubsidyBalance']
  },
  /**
   * Lookup677: pallet_scheduler::Scheduled<Name, frame_support::traits::preimages::Bounded<polymesh_runtime_develop::runtime::RuntimeCall>, BlockNumber, polymesh_runtime_develop::runtime::OriginCaller, sp_core::crypto::AccountId32>
   **/
  PalletSchedulerScheduled: {
    maybeId: 'Option<[u8;32]>',
    priority: 'u8',
    call: 'FrameSupportPreimagesBounded',
    maybePeriodic: 'Option<(u32,u32)>',
    origin: 'PolymeshRuntimeDevelopRuntimeOriginCaller'
  },
  /**
   * Lookup678: frame_support::traits::preimages::Bounded<polymesh_runtime_develop::runtime::RuntimeCall>
   **/
  FrameSupportPreimagesBounded: {
    _enum: {
      Legacy: {
        _alias: {
          hash_: 'hash',
        },
        hash_: 'H256',
      },
      Inline: 'Bytes',
      Lookup: {
        _alias: {
          hash_: 'hash',
        },
        hash_: 'H256',
        len: 'u32'
      }
    }
  },
  /**
   * Lookup681: pallet_scheduler::pallet::Error<T>
   **/
  PalletSchedulerError: {
    _enum: ['FailedToSchedule', 'NotFound', 'TargetBlockNumberInPast', 'RescheduleNoChange', 'Named']
  },
  /**
   * Lookup682: polymesh_primitives::settlement::Venue
   **/
  PolymeshPrimitivesSettlementVenue: {
    creator: 'PolymeshPrimitivesIdentityId',
    venueType: 'PolymeshPrimitivesSettlementVenueType'
  },
  /**
   * Lookup685: polymesh_primitives::settlement::Instruction<Moment, BlockNumber>
   **/
  PolymeshPrimitivesSettlementInstruction: {
    instructionId: 'u64',
    venueId: 'u64',
    settlementType: 'PolymeshPrimitivesSettlementSettlementType',
    createdAt: 'Option<u64>',
    tradeDate: 'Option<u64>',
    valueDate: 'Option<u64>'
  },
  /**
   * Lookup687: polymesh_primitives::settlement::LegStatus<sp_core::crypto::AccountId32>
   **/
  PolymeshPrimitivesSettlementLegStatus: {
    _enum: {
      PendingTokenLock: 'Null',
      ExecutionPending: 'Null',
      ExecutionToBeSkipped: '(AccountId32,u64)'
    }
  },
  /**
   * Lookup689: polymesh_primitives::settlement::AffirmationStatus
   **/
  PolymeshPrimitivesSettlementAffirmationStatus: {
    _enum: ['Unknown', 'Pending', 'Affirmed']
  },
  /**
   * Lookup693: polymesh_primitives::settlement::InstructionStatus<BlockNumber>
   **/
  PolymeshPrimitivesSettlementInstructionStatus: {
    _enum: {
      Unknown: 'Null',
      Pending: 'Null',
      Failed: 'Null',
      Success: 'u32',
      Rejected: 'u32'
    }
  },
  /**
   * Lookup694: pallet_settlement::Error<T>
   **/
  PalletSettlementError: {
    _enum: ['InvalidVenue', 'Unauthorized', 'InstructionNotAffirmed', 'InstructionNotPending', 'InstructionNotFailed', 'UnauthorizedSigner', 'ReceiptAlreadyClaimed', 'UnauthorizedVenue', 'InstructionFailed', 'InstructionDatesInvalid', 'InstructionSettleBlockPassed', 'InvalidSignature', 'SameSenderReceiver', 'SettleOnPastBlock', 'UnexpectedAffirmationStatus', 'FailedToSchedule', 'UnknownInstruction', 'SignerAlreadyExists', 'SignerDoesNotExist', 'ZeroAmount', 'InstructionSettleBlockNotReached', 'CallerIsNotAParty', 'MaxNumberOfNFTsExceeded', 'NumberOfTransferredNFTsUnderestimated', 'ReceiptForInvalidLegType', 'WeightLimitExceeded', 'MaxNumberOfFungibleAssetsExceeded', 'MaxNumberOfOffChainAssetsExceeded', 'NumberOfFungibleTransfersUnderestimated', 'UnexpectedOnChainAsset', 'UnexpectedOFFChainAsset', 'OffChainAssetCantBeLocked', 'NumberOfOffChainTransfersUnderestimated', 'LegNotFound', 'InputWeightIsLessThanMinimum', 'MaxNumberOfReceiptsExceeded', 'DuplicateReceiptUid', 'ReceiptInstructionIdMissmatch', 'MultipleReceiptsForOneLeg', 'UnexpectedLegStatus']
  },
  /**
   * Lookup697: polymesh_primitives::statistics::Stat1stKey
   **/
  PolymeshPrimitivesStatisticsStat1stKey: {
    asset: 'PolymeshPrimitivesStatisticsAssetScope',
    statType: 'PolymeshPrimitivesStatisticsStatType'
  },
  /**
   * Lookup698: polymesh_primitives::transfer_compliance::AssetTransferCompliance<S>
   **/
  PolymeshPrimitivesTransferComplianceAssetTransferCompliance: {
    paused: 'bool',
    requirements: 'BTreeSet<PolymeshPrimitivesTransferComplianceTransferCondition>'
  },
  /**
   * Lookup702: pallet_statistics::Error<T>
   **/
  PalletStatisticsError: {
    _enum: ['InvalidTransfer', 'StatTypeMissing', 'StatTypeNeededByTransferCondition', 'CannotRemoveStatTypeInUse', 'StatTypeLimitReached', 'TransferConditionLimitReached', 'WeightLimitExceeded']
  },
  /**
   * Lookup704: pallet_sto::Error<T>
   **/
  PalletStoError: {
    _enum: ['Unauthorized', 'Overflow', 'InsufficientTokensRemaining', 'FundraiserNotFound', 'FundraiserNotLive', 'FundraiserClosed', 'FundraiserExpired', 'InvalidVenue', 'InvalidPriceTiers', 'InvalidOfferingWindow', 'MaxPriceExceeded', 'InvestmentAmountTooLow']
  },
  /**
   * Lookup705: pallet_treasury::Error<T>
   **/
  PalletTreasuryError: {
    _enum: ['InsufficientBalance', 'InvalidIdentity']
  },
  /**
   * Lookup706: pallet_utility::pallet::Error<T>
   **/
  PalletUtilityError: {
    _enum: ['TooManyCalls', 'InvalidSignature', 'TargetCddMissing', 'InvalidNonce']
  },
  /**
   * Lookup707: pallet_base::Error<T>
   **/
  PalletBaseError: {
    _enum: ['TooLong', 'CounterOverflow']
  },
  /**
   * Lookup709: pallet_external_agents::Error<T>
   **/
  PalletExternalAgentsError: {
    _enum: ['NoSuchAG', 'UnauthorizedAgent', 'AlreadyAnAgent', 'NotAnAgent', 'RemovingLastFullAgent', 'SecondaryKeyNotAuthorizedForAsset']
  },
  /**
   * Lookup710: pallet_relayer::Subsidy<sp_core::crypto::AccountId32>
   **/
  PalletRelayerSubsidy: {
    payingKey: 'AccountId32',
    remaining: 'u128'
  },
  /**
   * Lookup711: pallet_relayer::Error<T>
   **/
  PalletRelayerError: {
    _enum: ['UserKeyCddMissing', 'PayingKeyCddMissing', 'NoPayingKey', 'NotPayingKey', 'NotAuthorizedForPayingKey', 'NotAuthorizedForUserKey', 'Overflow']
  },
  /**
   * Lookup712: pallet_rewards::Error<T>
   **/
  PalletRewardsError: {
    _enum: ['UnknownItnAddress', 'ItnRewardAlreadyClaimed', 'InvalidSignature', 'UnableToCovertBalance']
  },
  /**
   * Lookup714: pallet_contracts::wasm::PrefabWasmModule<T>
   **/
  PalletContractsWasmPrefabWasmModule: {
    instructionWeightsVersion: 'Compact<u32>',
    initial: 'Compact<u32>',
    maximum: 'Compact<u32>',
    code: 'Bytes',
    determinism: 'PalletContractsWasmDeterminism'
  },
  /**
   * Lookup716: pallet_contracts::wasm::OwnerInfo<T>
   **/
  PalletContractsWasmOwnerInfo: {
    owner: 'AccountId32',
    deposit: 'Compact<u128>',
    refcount: 'Compact<u64>'
  },
  /**
   * Lookup717: pallet_contracts::storage::ContractInfo<T>
   **/
  PalletContractsStorageContractInfo: {
    trieId: 'Bytes',
    depositAccount: 'AccountId32',
    codeHash: 'H256',
    storageBytes: 'u32',
    storageItems: 'u32',
    storageByteDeposit: 'u128',
    storageItemDeposit: 'u128',
    storageBaseDeposit: 'u128'
  },
  /**
   * Lookup720: pallet_contracts::storage::DeletedContract
   **/
  PalletContractsStorageDeletedContract: {
    trieId: 'Bytes'
  },
  /**
   * Lookup722: pallet_contracts::schedule::Schedule<T>
   **/
  PalletContractsSchedule: {
    limits: 'PalletContractsScheduleLimits',
    instructionWeights: 'PalletContractsScheduleInstructionWeights',
    hostFnWeights: 'PalletContractsScheduleHostFnWeights'
  },
  /**
   * Lookup723: pallet_contracts::schedule::Limits
   **/
  PalletContractsScheduleLimits: {
    eventTopics: 'u32',
    globals: 'u32',
    locals: 'u32',
    parameters: 'u32',
    memoryPages: 'u32',
    tableSize: 'u32',
    brTableSize: 'u32',
    subjectLen: 'u32',
    payloadLen: 'u32'
  },
  /**
   * Lookup724: pallet_contracts::schedule::InstructionWeights<T>
   **/
  PalletContractsScheduleInstructionWeights: {
    _alias: {
      r_if: 'r#if'
    },
    version: 'u32',
    fallback: 'u32',
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
    callPerLocal: 'u32',
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
   * Lookup725: pallet_contracts::schedule::HostFnWeights<T>
   **/
  PalletContractsScheduleHostFnWeights: {
    _alias: {
      r_return: 'r#return'
    },
    caller: 'SpWeightsWeightV2Weight',
    isContract: 'SpWeightsWeightV2Weight',
    codeHash: 'SpWeightsWeightV2Weight',
    ownCodeHash: 'SpWeightsWeightV2Weight',
    callerIsOrigin: 'SpWeightsWeightV2Weight',
    address: 'SpWeightsWeightV2Weight',
    gasLeft: 'SpWeightsWeightV2Weight',
    balance: 'SpWeightsWeightV2Weight',
    valueTransferred: 'SpWeightsWeightV2Weight',
    minimumBalance: 'SpWeightsWeightV2Weight',
    blockNumber: 'SpWeightsWeightV2Weight',
    now: 'SpWeightsWeightV2Weight',
    weightToFee: 'SpWeightsWeightV2Weight',
    gas: 'SpWeightsWeightV2Weight',
    input: 'SpWeightsWeightV2Weight',
    inputPerByte: 'SpWeightsWeightV2Weight',
    r_return: 'SpWeightsWeightV2Weight',
    returnPerByte: 'SpWeightsWeightV2Weight',
    terminate: 'SpWeightsWeightV2Weight',
    random: 'SpWeightsWeightV2Weight',
    depositEvent: 'SpWeightsWeightV2Weight',
    depositEventPerTopic: 'SpWeightsWeightV2Weight',
    depositEventPerByte: 'SpWeightsWeightV2Weight',
    debugMessage: 'SpWeightsWeightV2Weight',
    debugMessagePerByte: 'SpWeightsWeightV2Weight',
    setStorage: 'SpWeightsWeightV2Weight',
    setStoragePerNewByte: 'SpWeightsWeightV2Weight',
    setStoragePerOldByte: 'SpWeightsWeightV2Weight',
    setCodeHash: 'SpWeightsWeightV2Weight',
    clearStorage: 'SpWeightsWeightV2Weight',
    clearStoragePerByte: 'SpWeightsWeightV2Weight',
    containsStorage: 'SpWeightsWeightV2Weight',
    containsStoragePerByte: 'SpWeightsWeightV2Weight',
    getStorage: 'SpWeightsWeightV2Weight',
    getStoragePerByte: 'SpWeightsWeightV2Weight',
    takeStorage: 'SpWeightsWeightV2Weight',
    takeStoragePerByte: 'SpWeightsWeightV2Weight',
    transfer: 'SpWeightsWeightV2Weight',
    call: 'SpWeightsWeightV2Weight',
    delegateCall: 'SpWeightsWeightV2Weight',
    callTransferSurcharge: 'SpWeightsWeightV2Weight',
    callPerClonedByte: 'SpWeightsWeightV2Weight',
    instantiate: 'SpWeightsWeightV2Weight',
    instantiateTransferSurcharge: 'SpWeightsWeightV2Weight',
    instantiatePerInputByte: 'SpWeightsWeightV2Weight',
    instantiatePerSaltByte: 'SpWeightsWeightV2Weight',
    hashSha2256: 'SpWeightsWeightV2Weight',
    hashSha2256PerByte: 'SpWeightsWeightV2Weight',
    hashKeccak256: 'SpWeightsWeightV2Weight',
    hashKeccak256PerByte: 'SpWeightsWeightV2Weight',
    hashBlake2256: 'SpWeightsWeightV2Weight',
    hashBlake2256PerByte: 'SpWeightsWeightV2Weight',
    hashBlake2128: 'SpWeightsWeightV2Weight',
    hashBlake2128PerByte: 'SpWeightsWeightV2Weight',
    ecdsaRecover: 'SpWeightsWeightV2Weight',
    ecdsaToEthAddress: 'SpWeightsWeightV2Weight',
    reentranceCount: 'SpWeightsWeightV2Weight',
    accountReentranceCount: 'SpWeightsWeightV2Weight',
    instantiationNonce: 'SpWeightsWeightV2Weight'
  },
  /**
   * Lookup726: pallet_contracts::pallet::Error<T>
   **/
  PalletContractsError: {
    _enum: ['InvalidScheduleVersion', 'InvalidCallFlags', 'OutOfGas', 'OutputBufferTooSmall', 'TransferFailed', 'MaxCallDepthReached', 'ContractNotFound', 'CodeTooLarge', 'CodeNotFound', 'OutOfBounds', 'DecodingFailed', 'ContractTrapped', 'ValueTooLarge', 'TerminatedWhileReentrant', 'InputForwarded', 'RandomSubjectTooLong', 'TooManyTopics', 'NoChainExtension', 'DeletionQueueFull', 'DuplicateContract', 'TerminatedInConstructor', 'ReentranceDenied', 'StorageDepositNotEnoughFunds', 'StorageDepositLimitExhausted', 'CodeInUse', 'ContractReverted', 'CodeRejected', 'Indeterministic']
  },
  /**
   * Lookup727: polymesh_contracts::Error<T>
   **/
  PolymeshContractsError: {
    _enum: ['InvalidFuncId', 'InvalidRuntimeCall', 'ReadStorageFailed', 'DataLeftAfterDecoding', 'InLenTooLarge', 'OutLenTooLarge', 'InstantiatorWithNoIdentity', 'RuntimeCallDenied']
  },
  /**
   * Lookup728: pallet_preimage::RequestStatus<sp_core::crypto::AccountId32, Balance>
   **/
  PalletPreimageRequestStatus: {
    _enum: {
      Unrequested: {
        deposit: '(AccountId32,u128)',
        len: 'u32',
      },
      Requested: {
        deposit: 'Option<(AccountId32,u128)>',
        count: 'u32',
        len: 'Option<u32>'
      }
    }
  },
  /**
   * Lookup732: pallet_preimage::pallet::Error<T>
   **/
  PalletPreimageError: {
    _enum: ['TooBig', 'AlreadyNoted', 'NotAuthorized', 'NotNoted', 'Requested', 'NotRequested']
  },
  /**
   * Lookup733: polymesh_primitives::nft::NFTCollection
   **/
  PolymeshPrimitivesNftNftCollection: {
    id: 'u64',
    ticker: 'PolymeshPrimitivesTicker'
  },
  /**
   * Lookup737: pallet_nft::Error<T>
   **/
  PalletNftError: {
    _enum: ['BalanceOverflow', 'BalanceUnderflow', 'CollectionAlredyRegistered', 'CollectionNotFound', 'DuplicateMetadataKey', 'DuplicatedNFTId', 'InvalidAssetType', 'InvalidMetadataAttribute', 'InvalidNFTTransferCollectionNotFound', 'InvalidNFTTransferSamePortfolio', 'InvalidNFTTransferNFTNotOwned', 'InvalidNFTTransferCountOverflow', 'InvalidNFTTransferComplianceFailure', 'InvalidNFTTransferFrozenAsset', 'InvalidNFTTransferInsufficientCount', 'MaxNumberOfKeysExceeded', 'MaxNumberOfNFTsPerLegExceeded', 'NFTNotFound', 'UnregisteredMetadataKey', 'ZeroCount']
  },
  /**
   * Lookup738: pallet_test_utils::Error<T>
   **/
  PalletTestUtilsError: 'Null',
  /**
   * Lookup741: frame_system::extensions::check_spec_version::CheckSpecVersion<T>
   **/
  FrameSystemExtensionsCheckSpecVersion: 'Null',
  /**
   * Lookup742: frame_system::extensions::check_tx_version::CheckTxVersion<T>
   **/
  FrameSystemExtensionsCheckTxVersion: 'Null',
  /**
   * Lookup743: frame_system::extensions::check_genesis::CheckGenesis<T>
   **/
  FrameSystemExtensionsCheckGenesis: 'Null',
  /**
   * Lookup746: frame_system::extensions::check_nonce::CheckNonce<T>
   **/
  FrameSystemExtensionsCheckNonce: 'Compact<u32>',
  /**
   * Lookup747: polymesh_extensions::check_weight::CheckWeight<T>
   **/
  PolymeshExtensionsCheckWeight: 'FrameSystemExtensionsCheckWeight',
  /**
   * Lookup748: frame_system::extensions::check_weight::CheckWeight<T>
   **/
  FrameSystemExtensionsCheckWeight: 'Null',
  /**
   * Lookup749: pallet_transaction_payment::ChargeTransactionPayment<T>
   **/
  PalletTransactionPaymentChargeTransactionPayment: 'Compact<u128>',
  /**
   * Lookup750: pallet_permissions::StoreCallMetadata<T>
   **/
  PalletPermissionsStoreCallMetadata: 'Null',
  /**
   * Lookup751: polymesh_runtime_develop::runtime::Runtime
   **/
  PolymeshRuntimeDevelopRuntime: 'Null'
};
