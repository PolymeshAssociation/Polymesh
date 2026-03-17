// Auto-generated via `yarn polkadot-types-from-defs`, do not edit
/* eslint-disable */

/* eslint-disable sort-keys */

export default {
  /**
   * Lookup3: frame_system::AccountInfo<Nonce, pallet_balances::types::AccountData<Balance>>
   **/
  FrameSystemAccountInfo: {
    nonce: 'u32',
    consumers: 'u32',
    providers: 'u32',
    sufficients: 'u32',
    data: 'PalletBalancesAccountData'
  },
  /**
   * Lookup5: pallet_balances::types::AccountData<Balance>
   **/
  PalletBalancesAccountData: {
    free: 'u128',
    reserved: 'u128',
    frozen: 'u128',
    flags: 'u128'
  },
  /**
   * Lookup9: frame_support::dispatch::PerDispatchClass<sp_weights::weight_v2::Weight>
   **/
  FrameSupportDispatchPerDispatchClassWeight: {
    normal: 'SpWeightsWeightV2Weight',
    operational: 'SpWeightsWeightV2Weight',
    mandatory: 'SpWeightsWeightV2Weight'
  },
  /**
   * Lookup10: sp_weights::weight_v2::Weight
   **/
  SpWeightsWeightV2Weight: {
    refTime: 'Compact<u64>',
    proofSize: 'Compact<u64>'
  },
  /**
   * Lookup15: sp_runtime::generic::digest::Digest
   **/
  SpRuntimeDigest: {
    logs: 'Vec<SpRuntimeDigestDigestItem>'
  },
  /**
   * Lookup17: sp_runtime::generic::digest::DigestItem
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
   * Lookup20: frame_system::EventRecord<polymesh_runtime_develop::runtime::RuntimeEvent, primitive_types::H256>
   **/
  FrameSystemEventRecord: {
    phase: 'FrameSystemPhase',
    event: 'Event',
    topics: 'Vec<H256>'
  },
  /**
   * Lookup22: frame_system::pallet::Event<T>
   **/
  FrameSystemEvent: {
    _enum: {
      ExtrinsicSuccess: {
        dispatchInfo: 'FrameSystemDispatchEventInfo',
      },
      ExtrinsicFailed: {
        dispatchError: 'SpRuntimeDispatchError',
        dispatchInfo: 'FrameSystemDispatchEventInfo',
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
        hash_: 'H256',
      },
      UpgradeAuthorized: {
        codeHash: 'H256',
        checkVersion: 'bool',
      },
      RejectedInvalidAuthorizedUpgrade: {
        codeHash: 'H256',
        error: 'SpRuntimeDispatchError'
      }
    }
  },
  /**
   * Lookup23: frame_system::DispatchEventInfo
   **/
  FrameSystemDispatchEventInfo: {
    weight: 'SpWeightsWeightV2Weight',
    class: 'FrameSupportDispatchDispatchClass',
    paysFee: 'FrameSupportDispatchPays'
  },
  /**
   * Lookup24: frame_support::dispatch::DispatchClass
   **/
  FrameSupportDispatchDispatchClass: {
    _enum: ['Normal', 'Operational', 'Mandatory']
  },
  /**
   * Lookup25: frame_support::dispatch::Pays
   **/
  FrameSupportDispatchPays: {
    _enum: ['Yes', 'No']
  },
  /**
   * Lookup26: sp_runtime::DispatchError
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
      Unavailable: 'Null',
      RootNotAllowed: 'Null',
      Trie: 'SpRuntimeProvingTrieTrieError'
    }
  },
  /**
   * Lookup27: sp_runtime::ModuleError
   **/
  SpRuntimeModuleError: {
    index: 'u8',
    error: '[u8;4]'
  },
  /**
   * Lookup28: sp_runtime::TokenError
   **/
  SpRuntimeTokenError: {
    _enum: ['FundsUnavailable', 'OnlyProvider', 'BelowMinimum', 'CannotCreate', 'UnknownAsset', 'Frozen', 'Unsupported', 'CannotCreateHold', 'NotExpendable', 'Blocked']
  },
  /**
   * Lookup29: sp_arithmetic::ArithmeticError
   **/
  SpArithmeticArithmeticError: {
    _enum: ['Underflow', 'Overflow', 'DivisionByZero']
  },
  /**
   * Lookup30: sp_runtime::TransactionalError
   **/
  SpRuntimeTransactionalError: {
    _enum: ['LimitReached', 'NoLayer']
  },
  /**
   * Lookup31: sp_runtime::proving_trie::TrieError
   **/
  SpRuntimeProvingTrieTrieError: {
    _enum: ['InvalidStateRoot', 'IncompleteDatabase', 'ValueAtIncompleteKey', 'DecoderError', 'InvalidHash', 'DuplicateKey', 'ExtraneousNode', 'ExtraneousValue', 'ExtraneousHashReference', 'InvalidChildReference', 'ValueMismatch', 'IncompleteProof', 'RootMismatch', 'DecodeError']
  },
  /**
   * Lookup32: pallet_indices::pallet::Event<T>
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
        who: 'AccountId32',
      },
      DepositPoked: {
        who: 'AccountId32',
        index: 'u32',
        oldDeposit: 'u128',
        newDeposit: 'u128'
      }
    }
  },
  /**
   * Lookup33: pallet_balances::pallet::Event<T, I>
   **/
  PalletBalancesEvent: {
    _enum: {
      Endowed: {
        account: 'AccountId32',
        freeBalance: 'u128',
      },
      DustLost: {
        account: 'AccountId32',
        amount: 'u128',
      },
      Transfer: {
        from: 'AccountId32',
        to: 'AccountId32',
        amount: 'u128',
      },
      BalanceSet: {
        who: 'AccountId32',
        free: 'u128',
      },
      Reserved: {
        who: 'AccountId32',
        amount: 'u128',
      },
      Unreserved: {
        who: 'AccountId32',
        amount: 'u128',
      },
      ReserveRepatriated: {
        from: 'AccountId32',
        to: 'AccountId32',
        amount: 'u128',
        destinationStatus: 'FrameSupportTokensMiscBalanceStatus',
      },
      Deposit: {
        who: 'AccountId32',
        amount: 'u128',
      },
      Withdraw: {
        who: 'AccountId32',
        amount: 'u128',
      },
      Slashed: {
        who: 'AccountId32',
        amount: 'u128',
      },
      Minted: {
        who: 'AccountId32',
        amount: 'u128',
      },
      MintedCredit: {
        amount: 'u128',
      },
      Burned: {
        who: 'AccountId32',
        amount: 'u128',
      },
      BurnedDebt: {
        amount: 'u128',
      },
      Suspended: {
        who: 'AccountId32',
        amount: 'u128',
      },
      Restored: {
        who: 'AccountId32',
        amount: 'u128',
      },
      Upgraded: {
        who: 'AccountId32',
      },
      Issued: {
        amount: 'u128',
      },
      Rescinded: {
        amount: 'u128',
      },
      Locked: {
        who: 'AccountId32',
        amount: 'u128',
      },
      Unlocked: {
        who: 'AccountId32',
        amount: 'u128',
      },
      Frozen: {
        who: 'AccountId32',
        amount: 'u128',
      },
      Thawed: {
        who: 'AccountId32',
        amount: 'u128',
      },
      TotalIssuanceForced: {
        _alias: {
          new_: 'new',
        },
        old: 'u128',
        new_: 'u128',
      },
      Held: {
        reason: 'PolymeshRuntimeDevelopRuntimeRuntimeHoldReason',
        who: 'AccountId32',
        amount: 'u128',
      },
      BurnedHeld: {
        reason: 'PolymeshRuntimeDevelopRuntimeRuntimeHoldReason',
        who: 'AccountId32',
        amount: 'u128',
      },
      TransferOnHold: {
        reason: 'PolymeshRuntimeDevelopRuntimeRuntimeHoldReason',
        source: 'AccountId32',
        dest: 'AccountId32',
        amount: 'u128',
      },
      TransferAndHold: {
        reason: 'PolymeshRuntimeDevelopRuntimeRuntimeHoldReason',
        source: 'AccountId32',
        dest: 'AccountId32',
        transferred: 'u128',
      },
      Released: {
        reason: 'PolymeshRuntimeDevelopRuntimeRuntimeHoldReason',
        who: 'AccountId32',
        amount: 'u128',
      },
      Unexpected: 'PalletBalancesUnexpectedKind',
      TransferWithMemo: {
        from: 'AccountId32',
        to: 'AccountId32',
        amount: 'u128',
        memo: 'Option<PolymeshPrimitivesMemo>'
      }
    }
  },
  /**
   * Lookup34: frame_support::traits::tokens::misc::BalanceStatus
   **/
  FrameSupportTokensMiscBalanceStatus: {
    _enum: ['Free', 'Reserved']
  },
  /**
   * Lookup35: polymesh_runtime_develop::runtime::RuntimeHoldReason
   **/
  PolymeshRuntimeDevelopRuntimeRuntimeHoldReason: {
    _enum: {
      __Unused0: 'Null',
      __Unused1: 'Null',
      __Unused2: 'Null',
      __Unused3: 'Null',
      __Unused4: 'Null',
      __Unused5: 'Null',
      __Unused6: 'Null',
      __Unused7: 'Null',
      __Unused8: 'Null',
      __Unused9: 'Null',
      __Unused10: 'Null',
      __Unused11: 'Null',
      __Unused12: 'Null',
      __Unused13: 'Null',
      __Unused14: 'Null',
      __Unused15: 'Null',
      __Unused16: 'Null',
      Staking: 'PalletStakingPalletHoldReason',
      __Unused18: 'Null',
      Session: 'PalletSessionHoldReason',
      __Unused20: 'Null',
      __Unused21: 'Null',
      __Unused22: 'Null',
      __Unused23: 'Null',
      __Unused24: 'Null',
      __Unused25: 'Null',
      __Unused26: 'Null',
      __Unused27: 'Null',
      __Unused28: 'Null',
      __Unused29: 'Null',
      __Unused30: 'Null',
      __Unused31: 'Null',
      __Unused32: 'Null',
      __Unused33: 'Null',
      __Unused34: 'Null',
      __Unused35: 'Null',
      __Unused36: 'Null',
      __Unused37: 'Null',
      __Unused38: 'Null',
      __Unused39: 'Null',
      __Unused40: 'Null',
      __Unused41: 'Null',
      __Unused42: 'Null',
      __Unused43: 'Null',
      __Unused44: 'Null',
      __Unused45: 'Null',
      Contracts: 'PalletContractsHoldReason',
      __Unused47: 'Null',
      Preimage: 'PalletPreimageHoldReason'
    }
  },
  /**
   * Lookup36: pallet_staking::pallet::pallet::HoldReason
   **/
  PalletStakingPalletHoldReason: {
    _enum: ['Staking']
  },
  /**
   * Lookup37: pallet_session::pallet::HoldReason
   **/
  PalletSessionHoldReason: {
    _enum: ['Keys']
  },
  /**
   * Lookup38: pallet_contracts::pallet::HoldReason
   **/
  PalletContractsHoldReason: {
    _enum: ['CodeUploadDepositReserve', 'StorageDepositReserve']
  },
  /**
   * Lookup39: pallet_preimage::pallet::HoldReason
   **/
  PalletPreimageHoldReason: {
    _enum: ['Preimage']
  },
  /**
   * Lookup40: pallet_balances::pallet::UnexpectedKind
   **/
  PalletBalancesUnexpectedKind: {
    _enum: ['BalanceUpdated', 'FailedToMutateAccount']
  },
  /**
   * Lookup42: polymesh_primitives::Memo
   **/
  PolymeshPrimitivesMemo: '[u8;32]',
  /**
   * Lookup43: pallet_transaction_payment::pallet::Event<T>
   **/
  PalletTransactionPaymentEvent: {
    _enum: {
      TransactionFeePaid: {
        who: 'AccountId32',
        actualFee: 'u128',
        tip: 'u128'
      }
    }
  },
  /**
   * Lookup44: pallet_identity::pallet::Event<T>
   **/
  PalletIdentityEvent: {
    _enum: {
      DidCreated: '(PolymeshPrimitivesIdentityId,AccountId32,Vec<PolymeshPrimitivesSecondaryKey>)',
      SecondaryKeysAdded: '(PolymeshPrimitivesIdentityId,Vec<PolymeshPrimitivesSecondaryKey>)',
      SecondaryKeysRemoved: '(PolymeshPrimitivesIdentityId,Vec<AccountId32>)',
      SecondaryKeyLeftIdentity: '(PolymeshPrimitivesIdentityId,AccountId32)',
      SecondaryKeyPermissionsUpdated: '(PolymeshPrimitivesIdentityId,AccountId32,PolymeshPrimitivesSecondaryKeyPermissions,PolymeshPrimitivesSecondaryKeyPermissions)',
      PrimaryKeyUpdated: '(PolymeshPrimitivesIdentityId,AccountId32,AccountId32)',
      ClaimAdded: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesIdentityClaim)',
      ClaimRevoked: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesIdentityClaim)',
      AuthorizationAdded: '(PolymeshPrimitivesIdentityId,Option<PolymeshPrimitivesIdentityId>,Option<AccountId32>,u64,PolymeshPrimitivesAuthorizationAuthorizationData,Option<u64>)',
      AuthorizationRevoked: '(Option<PolymeshPrimitivesIdentityId>,Option<AccountId32>,u64)',
      AuthorizationRejected: '(Option<PolymeshPrimitivesIdentityId>,Option<AccountId32>,u64)',
      AuthorizationConsumed: '(Option<PolymeshPrimitivesIdentityId>,Option<AccountId32>,u64)',
      AuthorizationRetryLimitReached: '(Option<PolymeshPrimitivesIdentityId>,Option<AccountId32>,u64)',
      SecondaryKeysFrozen: 'PolymeshPrimitivesIdentityId',
      SecondaryKeysUnfrozen: 'PolymeshPrimitivesIdentityId',
      CustomClaimTypeAdded: '(PolymeshPrimitivesIdentityId,u32,Bytes)',
      ChildDidCreated: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesIdentityId,AccountId32)',
      ChildDidUnlinked: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesIdentityId,PolymeshPrimitivesIdentityId)'
    }
  },
  /**
   * Lookup45: polymesh_primitives::identity_id::IdentityId
   **/
  PolymeshPrimitivesIdentityId: '[u8;32]',
  /**
   * Lookup47: polymesh_primitives::secondary_key::SecondaryKey<sp_core::crypto::AccountId32>
   **/
  PolymeshPrimitivesSecondaryKey: {
    key: 'AccountId32',
    permissions: 'PolymeshPrimitivesSecondaryKeyPermissions'
  },
  /**
   * Lookup48: polymesh_primitives::secondary_key::Permissions
   **/
  PolymeshPrimitivesSecondaryKeyPermissions: {
    asset: 'PolymeshPrimitivesSubsetSubsetRestrictionAssetId',
    extrinsic: 'PolymeshPrimitivesSecondaryKeyExtrinsicPermissions',
    portfolio: 'PolymeshPrimitivesSubsetSubsetRestrictionPortfolioId'
  },
  /**
   * Lookup49: polymesh_primitives::subset::SubsetRestriction<polymesh_primitives::asset::AssetId>
   **/
  PolymeshPrimitivesSubsetSubsetRestrictionAssetId: {
    _enum: {
      Whole: 'Null',
      These: 'BTreeSet<PolymeshPrimitivesAssetAssetId>',
      Except: 'BTreeSet<PolymeshPrimitivesAssetAssetId>'
    }
  },
  /**
   * Lookup50: polymesh_primitives::asset::AssetId
   **/
  PolymeshPrimitivesAssetAssetId: '[u8;16]',
  /**
   * Lookup54: polymesh_primitives::secondary_key::ExtrinsicPermissions
   **/
  PolymeshPrimitivesSecondaryKeyExtrinsicPermissions: {
    _enum: {
      Whole: 'Null',
      These: 'BTreeMap<Text, PolymeshPrimitivesSecondaryKeyPalletPermissions>',
      Except: 'BTreeMap<Text, PolymeshPrimitivesSecondaryKeyPalletPermissions>'
    }
  },
  /**
   * Lookup58: polymesh_primitives::secondary_key::PalletPermissions
   **/
  PolymeshPrimitivesSecondaryKeyPalletPermissions: {
    extrinsics: 'PolymeshPrimitivesSubsetSubsetRestrictionExtrinsicName'
  },
  /**
   * Lookup59: polymesh_primitives::subset::SubsetRestriction<polymesh_primitives::ExtrinsicName>
   **/
  PolymeshPrimitivesSubsetSubsetRestrictionExtrinsicName: {
    _enum: {
      Whole: 'Null',
      These: 'BTreeSet<Text>',
      Except: 'BTreeSet<Text>'
    }
  },
  /**
   * Lookup65: polymesh_primitives::subset::SubsetRestriction<polymesh_primitives::identity_id::PortfolioId>
   **/
  PolymeshPrimitivesSubsetSubsetRestrictionPortfolioId: {
    _enum: {
      Whole: 'Null',
      These: 'BTreeSet<PolymeshPrimitivesIdentityIdPortfolioId>',
      Except: 'BTreeSet<PolymeshPrimitivesIdentityIdPortfolioId>'
    }
  },
  /**
   * Lookup66: polymesh_primitives::identity_id::PortfolioId
   **/
  PolymeshPrimitivesIdentityIdPortfolioId: {
    did: 'PolymeshPrimitivesIdentityId',
    kind: 'PolymeshPrimitivesIdentityIdPortfolioKind'
  },
  /**
   * Lookup67: polymesh_primitives::identity_id::PortfolioKind
   **/
  PolymeshPrimitivesIdentityIdPortfolioKind: {
    _enum: {
      Default: 'Null',
      User: 'u64'
    }
  },
  /**
   * Lookup72: polymesh_primitives::identity_claim::IdentityClaim
   **/
  PolymeshPrimitivesIdentityClaim: {
    claimIssuer: 'PolymeshPrimitivesIdentityId',
    issuanceDate: 'u64',
    lastUpdateDate: 'u64',
    expiry: 'Option<u64>',
    claim: 'PolymeshPrimitivesIdentityClaimClaim'
  },
  /**
   * Lookup74: polymesh_primitives::identity_claim::Claim
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
      Custom: '(u32,Option<PolymeshPrimitivesIdentityClaimScope>)'
    }
  },
  /**
   * Lookup75: polymesh_primitives::identity_claim::Scope
   **/
  PolymeshPrimitivesIdentityClaimScope: {
    _enum: {
      Identity: 'PolymeshPrimitivesIdentityId',
      Asset: 'PolymeshPrimitivesAssetAssetId',
      Custom: 'Bytes'
    }
  },
  /**
   * Lookup76: polymesh_primitives::cdd_id::CddId
   **/
  PolymeshPrimitivesCddId: '[u8;32]',
  /**
   * Lookup77: polymesh_primitives::jurisdiction::CountryCode
   **/
  PolymeshPrimitivesJurisdictionCountryCode: {
    _enum: ['AF', 'AX', 'AL', 'DZ', 'AS', 'AD', 'AO', 'AI', 'AQ', 'AG', 'AR', 'AM', 'AW', 'AU', 'AT', 'AZ', 'BS', 'BH', 'BD', 'BB', 'BY', 'BE', 'BZ', 'BJ', 'BM', 'BT', 'BO', 'BA', 'BW', 'BV', 'BR', 'VG', 'IO', 'BN', 'BG', 'BF', 'BI', 'KH', 'CM', 'CA', 'CV', 'KY', 'CF', 'TD', 'CL', 'CN', 'HK', 'MO', 'CX', 'CC', 'CO', 'KM', 'CG', 'CD', 'CK', 'CR', 'CI', 'HR', 'CU', 'CY', 'CZ', 'DK', 'DJ', 'DM', 'DO', 'EC', 'EG', 'SV', 'GQ', 'ER', 'EE', 'ET', 'FK', 'FO', 'FJ', 'FI', 'FR', 'GF', 'PF', 'TF', 'GA', 'GM', 'GE', 'DE', 'GH', 'GI', 'GR', 'GL', 'GD', 'GP', 'GU', 'GT', 'GG', 'GN', 'GW', 'GY', 'HT', 'HM', 'VA', 'HN', 'HU', 'IS', 'IN', 'ID', 'IR', 'IQ', 'IE', 'IM', 'IL', 'IT', 'JM', 'JP', 'JE', 'JO', 'KZ', 'KE', 'KI', 'KP', 'KR', 'KW', 'KG', 'LA', 'LV', 'LB', 'LS', 'LR', 'LY', 'LI', 'LT', 'LU', 'MK', 'MG', 'MW', 'MY', 'MV', 'ML', 'MT', 'MH', 'MQ', 'MR', 'MU', 'YT', 'MX', 'FM', 'MD', 'MC', 'MN', 'ME', 'MS', 'MA', 'MZ', 'MM', 'NA', 'NR', 'NP', 'NL', 'AN', 'NC', 'NZ', 'NI', 'NE', 'NG', 'NU', 'NF', 'MP', 'NO', 'OM', 'PK', 'PW', 'PS', 'PA', 'PG', 'PY', 'PE', 'PH', 'PN', 'PL', 'PT', 'PR', 'QA', 'RE', 'RO', 'RU', 'RW', 'BL', 'SH', 'KN', 'LC', 'MF', 'PM', 'VC', 'WS', 'SM', 'ST', 'SA', 'SN', 'RS', 'SC', 'SL', 'SG', 'SK', 'SI', 'SB', 'SO', 'ZA', 'GS', 'SS', 'ES', 'LK', 'SD', 'SR', 'SJ', 'SZ', 'SE', 'CH', 'SY', 'TW', 'TJ', 'TZ', 'TH', 'TL', 'TG', 'TK', 'TO', 'TT', 'TN', 'TR', 'TM', 'TC', 'TV', 'UG', 'UA', 'AE', 'GB', 'US', 'UM', 'UY', 'UZ', 'VU', 'VE', 'VN', 'VI', 'WF', 'EH', 'YE', 'ZM', 'ZW', 'BQ', 'CW', 'SX']
  },
  /**
   * Lookup82: polymesh_primitives::authorization::AuthorizationData<sp_core::crypto::AccountId32>
   **/
  PolymeshPrimitivesAuthorizationAuthorizationData: {
    _enum: {
      AttestPrimaryKeyRotation: 'PolymeshPrimitivesIdentityId',
      RotatePrimaryKey: 'Null',
      TransferTicker: 'PolymeshPrimitivesTicker',
      AddMultiSigSigner: 'AccountId32',
      TransferAssetOwnership: 'PolymeshPrimitivesAssetAssetId',
      JoinIdentity: 'PolymeshPrimitivesSecondaryKeyPermissions',
      PortfolioCustody: 'PolymeshPrimitivesIdentityIdPortfolioId',
      BecomeAgent: '(PolymeshPrimitivesAssetAssetId,PolymeshPrimitivesAgentAgentGroup)',
      OldAddRelayerPayingKey: '(AccountId32,AccountId32,u128)',
      RotatePrimaryKeyToSecondary: 'PolymeshPrimitivesSecondaryKeyPermissions'
    }
  },
  /**
   * Lookup83: polymesh_primitives::ticker::Ticker
   **/
  PolymeshPrimitivesTicker: '[u8;12]',
  /**
   * Lookup85: polymesh_primitives::agent::AgentGroup
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
   * Lookup87: pallet_group::pallet::Event<T, I>
   **/
  PalletGroupEvent: {
    _enum: {
      MemberAdded: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesIdentityId)',
      MemberRemoved: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesIdentityId)',
      MemberRevoked: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesIdentityId)',
      MembersSwapped: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesIdentityId,PolymeshPrimitivesIdentityId)',
      MembersReset: '(PolymeshPrimitivesIdentityId,Vec<PolymeshPrimitivesIdentityId>)',
      ActiveLimitChanged: '(PolymeshPrimitivesIdentityId,u32,u32)'
    }
  },
  /**
   * Lookup89: pallet_committee::pallet::Event<T, I>
   **/
  PalletCommitteeEvent: {
    _enum: {
      Proposed: '(PolymeshPrimitivesIdentityId,u32,H256)',
      Voted: '(PolymeshPrimitivesIdentityId,u32,H256,bool,u32,u32,u32)',
      VoteRetracted: '(PolymeshPrimitivesIdentityId,u32,H256,bool)',
      FinalVotes: '(Option<PolymeshPrimitivesIdentityId>,u32,H256,Vec<PolymeshPrimitivesIdentityId>,Vec<PolymeshPrimitivesIdentityId>)',
      Approved: '(Option<PolymeshPrimitivesIdentityId>,H256,u32,u32,u32)',
      Rejected: '(Option<PolymeshPrimitivesIdentityId>,H256,u32,u32,u32)',
      Executed: '(Option<PolymeshPrimitivesIdentityId>,H256,Result<Null, SpRuntimeDispatchError>)',
      ReleaseCoordinatorUpdated: 'Option<PolymeshPrimitivesIdentityId>',
      ExpiresAfterUpdated: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesMaybeBlock)',
      VoteThresholdUpdated: '(PolymeshPrimitivesIdentityId,u32,u32)'
    }
  },
  /**
   * Lookup92: polymesh_primitives::MaybeBlock<BlockNumber>
   **/
  PolymeshPrimitivesMaybeBlock: {
    _enum: {
      Some: 'u32',
      None: 'Null'
    }
  },
  /**
   * Lookup98: pallet_multisig::pallet::Event<T>
   **/
  PalletMultisigEvent: {
    _enum: {
      MultiSigCreated: {
        callerDid: 'PolymeshPrimitivesIdentityId',
        multisig: 'AccountId32',
        caller: 'AccountId32',
        signers: 'Vec<AccountId32>',
        sigsRequired: 'u64',
      },
      ProposalAdded: {
        callerDid: 'Option<PolymeshPrimitivesIdentityId>',
        multisig: 'AccountId32',
        proposalId: 'u64',
      },
      ProposalExecuted: {
        callerDid: 'Option<PolymeshPrimitivesIdentityId>',
        multisig: 'AccountId32',
        proposalId: 'u64',
        result: 'Result<Null, SpRuntimeDispatchError>',
      },
      MultiSigSignerAdded: {
        callerDid: 'PolymeshPrimitivesIdentityId',
        multisig: 'AccountId32',
        signer: 'AccountId32',
      },
      MultiSigSignersAuthorized: {
        callerDid: 'PolymeshPrimitivesIdentityId',
        multisig: 'AccountId32',
        signers: 'Vec<AccountId32>',
      },
      MultiSigSignersRemoved: {
        callerDid: 'PolymeshPrimitivesIdentityId',
        multisig: 'AccountId32',
        signers: 'Vec<AccountId32>',
      },
      MultiSigSignersRequiredChanged: {
        callerDid: 'Option<PolymeshPrimitivesIdentityId>',
        multisig: 'AccountId32',
        sigsRequired: 'u64',
      },
      ProposalApprovalVote: {
        callerDid: 'Option<PolymeshPrimitivesIdentityId>',
        multisig: 'AccountId32',
        signer: 'AccountId32',
        proposalId: 'u64',
      },
      ProposalRejectionVote: {
        callerDid: 'Option<PolymeshPrimitivesIdentityId>',
        multisig: 'AccountId32',
        signer: 'AccountId32',
        proposalId: 'u64',
      },
      ProposalApproved: {
        callerDid: 'Option<PolymeshPrimitivesIdentityId>',
        multisig: 'AccountId32',
        proposalId: 'u64',
      },
      ProposalRejected: {
        callerDid: 'Option<PolymeshPrimitivesIdentityId>',
        multisig: 'AccountId32',
        proposalId: 'u64',
      },
      MultiSigAddedAdmin: {
        callerDid: 'PolymeshPrimitivesIdentityId',
        multisig: 'AccountId32',
        adminDid: 'PolymeshPrimitivesIdentityId',
      },
      MultiSigRemovedAdmin: {
        callerDid: 'PolymeshPrimitivesIdentityId',
        multisig: 'AccountId32',
        adminDid: 'PolymeshPrimitivesIdentityId',
      },
      MultiSigRemovedPayingDid: {
        callerDid: 'PolymeshPrimitivesIdentityId',
        multisig: 'AccountId32',
        payingDid: 'PolymeshPrimitivesIdentityId'
      }
    }
  },
  /**
   * Lookup100: pallet_validators::pallet::Event<T>
   **/
  PalletValidatorsEvent: {
    _enum: {
      Nominated: {
        nominatorIdentity: 'PolymeshPrimitivesIdentityId',
        stash: 'AccountId32',
        targets: 'Vec<AccountId32>',
      },
      PermissionedIdentityAdded: {
        governanceCouncillDid: 'PolymeshPrimitivesIdentityId',
        validatorsIdentity: 'PolymeshPrimitivesIdentityId',
      },
      PermissionedIdentityRemoved: {
        governanceCouncillDid: 'PolymeshPrimitivesIdentityId',
        validatorsIdentity: 'PolymeshPrimitivesIdentityId',
      },
      InvalidatedNominators: {
        governanceCouncillDid: 'PolymeshPrimitivesIdentityId',
        governanceCouncillAccount: 'PolymeshPrimitivesIdentityId',
        expiredNominators: 'Vec<AccountId32>',
      },
      SlashingAllowedForChanged: {
        slashingSwitch: 'PalletValidatorsSlashingSwitch',
      },
      RewardPaymentSchedulingInterrupted: {
        accountId: 'AccountId32',
        era: 'u32',
        error: 'SpRuntimeDispatchError',
      },
      CommissionCapUpdated: {
        governanceCouncillDid: 'PolymeshPrimitivesIdentityId',
        oldCommissionCap: 'Perbill',
        newCommissionCap: 'Perbill'
      }
    }
  },
  /**
   * Lookup101: pallet_validators::types::SlashingSwitch
   **/
  PalletValidatorsSlashingSwitch: {
    _enum: ['Validator', 'ValidatorAndNominator', 'None']
  },
  /**
   * Lookup103: pallet_staking::pallet::pallet::Event<T>
   **/
  PalletStakingPalletEvent: {
    _enum: {
      EraPaid: {
        eraIndex: 'u32',
        validatorPayout: 'u128',
        remainder: 'u128',
      },
      Rewarded: {
        stash: 'AccountId32',
        dest: 'PalletStakingRewardDestination',
        amount: 'u128',
      },
      Slashed: {
        staker: 'AccountId32',
        amount: 'u128',
      },
      SlashReported: {
        validator: 'AccountId32',
        fraction: 'Perbill',
        slashEra: 'u32',
      },
      OldSlashingReportDiscarded: {
        sessionIndex: 'u32',
      },
      StakersElected: 'Null',
      Bonded: {
        stash: 'AccountId32',
        amount: 'u128',
      },
      Unbonded: {
        stash: 'AccountId32',
        amount: 'u128',
      },
      Withdrawn: {
        stash: 'AccountId32',
        amount: 'u128',
      },
      Kicked: {
        nominator: 'AccountId32',
        stash: 'AccountId32',
      },
      StakingElectionFailed: 'Null',
      Chilled: {
        stash: 'AccountId32',
      },
      PayoutStarted: {
        eraIndex: 'u32',
        validatorStash: 'AccountId32',
        page: 'u32',
        next: 'Option<u32>',
      },
      ValidatorPrefsSet: {
        stash: 'AccountId32',
        prefs: 'PalletStakingValidatorPrefs',
      },
      SnapshotVotersSizeExceeded: {
        _alias: {
          size_: 'size',
        },
        size_: 'u32',
      },
      SnapshotTargetsSizeExceeded: {
        _alias: {
          size_: 'size',
        },
        size_: 'u32',
      },
      ForceEra: {
        mode: 'PalletStakingForcing',
      },
      ControllerBatchDeprecated: {
        failures: 'u32',
      },
      CurrencyMigrated: {
        stash: 'AccountId32',
        forceWithdraw: 'u128'
      }
    }
  },
  /**
   * Lookup104: pallet_staking::RewardDestination<sp_core::crypto::AccountId32>
   **/
  PalletStakingRewardDestination: {
    _enum: {
      Staked: 'Null',
      Stash: 'Null',
      Controller: 'Null',
      Account: 'AccountId32',
      None: 'Null'
    }
  },
  /**
   * Lookup106: pallet_staking::ValidatorPrefs
   **/
  PalletStakingValidatorPrefs: {
    commission: 'Compact<Perbill>',
    blocked: 'bool'
  },
  /**
   * Lookup108: pallet_staking::Forcing
   **/
  PalletStakingForcing: {
    _enum: ['NotForcing', 'ForceNew', 'ForceNone', 'ForceAlways']
  },
  /**
   * Lookup109: pallet_offences::pallet::Event
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
   * Lookup110: pallet_session::pallet::Event<T>
   **/
  PalletSessionEvent: {
    _enum: {
      NewSession: {
        sessionIndex: 'u32',
      },
      NewQueued: 'Null',
      ValidatorDisabled: {
        validator: 'AccountId32',
      },
      ValidatorReenabled: {
        validator: 'AccountId32'
      }
    }
  },
  /**
   * Lookup111: pallet_grandpa::pallet::Event
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
   * Lookup114: sp_consensus_grandpa::app::Public
   **/
  SpConsensusGrandpaAppPublic: '[u8;32]',
  /**
   * Lookup115: pallet_session::historical::pallet::Event<T>
   **/
  PalletSessionHistoricalPalletEvent: {
    _enum: {
      RootStored: {
        index: 'u32',
      },
      RootsPruned: {
        upTo: 'u32'
      }
    }
  },
  /**
   * Lookup116: pallet_im_online::pallet::Event<T>
   **/
  PalletImOnlineEvent: {
    _enum: {
      HeartbeatReceived: {
        authorityId: 'PalletImOnlineSr25519AppSr25519Public',
      },
      AllGood: 'Null',
      SomeOffline: {
        offline: 'Vec<(AccountId32,SpStakingExposure)>'
      }
    }
  },
  /**
   * Lookup117: pallet_im_online::sr25519::app_sr25519::Public
   **/
  PalletImOnlineSr25519AppSr25519Public: '[u8;32]',
  /**
   * Lookup120: sp_staking::Exposure<sp_core::crypto::AccountId32, Balance>
   **/
  SpStakingExposure: {
    total: 'Compact<u128>',
    own: 'Compact<u128>',
    others: 'Vec<SpStakingIndividualExposure>'
  },
  /**
   * Lookup123: sp_staking::IndividualExposure<sp_core::crypto::AccountId32, Balance>
   **/
  SpStakingIndividualExposure: {
    who: 'AccountId32',
    value: 'Compact<u128>'
  },
  /**
   * Lookup124: pallet_sudo::pallet::Event<T>
   **/
  PalletSudoEvent: {
    _enum: {
      Sudid: {
        sudoResult: 'Result<Null, SpRuntimeDispatchError>',
      },
      KeyChanged: {
        _alias: {
          new_: 'new',
        },
        old: 'Option<AccountId32>',
        new_: 'AccountId32',
      },
      KeyRemoved: 'Null',
      SudoAsDone: {
        sudoResult: 'Result<Null, SpRuntimeDispatchError>'
      }
    }
  },
  /**
   * Lookup125: pallet_asset::pallet::Event<T>
   **/
  PalletAssetEvent: {
    _enum: {
      AssetCreated: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesAssetAssetId,bool,PolymeshPrimitivesAssetAssetType,PolymeshPrimitivesIdentityId,Bytes,Vec<PolymeshPrimitivesAssetIdentifier>,Option<Bytes>)',
      IdentifiersUpdated: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesAssetAssetId,Vec<PolymeshPrimitivesAssetIdentifier>)',
      DivisibilityChanged: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesAssetAssetId,bool)',
      TickerRegistered: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesTicker,Option<u64>)',
      TickerTransferred: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesTicker,PolymeshPrimitivesIdentityId)',
      AssetOwnershipTransferred: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesAssetAssetId,PolymeshPrimitivesIdentityId)',
      AssetFrozen: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesAssetAssetId)',
      AssetUnfrozen: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesAssetAssetId)',
      AssetRenamed: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesAssetAssetId,Bytes)',
      FundingRoundSet: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesAssetAssetId,Bytes)',
      DocumentAdded: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesAssetAssetId,u32,PolymeshPrimitivesDocument)',
      DocumentRemoved: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesAssetAssetId,u32)',
      ControllerTransfer: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesAssetAssetId,PolymeshPrimitivesAssetAssetHolder,u128)',
      CustomAssetTypeExists: '(PolymeshPrimitivesIdentityId,u32,Bytes)',
      CustomAssetTypeRegistered: '(PolymeshPrimitivesIdentityId,u32,Bytes)',
      SetAssetMetadataValue: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesAssetAssetId,Bytes,Option<PolymeshPrimitivesAssetMetadataAssetMetadataValueDetail>)',
      SetAssetMetadataValueDetails: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesAssetAssetId,PolymeshPrimitivesAssetMetadataAssetMetadataValueDetail)',
      RegisterAssetMetadataLocalType: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesAssetAssetId,Bytes,u64,PolymeshPrimitivesAssetMetadataAssetMetadataSpec)',
      RegisterAssetMetadataGlobalType: '(Bytes,u64,PolymeshPrimitivesAssetMetadataAssetMetadataSpec)',
      AssetTypeChanged: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesAssetAssetId,PolymeshPrimitivesAssetAssetType)',
      LocalMetadataKeyDeleted: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesAssetAssetId,u64)',
      MetadataValueDeleted: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesAssetAssetId,PolymeshPrimitivesAssetMetadataAssetMetadataKey)',
      AssetBalanceUpdated: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesAssetAssetId,u128,Option<PolymeshPrimitivesAssetAssetHolder>,Option<PolymeshPrimitivesAssetAssetHolder>,PolymeshPrimitivesAssetHoldingsUpdateReason)',
      AssetAffirmationExemption: 'PolymeshPrimitivesAssetAssetId',
      RemoveAssetAffirmationExemption: 'PolymeshPrimitivesAssetAssetId',
      PreApprovedAsset: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesAssetAssetId)',
      RemovePreApprovedAsset: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesAssetAssetId)',
      AssetMediatorsAdded: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesAssetAssetId,BTreeSet<PolymeshPrimitivesIdentityId>)',
      AssetMediatorsRemoved: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesAssetAssetId,BTreeSet<PolymeshPrimitivesIdentityId>)',
      TickerLinkedToAsset: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesTicker,PolymeshPrimitivesAssetAssetId)',
      TickerUnlinkedFromAsset: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesTicker,PolymeshPrimitivesAssetAssetId)',
      GlobalMetadataSpecUpdated: '(Bytes,PolymeshPrimitivesAssetMetadataAssetMetadataSpec)',
      CreatedAssetTransfer: {
        assetId: 'PolymeshPrimitivesAssetAssetId',
        from: 'AccountId32',
        to: 'AccountId32',
        amount: 'u128',
        memo: 'Option<PolymeshPrimitivesMemo>',
        pendingTransferId: 'Option<u64>',
      },
      MandatoryReceiverAffirmationSet: '(PolymeshPrimitivesIdentityId,bool)'
    }
  },
  /**
   * Lookup126: polymesh_primitives::asset::AssetType
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
   * Lookup128: polymesh_primitives::asset::NonFungibleType
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
   * Lookup131: polymesh_primitives::asset_identifier::AssetIdentifier
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
   * Lookup137: polymesh_primitives::document::Document
   **/
  PolymeshPrimitivesDocument: {
    uri: 'Bytes',
    contentHash: 'PolymeshPrimitivesDocumentHash',
    name: 'Bytes',
    docType: 'Option<Bytes>',
    filingDate: 'Option<u64>'
  },
  /**
   * Lookup139: polymesh_primitives::document_hash::DocumentHash
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
   * Lookup148: polymesh_primitives::asset::AssetHolder
   **/
  PolymeshPrimitivesAssetAssetHolder: {
    _enum: {
      Portfolio: 'PolymeshPrimitivesIdentityIdPortfolioId',
      Account: 'AccountId32'
    }
  },
  /**
   * Lookup151: polymesh_primitives::asset_metadata::AssetMetadataValueDetail<Moment>
   **/
  PolymeshPrimitivesAssetMetadataAssetMetadataValueDetail: {
    expire: 'Option<u64>',
    lockStatus: 'PolymeshPrimitivesAssetMetadataAssetMetadataLockStatus'
  },
  /**
   * Lookup152: polymesh_primitives::asset_metadata::AssetMetadataLockStatus<Moment>
   **/
  PolymeshPrimitivesAssetMetadataAssetMetadataLockStatus: {
    _enum: {
      Unlocked: 'Null',
      Locked: 'Null',
      LockedUntil: 'u64'
    }
  },
  /**
   * Lookup155: polymesh_primitives::asset_metadata::AssetMetadataSpec
   **/
  PolymeshPrimitivesAssetMetadataAssetMetadataSpec: {
    url: 'Option<Bytes>',
    description: 'Option<Bytes>',
    typeDef: 'Option<Bytes>'
  },
  /**
   * Lookup162: polymesh_primitives::asset_metadata::AssetMetadataKey
   **/
  PolymeshPrimitivesAssetMetadataAssetMetadataKey: {
    _enum: {
      Global: 'u64',
      Local: 'u64'
    }
  },
  /**
   * Lookup164: polymesh_primitives::asset::HoldingsUpdateReason
   **/
  PolymeshPrimitivesAssetHoldingsUpdateReason: {
    _enum: {
      Issued: {
        fundingRoundName: 'Option<Bytes>',
      },
      Redeemed: 'Null',
      Transferred: {
        instructionId: 'Option<u64>',
        instructionMemo: 'Option<PolymeshPrimitivesMemo>',
      },
      ControllerTransfer: 'Null'
    }
  },
  /**
   * Lookup168: pallet_corporate_actions::distribution::pallet::Event<T>
   **/
  PalletCorporateActionsDistributionPalletEvent: {
    _enum: {
      Created: '(PolymeshPrimitivesEventOnly,PalletCorporateActionsCaId,PalletCorporateActionsDistribution)',
      BenefitClaimed: '(PolymeshPrimitivesEventOnly,PolymeshPrimitivesEventOnly,PalletCorporateActionsCaId,PalletCorporateActionsDistribution,u128,Permill)',
      Reclaimed: '(PolymeshPrimitivesEventOnly,PalletCorporateActionsCaId,u128)',
      Removed: '(PolymeshPrimitivesEventOnly,PalletCorporateActionsCaId)'
    }
  },
  /**
   * Lookup169: polymesh_primitives::event_only::EventOnly<polymesh_primitives::identity_id::IdentityId>
   **/
  PolymeshPrimitivesEventOnly: 'PolymeshPrimitivesIdentityId',
  /**
   * Lookup170: pallet_corporate_actions::CAId
   **/
  PalletCorporateActionsCaId: {
    assetId: 'PolymeshPrimitivesAssetAssetId',
    localId: 'u32'
  },
  /**
   * Lookup172: pallet_corporate_actions::distribution::Distribution
   **/
  PalletCorporateActionsDistribution: {
    from: 'PolymeshPrimitivesIdentityIdPortfolioId',
    currency: 'PolymeshPrimitivesAssetAssetId',
    perShare: 'u128',
    amount: 'u128',
    remaining: 'u128',
    reclaimed: 'bool',
    paymentAt: 'u64',
    expiresAt: 'Option<u64>'
  },
  /**
   * Lookup174: pallet_asset::checkpoint::pallet::Event<T>
   **/
  PalletAssetCheckpointPalletEvent: {
    _enum: {
      CheckpointCreated: '(Option<PolymeshPrimitivesIdentityId>,PolymeshPrimitivesAssetAssetId,u64,u128,u64)',
      MaximumSchedulesComplexityChanged: '(PolymeshPrimitivesIdentityId,u64)',
      ScheduleCreated: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesAssetAssetId,u64,PolymeshPrimitivesCheckpointScheduleCheckpoints)',
      ScheduleRemoved: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesAssetAssetId,u64,PolymeshPrimitivesCheckpointScheduleCheckpoints)'
    }
  },
  /**
   * Lookup177: polymesh_primitives::checkpoint::ScheduleCheckpoints
   **/
  PolymeshPrimitivesCheckpointScheduleCheckpoints: {
    pending: 'BTreeSet<u64>'
  },
  /**
   * Lookup180: pallet_compliance_manager::pallet::Event<T>
   **/
  PalletComplianceManagerEvent: {
    _enum: {
      ComplianceRequirementCreated: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesAssetAssetId,PolymeshPrimitivesComplianceManagerComplianceRequirement)',
      ComplianceRequirementRemoved: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesAssetAssetId,u32)',
      AssetComplianceReplaced: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesAssetAssetId,Vec<PolymeshPrimitivesComplianceManagerComplianceRequirement>)',
      AssetComplianceReset: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesAssetAssetId)',
      AssetComplianceResumed: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesAssetAssetId)',
      AssetCompliancePaused: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesAssetAssetId)',
      ComplianceRequirementChanged: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesAssetAssetId,PolymeshPrimitivesComplianceManagerComplianceRequirement)',
      TrustedDefaultClaimIssuerAdded: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesAssetAssetId,PolymeshPrimitivesConditionTrustedIssuer)',
      TrustedDefaultClaimIssuerRemoved: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesAssetAssetId,PolymeshPrimitivesIdentityId)'
    }
  },
  /**
   * Lookup181: polymesh_primitives::compliance_manager::ComplianceRequirement
   **/
  PolymeshPrimitivesComplianceManagerComplianceRequirement: {
    senderConditions: 'Vec<PolymeshPrimitivesCondition>',
    receiverConditions: 'Vec<PolymeshPrimitivesCondition>',
    id: 'u32'
  },
  /**
   * Lookup183: polymesh_primitives::condition::Condition
   **/
  PolymeshPrimitivesCondition: {
    conditionType: 'PolymeshPrimitivesConditionConditionType',
    issuers: 'Vec<PolymeshPrimitivesConditionTrustedIssuer>'
  },
  /**
   * Lookup184: polymesh_primitives::condition::ConditionType
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
   * Lookup186: polymesh_primitives::condition::TargetIdentity
   **/
  PolymeshPrimitivesConditionTargetIdentity: {
    _enum: {
      ExternalAgent: 'Null',
      Specific: 'PolymeshPrimitivesIdentityId'
    }
  },
  /**
   * Lookup188: polymesh_primitives::condition::TrustedIssuer
   **/
  PolymeshPrimitivesConditionTrustedIssuer: {
    issuer: 'PolymeshPrimitivesIdentityId',
    trustedFor: 'PolymeshPrimitivesConditionTrustedFor'
  },
  /**
   * Lookup189: polymesh_primitives::condition::TrustedFor
   **/
  PolymeshPrimitivesConditionTrustedFor: {
    _enum: {
      Any: 'Null',
      Specific: 'Vec<PolymeshPrimitivesIdentityClaimClaimType>'
    }
  },
  /**
   * Lookup191: polymesh_primitives::identity_claim::ClaimType
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
      Custom: 'u32'
    }
  },
  /**
   * Lookup193: pallet_corporate_actions::pallet::Event<T>
   **/
  PalletCorporateActionsEvent: {
    _enum: {
      MaxDetailsLengthChanged: '(PolymeshPrimitivesIdentityId,u32)',
      DefaultTargetIdentitiesChanged: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesAssetAssetId,PalletCorporateActionsTargetIdentities)',
      DefaultWithholdingTaxChanged: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesAssetAssetId,Permill)',
      DidWithholdingTaxChanged: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesAssetAssetId,PolymeshPrimitivesIdentityId,Option<Permill>)',
      CAInitiated: '(PolymeshPrimitivesEventOnly,PalletCorporateActionsCaId,PalletCorporateActionsCorporateAction,Bytes)',
      CALinkedToDoc: '(PolymeshPrimitivesIdentityId,PalletCorporateActionsCaId,Vec<u32>)',
      CARemoved: '(PolymeshPrimitivesEventOnly,PalletCorporateActionsCaId)',
      RecordDateChanged: '(PolymeshPrimitivesEventOnly,PalletCorporateActionsCaId,PalletCorporateActionsCorporateAction)'
    }
  },
  /**
   * Lookup194: pallet_corporate_actions::TargetIdentities
   **/
  PalletCorporateActionsTargetIdentities: {
    identities: 'Vec<PolymeshPrimitivesIdentityId>',
    treatment: 'PalletCorporateActionsTargetTreatment'
  },
  /**
   * Lookup195: pallet_corporate_actions::TargetTreatment
   **/
  PalletCorporateActionsTargetTreatment: {
    _enum: ['Include', 'Exclude']
  },
  /**
   * Lookup197: pallet_corporate_actions::CorporateAction
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
   * Lookup198: pallet_corporate_actions::CAKind
   **/
  PalletCorporateActionsCaKind: {
    _enum: ['PredictableBenefit', 'UnpredictableBenefit', 'IssuerNotice', 'Reorganization', 'Other']
  },
  /**
   * Lookup200: pallet_corporate_actions::RecordDate
   **/
  PalletCorporateActionsRecordDate: {
    date: 'u64',
    checkpoint: 'PalletCorporateActionsCaCheckpoint'
  },
  /**
   * Lookup201: pallet_corporate_actions::CACheckpoint
   **/
  PalletCorporateActionsCaCheckpoint: {
    _enum: {
      Scheduled: '(u64,u64)',
      Existing: 'u64'
    }
  },
  /**
   * Lookup206: pallet_corporate_actions::ballot::pallet::Event<T>
   **/
  PalletCorporateActionsBallotPalletEvent: {
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
   * Lookup207: pallet_corporate_actions::ballot::BallotTimeRange
   **/
  PalletCorporateActionsBallotBallotTimeRange: {
    start: 'u64',
    end: 'u64'
  },
  /**
   * Lookup208: pallet_corporate_actions::ballot::BallotMeta
   **/
  PalletCorporateActionsBallotBallotMeta: {
    title: 'Bytes',
    motions: 'Vec<PalletCorporateActionsBallotMotion>'
  },
  /**
   * Lookup211: pallet_corporate_actions::ballot::Motion
   **/
  PalletCorporateActionsBallotMotion: {
    title: 'Bytes',
    infoLink: 'Bytes',
    choices: 'Vec<Bytes>'
  },
  /**
   * Lookup217: pallet_corporate_actions::ballot::BallotVote
   **/
  PalletCorporateActionsBallotBallotVote: {
    power: 'u128',
    fallback: 'Option<u16>'
  },
  /**
   * Lookup220: pallet_pips::pallet::Event<T>
   **/
  PalletPipsEvent: {
    _enum: {
      HistoricalPipsPruned: '(PolymeshPrimitivesIdentityId,bool,bool)',
      ProposalCreated: '(PolymeshPrimitivesIdentityId,PalletPipsProposer,u32,u128,Option<Bytes>,Option<Bytes>,PolymeshPrimitivesMaybeBlock,PalletPipsProposalData)',
      ProposalStateUpdated: '(PolymeshPrimitivesIdentityId,u32,PalletPipsProposalState)',
      Voted: '(PolymeshPrimitivesIdentityId,AccountId32,u32,bool,u128)',
      PipClosed: '(PolymeshPrimitivesIdentityId,u32,bool)',
      ExecutionScheduled: '(PolymeshPrimitivesIdentityId,u32,u32)',
      DefaultEnactmentPeriodChanged: '(PolymeshPrimitivesIdentityId,u32,u32)',
      MinimumProposalDepositChanged: '(PolymeshPrimitivesIdentityId,u128,u128)',
      PendingPipExpiryChanged: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesMaybeBlock,PolymeshPrimitivesMaybeBlock)',
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
   * Lookup221: pallet_pips::types::Proposer<sp_core::crypto::AccountId32>
   **/
  PalletPipsProposer: {
    _enum: {
      Community: 'AccountId32',
      Committee: 'PalletPipsCommittee'
    }
  },
  /**
   * Lookup222: pallet_pips::types::Committee
   **/
  PalletPipsCommittee: {
    _enum: ['Technical', 'Upgrade']
  },
  /**
   * Lookup226: pallet_pips::types::ProposalData
   **/
  PalletPipsProposalData: {
    _enum: {
      Hash: 'H256',
      Proposal: 'Bytes'
    }
  },
  /**
   * Lookup227: pallet_pips::types::ProposalState
   **/
  PalletPipsProposalState: {
    _enum: ['Pending', 'Rejected', 'Scheduled', 'Failed', 'Executed', 'Expired']
  },
  /**
   * Lookup230: pallet_pips::types::SnapshottedPip
   **/
  PalletPipsSnapshottedPip: {
    id: 'u32',
    weight: '(bool,u128)'
  },
  /**
   * Lookup236: pallet_portfolio::pallet::Event<T>
   **/
  PalletPortfolioEvent: {
    _enum: {
      PortfolioCreated: '(PolymeshPrimitivesIdentityId,u64,Bytes)',
      PortfolioDeleted: '(PolymeshPrimitivesIdentityId,u64)',
      PortfolioRenamed: '(PolymeshPrimitivesIdentityId,u64,Bytes)',
      UserPortfolios: '(PolymeshPrimitivesIdentityId,Vec<(u64,Bytes)>)',
      PortfolioCustodianChanged: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesIdentityIdPortfolioId,PolymeshPrimitivesIdentityId)',
      FundsMovedBetweenPortfolios: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesIdentityIdPortfolioId,PolymeshPrimitivesIdentityIdPortfolioId,PolymeshPrimitivesPortfolioFundDescription,Option<PolymeshPrimitivesMemo>)',
      PreApprovedPortfolio: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesIdentityIdPortfolioId,PolymeshPrimitivesAssetAssetId)',
      RevokePreApprovedPortfolio: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesIdentityIdPortfolioId,PolymeshPrimitivesAssetAssetId)',
      AllowIdentityToCreatePortfolios: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesIdentityId)',
      RevokeCreatePortfoliosPermission: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesIdentityId)'
    }
  },
  /**
   * Lookup240: polymesh_primitives::portfolio::FundDescription
   **/
  PolymeshPrimitivesPortfolioFundDescription: {
    _enum: {
      Fungible: {
        assetId: 'PolymeshPrimitivesAssetAssetId',
        amount: 'u128',
      },
      NonFungible: 'PolymeshPrimitivesNftNfTs'
    }
  },
  /**
   * Lookup241: polymesh_primitives::nft::NFTs
   **/
  PolymeshPrimitivesNftNfTs: {
    assetId: 'PolymeshPrimitivesAssetAssetId',
    ids: 'Vec<u64>'
  },
  /**
   * Lookup244: pallet_protocol_fee::pallet::Event<T>
   **/
  PalletProtocolFeeEvent: {
    _enum: {
      FeeSet: '(PolymeshPrimitivesIdentityId,u128)',
      CoefficientSet: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesPosRatio)',
      FeeCharged: '(AccountId32,u128)'
    }
  },
  /**
   * Lookup245: polymesh_primitives::PosRatio
   **/
  PolymeshPrimitivesPosRatio: '(u32,u32)',
  /**
   * Lookup246: pallet_scheduler::pallet::Event<T>
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
      RetrySet: {
        task: '(u32,u32)',
        id: 'Option<[u8;32]>',
        period: 'u32',
        retries: 'u8',
      },
      RetryCancelled: {
        task: '(u32,u32)',
        id: 'Option<[u8;32]>',
      },
      CallUnavailable: {
        task: '(u32,u32)',
        id: 'Option<[u8;32]>',
      },
      PeriodicFailed: {
        task: '(u32,u32)',
        id: 'Option<[u8;32]>',
      },
      RetryFailed: {
        task: '(u32,u32)',
        id: 'Option<[u8;32]>',
      },
      PermanentlyOverweight: {
        task: '(u32,u32)',
        id: 'Option<[u8;32]>',
      },
      AgendaIncomplete: {
        when: 'u32'
      }
    }
  },
  /**
   * Lookup249: pallet_settlement::pallet::Event<T>
   **/
  PalletSettlementEvent: {
    _enum: {
      VenueCreated: '(PolymeshPrimitivesIdentityId,u64,Bytes,PolymeshPrimitivesSettlementVenueType)',
      VenueDetailsUpdated: '(PolymeshPrimitivesIdentityId,u64,Bytes)',
      VenueTypeUpdated: '(PolymeshPrimitivesIdentityId,u64,PolymeshPrimitivesSettlementVenueType)',
      InstructionAffirmed: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesAssetAssetHolder,u64)',
      AffirmationWithdrawn: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesAssetAssetHolder,u64)',
      InstructionRejected: '(PolymeshPrimitivesIdentityId,u64)',
      ReceiptClaimed: '(PolymeshPrimitivesIdentityId,u64,u64,u64,AccountId32,Option<PolymeshPrimitivesSettlementReceiptMetadata>)',
      VenueFiltering: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesAssetAssetId,bool)',
      VenuesAllowed: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesAssetAssetId,Vec<u64>)',
      VenuesBlocked: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesAssetAssetId,Vec<u64>)',
      LegFailedExecution: '(PolymeshPrimitivesIdentityId,u64,u64)',
      InstructionExecuted: '(PolymeshPrimitivesIdentityId,u64)',
      VenueUnauthorized: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesAssetAssetId,u64)',
      SchedulingFailed: '(u64,SpRuntimeDispatchError)',
      InstructionRescheduled: '(PolymeshPrimitivesIdentityId,u64)',
      VenueSignersUpdated: '(PolymeshPrimitivesIdentityId,u64,Vec<AccountId32>,bool)',
      SettlementManuallyExecuted: '(PolymeshPrimitivesIdentityId,u64)',
      InstructionCreated: '(PolymeshPrimitivesIdentityId,Option<u64>,u64,PolymeshPrimitivesSettlementSettlementType,Option<u64>,Option<u64>,Vec<PolymeshPrimitivesSettlementLeg>,Option<PolymeshPrimitivesMemo>)',
      FailedToExecuteInstruction: '(u64,SpRuntimeDispatchError)',
      InstructionAutomaticallyAffirmed: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesAssetAssetHolder,u64)',
      MediatorAffirmationReceived: '(PolymeshPrimitivesIdentityId,u64,Option<u64>)',
      MediatorAffirmationWithdrawn: '(PolymeshPrimitivesIdentityId,u64)',
      InstructionMediators: '(u64,BTreeSet<PolymeshPrimitivesIdentityId>)',
      InstructionLocked: '(PolymeshPrimitivesIdentityId,u64)'
    }
  },
  /**
   * Lookup252: polymesh_primitives::settlement::VenueType
   **/
  PolymeshPrimitivesSettlementVenueType: {
    _enum: ['Other', 'Distribution', 'Sto', 'Exchange']
  },
  /**
   * Lookup255: polymesh_primitives::settlement::ReceiptMetadata
   **/
  PolymeshPrimitivesSettlementReceiptMetadata: '[u8;32]',
  /**
   * Lookup258: polymesh_primitives::settlement::SettlementType<BlockNumber>
   **/
  PolymeshPrimitivesSettlementSettlementType: {
    _enum: {
      SettleOnAffirmation: 'Null',
      SettleOnBlock: 'u32',
      SettleManual: 'u32',
      SettleAfterLock: 'Null'
    }
  },
  /**
   * Lookup260: polymesh_primitives::settlement::Leg
   **/
  PolymeshPrimitivesSettlementLeg: {
    _enum: {
      Fungible: {
        sender: 'PolymeshPrimitivesAssetAssetHolder',
        receiver: 'PolymeshPrimitivesAssetAssetHolder',
        assetId: 'PolymeshPrimitivesAssetAssetId',
        amount: 'u128',
      },
      NonFungible: {
        sender: 'PolymeshPrimitivesAssetAssetHolder',
        receiver: 'PolymeshPrimitivesAssetAssetHolder',
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
   * Lookup261: pallet_statistics::pallet::Event<T>
   **/
  PalletStatisticsEvent: {
    _enum: {
      StatTypesAdded: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesAssetAssetId,Vec<PolymeshPrimitivesStatisticsStatType>)',
      StatTypesRemoved: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesAssetAssetId,Vec<PolymeshPrimitivesStatisticsStatType>)',
      AssetStatsUpdated: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesAssetAssetId,PolymeshPrimitivesStatisticsStatType,Vec<PolymeshPrimitivesStatisticsStatUpdate>)',
      SetAssetTransferCompliance: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesAssetAssetId,Vec<PolymeshPrimitivesTransferComplianceTransferCondition>)',
      TransferConditionExemptionsAdded: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesTransferComplianceTransferConditionExemptKey,Vec<PolymeshPrimitivesIdentityId>)',
      TransferConditionExemptionsRemoved: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesTransferComplianceTransferConditionExemptKey,Vec<PolymeshPrimitivesIdentityId>)'
    }
  },
  /**
   * Lookup263: polymesh_primitives::statistics::StatType
   **/
  PolymeshPrimitivesStatisticsStatType: {
    operationType: 'PolymeshPrimitivesStatisticsStatOpType',
    claimIssuer: 'Option<(PolymeshPrimitivesIdentityClaimClaimType,PolymeshPrimitivesIdentityId)>'
  },
  /**
   * Lookup264: polymesh_primitives::statistics::StatOpType
   **/
  PolymeshPrimitivesStatisticsStatOpType: {
    _enum: ['Count', 'Balance']
  },
  /**
   * Lookup268: polymesh_primitives::statistics::StatUpdate
   **/
  PolymeshPrimitivesStatisticsStatUpdate: {
    key2: 'PolymeshPrimitivesStatisticsStat2ndKey',
    value: 'Option<u128>'
  },
  /**
   * Lookup269: polymesh_primitives::statistics::Stat2ndKey
   **/
  PolymeshPrimitivesStatisticsStat2ndKey: {
    _enum: {
      NoClaimStat: 'Null',
      Claim: 'PolymeshPrimitivesStatisticsStatClaim'
    }
  },
  /**
   * Lookup270: polymesh_primitives::statistics::StatClaim
   **/
  PolymeshPrimitivesStatisticsStatClaim: {
    _enum: {
      Accredited: 'bool',
      Affiliate: 'bool',
      Jurisdiction: 'Option<PolymeshPrimitivesJurisdictionCountryCode>'
    }
  },
  /**
   * Lookup274: polymesh_primitives::transfer_compliance::TransferCondition
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
   * Lookup275: polymesh_primitives::transfer_compliance::TransferConditionExemptKey
   **/
  PolymeshPrimitivesTransferComplianceTransferConditionExemptKey: {
    assetId: 'PolymeshPrimitivesAssetAssetId',
    op: 'PolymeshPrimitivesStatisticsStatOpType',
    claimType: 'Option<PolymeshPrimitivesIdentityClaimClaimType>'
  },
  /**
   * Lookup277: pallet_sto::pallet::Event<T>
   **/
  PalletStoEvent: {
    _enum: {
      FundraiserCreated: {
        agentDid: 'PolymeshPrimitivesIdentityId',
        offeringAsset: 'PolymeshPrimitivesAssetAssetId',
        raisingAsset: 'PolymeshPrimitivesAssetAssetId',
        fundraiserId: 'u64',
        fundraiserName: 'Bytes',
        fundraiser: 'PalletStoFundraiser',
      },
      Invested: {
        investorDid: 'PolymeshPrimitivesIdentityId',
        offeringAsset: 'PolymeshPrimitivesAssetAssetId',
        fundraiserId: 'u64',
        fundingAsset: 'PalletStoFundingAsset',
        offeringAmount: 'u128',
        raiseAmount: 'u128',
      },
      FundraiserFrozen: {
        agentDid: 'PolymeshPrimitivesIdentityId',
        offeringAsset: 'PolymeshPrimitivesAssetAssetId',
        fundraiserId: 'u64',
      },
      FundraiserUnfrozen: {
        agentDid: 'PolymeshPrimitivesIdentityId',
        offeringAsset: 'PolymeshPrimitivesAssetAssetId',
        fundraiserId: 'u64',
      },
      FundraiserWindowModified: {
        agentDid: 'PolymeshPrimitivesEventOnly',
        offeringAsset: 'PolymeshPrimitivesAssetAssetId',
        fundraiserId: 'u64',
        oldStart: 'u64',
        oldEnd: 'Option<u64>',
        newStart: 'u64',
        newEnd: 'Option<u64>',
      },
      FundraiserClosed: {
        agentDid: 'PolymeshPrimitivesIdentityId',
        offeringAsset: 'PolymeshPrimitivesAssetAssetId',
        fundraiserId: 'u64',
      },
      FundraiserOffchainFundingEnabled: {
        agentDid: 'PolymeshPrimitivesIdentityId',
        offeringAsset: 'PolymeshPrimitivesAssetAssetId',
        fundraiserId: 'u64',
        ticker: 'PolymeshPrimitivesTicker'
      }
    }
  },
  /**
   * Lookup280: pallet_sto::Fundraiser<Moment>
   **/
  PalletStoFundraiser: {
    creator: 'PolymeshPrimitivesIdentityId',
    offeringPortfolio: 'PolymeshPrimitivesIdentityIdPortfolioId',
    offeringAsset: 'PolymeshPrimitivesAssetAssetId',
    raisingPortfolio: 'PolymeshPrimitivesIdentityIdPortfolioId',
    raisingAsset: 'PolymeshPrimitivesAssetAssetId',
    tiers: 'Vec<PalletStoFundraiserTier>',
    venueId: 'u64',
    start: 'u64',
    end: 'Option<u64>',
    status: 'PalletStoFundraiserStatus',
    minimumInvestment: 'u128'
  },
  /**
   * Lookup282: pallet_sto::FundraiserTier
   **/
  PalletStoFundraiserTier: {
    total: 'u128',
    price: 'u128',
    remaining: 'u128'
  },
  /**
   * Lookup283: pallet_sto::FundraiserStatus
   **/
  PalletStoFundraiserStatus: {
    _enum: ['Live', 'Frozen', 'Closed', 'ClosedEarly']
  },
  /**
   * Lookup284: pallet_sto::FundingAsset
   **/
  PalletStoFundingAsset: {
    _enum: {
      OnChain: 'PolymeshPrimitivesAssetAssetId',
      OffChain: 'PolymeshPrimitivesTicker'
    }
  },
  /**
   * Lookup285: pallet_treasury::pallet::Event<T>
   **/
  PalletTreasuryEvent: {
    _enum: {
      TreasuryDisbursement: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesIdentityId,AccountId32,u128)',
      TreasuryDisbursementFailed: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesIdentityId,AccountId32,u128)',
      TreasuryReimbursement: '(PolymeshPrimitivesIdentityId,u128)'
    }
  },
  /**
   * Lookup286: pallet_utility::pallet::Event<T>
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
        result: 'Result<Null, SpRuntimeDispatchError>'
      }
    }
  },
  /**
   * Lookup287: pallet_external_agents::pallet::Event<T>
   **/
  PalletExternalAgentsEvent: {
    _enum: {
      GroupCreated: '(PolymeshPrimitivesEventOnly,PolymeshPrimitivesAssetAssetId,u32,PolymeshPrimitivesSecondaryKeyExtrinsicPermissions)',
      GroupPermissionsUpdated: '(PolymeshPrimitivesEventOnly,PolymeshPrimitivesAssetAssetId,u32,PolymeshPrimitivesSecondaryKeyExtrinsicPermissions)',
      AgentAdded: '(PolymeshPrimitivesEventOnly,PolymeshPrimitivesAssetAssetId,PolymeshPrimitivesAgentAgentGroup)',
      AgentRemoved: '(PolymeshPrimitivesEventOnly,PolymeshPrimitivesAssetAssetId,PolymeshPrimitivesIdentityId)',
      GroupChanged: '(PolymeshPrimitivesEventOnly,PolymeshPrimitivesAssetAssetId,PolymeshPrimitivesIdentityId,PolymeshPrimitivesAgentAgentGroup)'
    }
  },
  /**
   * Lookup288: pallet_relayer::pallet::Event<T>
   **/
  PalletRelayerEvent: {
    _enum: {
      ApprovedSubsidy: {
        userKey: 'AccountId32',
        payingKey: 'AccountId32',
        initialPolyxLimit: 'u128',
      },
      AcceptedSubsidy: {
        userKey: 'AccountId32',
        payingKey: 'AccountId32',
        initialPolyxLimit: 'u128',
      },
      RemovedSubsidy: {
        userKey: 'AccountId32',
        payingKey: 'AccountId32',
        remaining: 'u128',
      },
      RemovedPendingSubsidy: {
        userKey: 'AccountId32',
        payingKey: 'AccountId32',
        initialPolyxLimit: 'u128',
      },
      UpdatedPolyxLimit: {
        userKey: 'AccountId32',
        payingKey: 'AccountId32',
        remaining: 'u128',
        oldRemaining: 'u128',
      },
      SubsidyDebited: {
        userKey: 'AccountId32',
        payingKey: 'AccountId32',
        amount: 'u128',
      },
      RelayedTx: {
        caller: 'AccountId32',
        target: 'AccountId32',
        result: 'Result<Null, SpRuntimeDispatchError>'
      }
    }
  },
  /**
   * Lookup289: pallet_contracts::pallet::Event<T>
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
        depositHeld: 'u128',
        uploader: 'AccountId32',
      },
      ContractEmitted: {
        contract: 'AccountId32',
        data: 'Bytes',
      },
      CodeRemoved: {
        codeHash: 'H256',
        depositReleased: 'u128',
        remover: 'AccountId32',
      },
      ContractCodeUpdated: {
        contract: 'AccountId32',
        newCodeHash: 'H256',
        oldCodeHash: 'H256',
      },
      Called: {
        caller: 'PalletContractsOrigin',
        contract: 'AccountId32',
      },
      DelegateCalled: {
        contract: 'AccountId32',
        codeHash: 'H256',
      },
      StorageDepositTransferredAndHeld: {
        from: 'AccountId32',
        to: 'AccountId32',
        amount: 'u128',
      },
      StorageDepositTransferredAndReleased: {
        from: 'AccountId32',
        to: 'AccountId32',
        amount: 'u128'
      }
    }
  },
  /**
   * Lookup290: pallet_contracts::Origin<polymesh_runtime_develop::runtime::Runtime>
   **/
  PalletContractsOrigin: {
    _enum: {
      Root: 'Null',
      Signed: 'AccountId32'
    }
  },
  /**
   * Lookup291: polymesh_runtime_develop::runtime::Runtime
   **/
  PolymeshRuntimeDevelopRuntime: 'Null',
  /**
   * Lookup292: polymesh_contracts::pallet::Event<T>
   **/
  PolymeshContractsEvent: {
    _enum: {
      ApiHashUpdated: '(PolymeshContractsApi,PolymeshContractsChainVersion,H256)',
      SCRuntimeCall: '(AccountId32,PolymeshContractsChainExtensionExtrinsicId)'
    }
  },
  /**
   * Lookup293: polymesh_contracts::Api
   **/
  PolymeshContractsApi: {
    desc: '[u8;4]',
    major: 'u32'
  },
  /**
   * Lookup294: polymesh_contracts::ChainVersion
   **/
  PolymeshContractsChainVersion: {
    specVersion: 'u32',
    txVersion: 'u32'
  },
  /**
   * Lookup295: polymesh_contracts::chain_extension::ExtrinsicId
   **/
  PolymeshContractsChainExtensionExtrinsicId: '(u8,u8)',
  /**
   * Lookup296: pallet_preimage::pallet::Event<T>
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
   * Lookup297: pallet_nft::pallet::Event<T>
   **/
  PalletNftEvent: {
    _enum: {
      NftCollectionCreated: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesAssetAssetId,u64)',
      NFTHoldingsUpdated: '(PolymeshPrimitivesIdentityId,PolymeshPrimitivesNftNfTs,Option<PolymeshPrimitivesAssetAssetHolder>,Option<PolymeshPrimitivesAssetAssetHolder>,PolymeshPrimitivesAssetHoldingsUpdateReason)'
    }
  },
  /**
   * Lookup299: pallet_election_provider_multi_phase::pallet::Event<T>
   **/
  PalletElectionProviderMultiPhaseEvent: {
    _enum: {
      SolutionStored: {
        compute: 'PalletElectionProviderMultiPhaseElectionCompute',
        origin: 'Option<AccountId32>',
        prevEjected: 'bool',
      },
      ElectionFinalized: {
        compute: 'PalletElectionProviderMultiPhaseElectionCompute',
        score: 'SpNposElectionsElectionScore',
      },
      ElectionFailed: 'Null',
      Rewarded: {
        account: 'AccountId32',
        value: 'u128',
      },
      Slashed: {
        account: 'AccountId32',
        value: 'u128',
      },
      PhaseTransitioned: {
        from: 'PalletElectionProviderMultiPhasePhase',
        to: 'PalletElectionProviderMultiPhasePhase',
        round: 'u32'
      }
    }
  },
  /**
   * Lookup300: pallet_election_provider_multi_phase::ElectionCompute
   **/
  PalletElectionProviderMultiPhaseElectionCompute: {
    _enum: ['OnChain', 'Signed', 'Unsigned', 'Fallback', 'Emergency']
  },
  /**
   * Lookup301: sp_npos_elections::ElectionScore
   **/
  SpNposElectionsElectionScore: {
    minimalStake: 'u128',
    sumStake: 'u128',
    sumStakeSquared: 'u128'
  },
  /**
   * Lookup302: pallet_election_provider_multi_phase::Phase<Bn>
   **/
  PalletElectionProviderMultiPhasePhase: {
    _enum: {
      Off: 'Null',
      Signed: 'Null',
      Unsigned: '(bool,u32)',
      Emergency: 'Null'
    }
  },
  /**
   * Lookup304: frame_system::Phase
   **/
  FrameSystemPhase: {
    _enum: {
      ApplyExtrinsic: 'u32',
      Finalization: 'Null',
      Initialization: 'Null'
    }
  },
  /**
   * Lookup307: frame_system::LastRuntimeUpgradeInfo
   **/
  FrameSystemLastRuntimeUpgradeInfo: {
    specVersion: 'Compact<u32>',
    specName: 'Text'
  },
  /**
   * Lookup310: frame_system::CodeUpgradeAuthorization<T>
   **/
  FrameSystemCodeUpgradeAuthorization: {
    codeHash: 'H256',
    checkVersion: 'bool'
  },
  /**
   * Lookup311: frame_system::pallet::Call<T>
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
        remark: 'Bytes',
      },
      __Unused8: 'Null',
      authorize_upgrade: {
        codeHash: 'H256',
      },
      authorize_upgrade_without_checks: {
        codeHash: 'H256',
      },
      apply_authorized_upgrade: {
        code: 'Bytes'
      }
    }
  },
  /**
   * Lookup315: frame_system::limits::BlockWeights
   **/
  FrameSystemLimitsBlockWeights: {
    baseBlock: 'SpWeightsWeightV2Weight',
    maxBlock: 'SpWeightsWeightV2Weight',
    perClass: 'FrameSupportDispatchPerDispatchClassWeightsPerClass'
  },
  /**
   * Lookup316: frame_support::dispatch::PerDispatchClass<frame_system::limits::WeightsPerClass>
   **/
  FrameSupportDispatchPerDispatchClassWeightsPerClass: {
    normal: 'FrameSystemLimitsWeightsPerClass',
    operational: 'FrameSystemLimitsWeightsPerClass',
    mandatory: 'FrameSystemLimitsWeightsPerClass'
  },
  /**
   * Lookup317: frame_system::limits::WeightsPerClass
   **/
  FrameSystemLimitsWeightsPerClass: {
    baseExtrinsic: 'SpWeightsWeightV2Weight',
    maxExtrinsic: 'Option<SpWeightsWeightV2Weight>',
    maxTotal: 'Option<SpWeightsWeightV2Weight>',
    reserved: 'Option<SpWeightsWeightV2Weight>'
  },
  /**
   * Lookup319: frame_system::limits::BlockLength
   **/
  FrameSystemLimitsBlockLength: {
    max: 'FrameSupportDispatchPerDispatchClassU32'
  },
  /**
   * Lookup320: frame_support::dispatch::PerDispatchClass<T>
   **/
  FrameSupportDispatchPerDispatchClassU32: {
    normal: 'u32',
    operational: 'u32',
    mandatory: 'u32'
  },
  /**
   * Lookup321: sp_weights::RuntimeDbWeight
   **/
  SpWeightsRuntimeDbWeight: {
    read: 'u64',
    write: 'u64'
  },
  /**
   * Lookup322: sp_version::RuntimeVersion
   **/
  SpVersionRuntimeVersion: {
    specName: 'Text',
    implName: 'Text',
    authoringVersion: 'u32',
    specVersion: 'u32',
    implVersion: 'u32',
    apis: 'Vec<([u8;8],u32)>',
    transactionVersion: 'u32',
    systemVersion: 'u8'
  },
  /**
   * Lookup327: frame_system::pallet::Error<T>
   **/
  FrameSystemError: {
    _enum: ['InvalidSpecName', 'SpecVersionNeedsToIncrease', 'FailedToExtractRuntimeVersion', 'NonDefaultComposite', 'NonZeroRefCount', 'CallFiltered', 'MultiBlockMigrationsOngoing', 'NothingAuthorized', 'Unauthorized']
  },
  /**
   * Lookup330: sp_consensus_babe::app::Public
   **/
  SpConsensusBabeAppPublic: '[u8;32]',
  /**
   * Lookup333: sp_consensus_babe::digests::NextConfigDescriptor
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
   * Lookup335: sp_consensus_babe::AllowedSlots
   **/
  SpConsensusBabeAllowedSlots: {
    _enum: ['PrimarySlots', 'PrimaryAndSecondaryPlainSlots', 'PrimaryAndSecondaryVRFSlots']
  },
  /**
   * Lookup339: sp_consensus_babe::digests::PreDigest
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
   * Lookup340: sp_consensus_babe::digests::PrimaryPreDigest
   **/
  SpConsensusBabeDigestsPrimaryPreDigest: {
    authorityIndex: 'u32',
    slot: 'u64',
    vrfSignature: 'SpCoreSr25519VrfVrfSignature'
  },
  /**
   * Lookup341: sp_core::sr25519::vrf::VrfSignature
   **/
  SpCoreSr25519VrfVrfSignature: {
    preOutput: '[u8;32]',
    proof: '[u8;64]'
  },
  /**
   * Lookup342: sp_consensus_babe::digests::SecondaryPlainPreDigest
   **/
  SpConsensusBabeDigestsSecondaryPlainPreDigest: {
    authorityIndex: 'u32',
    slot: 'u64'
  },
  /**
   * Lookup343: sp_consensus_babe::digests::SecondaryVRFPreDigest
   **/
  SpConsensusBabeDigestsSecondaryVRFPreDigest: {
    authorityIndex: 'u32',
    slot: 'u64',
    vrfSignature: 'SpCoreSr25519VrfVrfSignature'
  },
  /**
   * Lookup344: sp_consensus_babe::BabeEpochConfiguration
   **/
  SpConsensusBabeBabeEpochConfiguration: {
    c: '(u64,u64)',
    allowedSlots: 'SpConsensusBabeAllowedSlots'
  },
  /**
   * Lookup348: pallet_babe::pallet::Call<T>
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
   * Lookup349: sp_consensus_slots::EquivocationProof<sp_runtime::generic::header::Header<Number, Hash>, sp_consensus_babe::app::Public>
   **/
  SpConsensusSlotsEquivocationProof: {
    offender: 'SpConsensusBabeAppPublic',
    slot: 'u64',
    firstHeader: 'SpRuntimeHeader',
    secondHeader: 'SpRuntimeHeader'
  },
  /**
   * Lookup350: sp_runtime::generic::header::Header<Number, Hash>
   **/
  SpRuntimeHeader: {
    parentHash: 'H256',
    number: 'Compact<u32>',
    stateRoot: 'H256',
    extrinsicsRoot: 'H256',
    digest: 'SpRuntimeDigest'
  },
  /**
   * Lookup351: sp_session::MembershipProof
   **/
  SpSessionMembershipProof: {
    session: 'u32',
    trieNodes: 'Vec<Bytes>',
    validatorCount: 'u32'
  },
  /**
   * Lookup352: pallet_babe::pallet::Error<T>
   **/
  PalletBabeError: {
    _enum: ['InvalidEquivocationProof', 'InvalidKeyOwnershipProof', 'DuplicateOffenceReport', 'InvalidConfiguration']
  },
  /**
   * Lookup353: pallet_timestamp::pallet::Call<T>
   **/
  PalletTimestampCall: {
    _enum: {
      set: {
        now: 'Compact<u64>'
      }
    }
  },
  /**
   * Lookup355: pallet_indices::pallet::Call<T>
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
        index: 'u32',
      },
      poke_deposit: {
        index: 'u32'
      }
    }
  },
  /**
   * Lookup357: pallet_indices::pallet::Error<T>
   **/
  PalletIndicesError: {
    _enum: ['NotAssigned', 'NotOwner', 'InUse', 'NotTransfer', 'Permanent']
  },
  /**
   * Lookup359: pallet_balances::types::BalanceLock<Balance>
   **/
  PalletBalancesBalanceLock: {
    id: '[u8;8]',
    amount: 'u128',
    reasons: 'PalletBalancesReasons'
  },
  /**
   * Lookup360: pallet_balances::types::Reasons
   **/
  PalletBalancesReasons: {
    _enum: ['Fee', 'Misc', 'All']
  },
  /**
   * Lookup363: pallet_balances::types::ReserveData<ReserveIdentifier, Balance>
   **/
  PalletBalancesReserveData: {
    id: '[u8;8]',
    amount: 'u128'
  },
  /**
   * Lookup369: frame_support::traits::tokens::misc::IdAmount<Id, Balance>
   **/
  FrameSupportTokensMiscIdAmount: {
    id: '[u8;8]',
    amount: 'u128'
  },
  /**
   * Lookup371: pallet_balances::pallet::Call<T, I>
   **/
  PalletBalancesCall: {
    _enum: {
      transfer_allow_death: {
        dest: 'MultiAddress',
        value: 'Compact<u128>',
      },
      __Unused1: 'Null',
      force_transfer: {
        source: 'MultiAddress',
        dest: 'MultiAddress',
        value: 'Compact<u128>',
      },
      transfer_keep_alive: {
        dest: 'MultiAddress',
        value: 'Compact<u128>',
      },
      transfer_all: {
        dest: 'MultiAddress',
        keepAlive: 'bool',
      },
      force_unreserve: {
        who: 'MultiAddress',
        amount: 'u128',
      },
      upgrade_accounts: {
        who: 'Vec<AccountId32>',
      },
      __Unused7: 'Null',
      force_set_balance: {
        who: 'MultiAddress',
        newFree: 'Compact<u128>',
      },
      force_adjust_total_issuance: {
        direction: 'PalletBalancesAdjustmentDirection',
        delta: 'Compact<u128>',
      },
      burn: {
        value: 'Compact<u128>',
        keepAlive: 'bool',
      },
      __Unused11: 'Null',
      __Unused12: 'Null',
      __Unused13: 'Null',
      __Unused14: 'Null',
      __Unused15: 'Null',
      __Unused16: 'Null',
      __Unused17: 'Null',
      __Unused18: 'Null',
      __Unused19: 'Null',
      __Unused20: 'Null',
      __Unused21: 'Null',
      __Unused22: 'Null',
      __Unused23: 'Null',
      __Unused24: 'Null',
      __Unused25: 'Null',
      __Unused26: 'Null',
      __Unused27: 'Null',
      __Unused28: 'Null',
      __Unused29: 'Null',
      __Unused30: 'Null',
      __Unused31: 'Null',
      __Unused32: 'Null',
      __Unused33: 'Null',
      __Unused34: 'Null',
      __Unused35: 'Null',
      __Unused36: 'Null',
      __Unused37: 'Null',
      __Unused38: 'Null',
      __Unused39: 'Null',
      transfer_with_memo: {
        dest: 'MultiAddress',
        value: 'Compact<u128>',
        memo: 'Option<PolymeshPrimitivesMemo>'
      }
    }
  },
  /**
   * Lookup372: pallet_balances::types::AdjustmentDirection
   **/
  PalletBalancesAdjustmentDirection: {
    _enum: ['Increase', 'Decrease']
  },
  /**
   * Lookup373: pallet_balances::pallet::Error<T, I>
   **/
  PalletBalancesError: {
    _enum: ['VestingBalance', 'LiquidityRestrictions', 'InsufficientBalance', 'ExistentialDeposit', 'Expendability', 'ExistingVestingSchedule', 'DeadAccount', 'TooManyReserves', 'TooManyHolds', 'TooManyFreezes', 'IssuanceDeactivated', 'DeltaZero', 'LockIdentifierNotFound', 'Overflow', 'MaxLocksExceeded']
  },
  /**
   * Lookup375: pallet_transaction_payment::Releases
   **/
  PalletTransactionPaymentReleases: {
    _enum: ['V1Ancient', 'V2']
  },
  /**
   * Lookup376: frame_support::traits::storage::NoDrop<frame_support::traits::tokens::fungible::imbalance::Imbalance<B, OnDrop, OppositeOnDrop>>
   **/
  FrameSupportStorageNoDrop: 'FrameSupportTokensFungibleImbalance',
  /**
   * Lookup377: frame_support::traits::tokens::fungible::imbalance::Imbalance<B, OnDrop, OppositeOnDrop>
   **/
  FrameSupportTokensFungibleImbalance: {
    amount: 'u128'
  },
  /**
   * Lookup378: polymesh_primitives::identity::DidRecord<sp_core::crypto::AccountId32>
   **/
  PolymeshPrimitivesIdentityDidRecord: {
    primaryKey: 'Option<AccountId32>'
  },
  /**
   * Lookup380: pallet_identity::types::Claim1stKey
   **/
  PalletIdentityClaim1stKey: {
    target: 'PolymeshPrimitivesIdentityId',
    claimType: 'PolymeshPrimitivesIdentityClaimClaimType'
  },
  /**
   * Lookup381: pallet_identity::types::Claim2ndKey
   **/
  PalletIdentityClaim2ndKey: {
    issuer: 'PolymeshPrimitivesIdentityId',
    scope: 'Option<PolymeshPrimitivesIdentityClaimScope>'
  },
  /**
   * Lookup382: polymesh_primitives::secondary_key::KeyRecord<sp_core::crypto::AccountId32>
   **/
  PolymeshPrimitivesSecondaryKeyKeyRecord: {
    _enum: {
      PrimaryKey: 'PolymeshPrimitivesIdentityId',
      SecondaryKey: 'PolymeshPrimitivesIdentityId',
      MultiSigSignerKey: 'AccountId32'
    }
  },
  /**
   * Lookup385: polymesh_primitives::secondary_key::Signatory<sp_core::crypto::AccountId32>
   **/
  PolymeshPrimitivesSecondaryKeySignatory: {
    _enum: {
      Identity: 'PolymeshPrimitivesIdentityId',
      Account: 'AccountId32'
    }
  },
  /**
   * Lookup386: polymesh_primitives::authorization::Authorization<sp_core::crypto::AccountId32, Moment>
   **/
  PolymeshPrimitivesAuthorization: {
    authorizationData: 'PolymeshPrimitivesAuthorizationAuthorizationData',
    authorizedBy: 'PolymeshPrimitivesIdentityId',
    expiry: 'Option<u64>',
    authId: 'u64',
    count: 'u32'
  },
  /**
   * Lookup390: pallet_identity::pallet::Call<T>
   **/
  PalletIdentityCall: {
    _enum: {
      cdd_register_did: {
        targetAccount: 'AccountId32',
        secondaryKeys: 'Vec<PolymeshPrimitivesSecondaryKey>',
      },
      __Unused1: 'Null',
      accept_primary_key: {
        rotationAuthId: 'u64',
      },
      __Unused3: 'Null',
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
      gc_add_cdd_claim: {
        target: 'PolymeshPrimitivesIdentityId',
      },
      gc_revoke_cdd_claim: {
        target: 'PolymeshPrimitivesIdentityId',
      },
      revoke_claim_by_index: {
        target: 'PolymeshPrimitivesIdentityId',
        claimType: 'PolymeshPrimitivesIdentityClaimClaimType',
        scope: 'Option<PolymeshPrimitivesIdentityClaimScope>',
      },
      rotate_primary_key_to_secondary: {
        authId: 'u64',
      },
      add_secondary_keys_with_authorization: {
        additionalKeys: 'Vec<PolymeshPrimitivesIdentitySecondaryKeyWithAuth>',
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
        childKeys: 'Vec<PolymeshPrimitivesIdentityCreateChildIdentityWithAuth>',
        expiresAt: 'u64',
      },
      unlink_child_identity: {
        childDid: 'PolymeshPrimitivesIdentityId',
      },
      register_did: {
        targetAccount: 'AccountId32'
      }
    }
  },
  /**
   * Lookup392: polymesh_primitives::identity::SecondaryKeyWithAuth<sp_core::crypto::AccountId32>
   **/
  PolymeshPrimitivesIdentitySecondaryKeyWithAuth: {
    secondaryKey: 'PolymeshPrimitivesSecondaryKey',
    authSignature: 'H512'
  },
  /**
   * Lookup395: polymesh_primitives::identity::CreateChildIdentityWithAuth<sp_core::crypto::AccountId32>
   **/
  PolymeshPrimitivesIdentityCreateChildIdentityWithAuth: {
    key: 'AccountId32',
    authSignature: 'H512'
  },
  /**
   * Lookup396: pallet_identity::pallet::Error<T>
   **/
  PalletIdentityError: {
    _enum: ['AlreadyLinked', 'MissingIdentity', 'Unauthorized', 'InvalidAccountKey', 'UnAuthorizedDidRegistrar', 'InvalidAuthorizationFromOwner', 'InvalidAuthorizationFromDidRegistrar', 'NotDidRegistrarAttestation', 'AuthorizationsNotForSameDids', 'DidMustAlreadyExist', 'AuthorizationExpired', 'TargetDidInactive', 'AuthorizationHasBeenRevoked', 'InvalidAuthorizationSignature', 'KeyNotAllowed', 'NotPrimaryKey', 'DidDoesNotExist', 'DidAlreadyExists', 'SecondaryKeysContainPrimaryKey', 'FailedToChargeFee', 'NotASigner', 'CannotDecodeSignerAccountId', 'AccountKeyIsBeingUsed', 'CustomScopeTooLong', 'CustomClaimTypeAlreadyExists', 'CustomClaimTypeDoesNotExist', 'ClaimDoesNotExist', 'IsChildIdentity', 'NoParentIdentity', 'NotParentOrChildIdentity', 'DuplicateKey', 'ExceptNotAllowedForExtrinsics', 'ExceededNumberOfGivenAuths', 'BadAuthorizationType', 'InvalidAuthorization', 'UnauthorizedCallerFrozenDid', 'UnauthorizedCallerDidInactive', 'UnauthorizedCallerMissingPermissions', 'IdentityNotFoundForAccountPortfolio']
  },
  /**
   * Lookup398: polymesh_primitives::traits::group::InactiveMember<Moment>
   **/
  PolymeshPrimitivesGroupInactiveMember: {
    id: 'PolymeshPrimitivesIdentityId',
    deactivatedAt: 'u64',
    expiry: 'Option<u64>'
  },
  /**
   * Lookup399: pallet_group::pallet::Call<T, I>
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
   * Lookup400: pallet_group::pallet::Error<T, I>
   **/
  PalletGroupError: {
    _enum: ['OnlyPrimaryKeyAllowed', 'DuplicateMember', 'NoSuchMember', 'LastMemberCannotQuit', 'ActiveMembersLimitExceeded', 'ActiveMembersLimitOverflow']
  },
  /**
   * Lookup402: pallet_committee::pallet::Call<T, I>
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
        expiry: 'PolymeshPrimitivesMaybeBlock',
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
   * Lookup408: pallet_multisig::pallet::Call<T>
   **/
  PalletMultisigCall: {
    _enum: {
      create_multisig: {
        signers: 'Vec<AccountId32>',
        sigsRequired: 'u64',
        permissions: 'Option<PolymeshPrimitivesSecondaryKeyPermissions>',
      },
      create_proposal: {
        multisig: 'AccountId32',
        proposal: 'Call',
        expiry: 'Option<u64>',
      },
      approve: {
        multisig: 'AccountId32',
        proposalId: 'u64',
        maxWeight: 'Option<SpWeightsWeightV2Weight>',
      },
      reject: {
        multisig: 'AccountId32',
        proposalId: 'u64',
      },
      accept_multisig_signer: {
        authId: 'u64',
      },
      add_multisig_signers: {
        signers: 'Vec<AccountId32>',
      },
      remove_multisig_signers: {
        signers: 'Vec<AccountId32>',
      },
      add_multisig_signers_via_admin: {
        multisig: 'AccountId32',
        signers: 'Vec<AccountId32>',
      },
      remove_multisig_signers_via_admin: {
        multisig: 'AccountId32',
        signers: 'Vec<AccountId32>',
      },
      change_sigs_required: {
        sigsRequired: 'u64',
      },
      change_sigs_required_via_admin: {
        multisig: 'AccountId32',
        signaturesRequired: 'u64',
      },
      add_admin: {
        adminDid: 'PolymeshPrimitivesIdentityId',
      },
      remove_admin_via_admin: {
        multisig: 'AccountId32',
      },
      remove_payer: 'Null',
      remove_payer_via_payer: {
        multisig: 'AccountId32',
      },
      approve_join_identity: {
        multisig: 'AccountId32',
        authId: 'u64',
      },
      join_identity: {
        authId: 'u64',
      },
      remove_admin: 'Null'
    }
  },
  /**
   * Lookup410: pallet_validators::pallet::Call<T>
   **/
  PalletValidatorsCall: {
    _enum: {
      add_permissioned_validator: {
        identity: 'PolymeshPrimitivesIdentityId',
        intendedCount: 'Option<u32>',
      },
      remove_permissioned_validator: {
        identity: 'PolymeshPrimitivesIdentityId',
      },
      payout_stakers_by_system: {
        validatorStash: 'AccountId32',
        era: 'u32',
      },
      change_slashing_allowed_for: {
        slashingSwitch: 'PalletValidatorsSlashingSwitch',
      },
      update_permissioned_validator_intended_count: {
        identity: 'PolymeshPrimitivesIdentityId',
        newIntendedCount: 'u32',
      },
      chill_from_governance: {
        identity: 'PolymeshPrimitivesIdentityId',
        stashKeys: 'Vec<AccountId32>',
      },
      set_commission_cap: {
        newCap: 'Perbill'
      }
    }
  },
  /**
   * Lookup411: pallet_staking::pallet::pallet::Call<T>
   **/
  PalletStakingPalletCall: {
    _enum: {
      bond: {
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
      set_controller: 'Null',
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
      reap_stash: {
        stash: 'AccountId32',
        numSlashingSpans: 'u32',
      },
      kick: {
        who: 'Vec<MultiAddress>',
      },
      set_staking_configs: {
        minNominatorBond: 'PalletStakingPalletConfigOpU128',
        minValidatorBond: 'PalletStakingPalletConfigOpU128',
        maxNominatorCount: 'PalletStakingPalletConfigOpU32',
        maxValidatorCount: 'PalletStakingPalletConfigOpU32',
        chillThreshold: 'PalletStakingPalletConfigOpPercent',
        minCommission: 'PalletStakingPalletConfigOpPerbill',
        maxStakedRewards: 'PalletStakingPalletConfigOpPercent',
      },
      chill_other: {
        stash: 'AccountId32',
      },
      force_apply_min_commission: {
        validatorStash: 'AccountId32',
      },
      set_min_commission: {
        _alias: {
          new_: 'new',
        },
        new_: 'Perbill',
      },
      payout_stakers_by_page: {
        validatorStash: 'AccountId32',
        era: 'u32',
        page: 'u32',
      },
      update_payee: {
        controller: 'AccountId32',
      },
      deprecate_controller_batch: {
        controllers: 'Vec<AccountId32>',
      },
      restore_ledger: {
        stash: 'AccountId32',
        maybeController: 'Option<AccountId32>',
        maybeTotal: 'Option<u128>',
        maybeUnlocking: 'Option<Vec<PalletStakingUnlockChunk>>',
      },
      migrate_currency: {
        stash: 'AccountId32',
      },
      __Unused31: 'Null',
      __Unused32: 'Null',
      manual_slash: {
        validatorStash: 'AccountId32',
        era: 'u32',
        slashFraction: 'Perbill'
      }
    }
  },
  /**
   * Lookup415: pallet_staking::pallet::pallet::ConfigOp<T>
   **/
  PalletStakingPalletConfigOpU128: {
    _enum: {
      Noop: 'Null',
      Set: 'u128',
      Remove: 'Null'
    }
  },
  /**
   * Lookup416: pallet_staking::pallet::pallet::ConfigOp<T>
   **/
  PalletStakingPalletConfigOpU32: {
    _enum: {
      Noop: 'Null',
      Set: 'u32',
      Remove: 'Null'
    }
  },
  /**
   * Lookup417: pallet_staking::pallet::pallet::ConfigOp<sp_arithmetic::per_things::Percent>
   **/
  PalletStakingPalletConfigOpPercent: {
    _enum: {
      Noop: 'Null',
      Set: 'Percent',
      Remove: 'Null'
    }
  },
  /**
   * Lookup418: pallet_staking::pallet::pallet::ConfigOp<sp_arithmetic::per_things::Perbill>
   **/
  PalletStakingPalletConfigOpPerbill: {
    _enum: {
      Noop: 'Null',
      Set: 'Perbill',
      Remove: 'Null'
    }
  },
  /**
   * Lookup422: pallet_staking::UnlockChunk<Balance>
   **/
  PalletStakingUnlockChunk: {
    value: 'Compact<u128>',
    era: 'Compact<u32>'
  },
  /**
   * Lookup424: pallet_session::pallet::Call<T>
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
   * Lookup425: polymesh_runtime_develop::runtime::SessionKeys
   **/
  PolymeshRuntimeDevelopRuntimeSessionKeys: {
    grandpa: 'SpConsensusGrandpaAppPublic',
    babe: 'SpConsensusBabeAppPublic',
    imOnline: 'PalletImOnlineSr25519AppSr25519Public',
    authorityDiscovery: 'SpAuthorityDiscoveryAppPublic',
    beefy: 'SpConsensusBeefyEcdsaCryptoPublic'
  },
  /**
   * Lookup426: sp_authority_discovery::app::Public
   **/
  SpAuthorityDiscoveryAppPublic: '[u8;32]',
  /**
   * Lookup427: sp_consensus_beefy::ecdsa_crypto::Public
   **/
  SpConsensusBeefyEcdsaCryptoPublic: '[u8;33]',
  /**
   * Lookup429: pallet_grandpa::pallet::Call<T>
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
   * Lookup430: sp_consensus_grandpa::EquivocationProof<primitive_types::H256, N>
   **/
  SpConsensusGrandpaEquivocationProof: {
    setId: 'u64',
    equivocation: 'SpConsensusGrandpaEquivocation'
  },
  /**
   * Lookup431: sp_consensus_grandpa::Equivocation<primitive_types::H256, N>
   **/
  SpConsensusGrandpaEquivocation: {
    _enum: {
      Prevote: 'FinalityGrandpaEquivocationPrevote',
      Precommit: 'FinalityGrandpaEquivocationPrecommit'
    }
  },
  /**
   * Lookup432: finality_grandpa::Equivocation<sp_consensus_grandpa::app::Public, finality_grandpa::Prevote<primitive_types::H256, N>, sp_consensus_grandpa::app::Signature>
   **/
  FinalityGrandpaEquivocationPrevote: {
    roundNumber: 'u64',
    identity: 'SpConsensusGrandpaAppPublic',
    first: '(FinalityGrandpaPrevote,SpConsensusGrandpaAppSignature)',
    second: '(FinalityGrandpaPrevote,SpConsensusGrandpaAppSignature)'
  },
  /**
   * Lookup433: finality_grandpa::Prevote<primitive_types::H256, N>
   **/
  FinalityGrandpaPrevote: {
    targetHash: 'H256',
    targetNumber: 'u32'
  },
  /**
   * Lookup434: sp_consensus_grandpa::app::Signature
   **/
  SpConsensusGrandpaAppSignature: '[u8;64]',
  /**
   * Lookup436: finality_grandpa::Equivocation<sp_consensus_grandpa::app::Public, finality_grandpa::Precommit<primitive_types::H256, N>, sp_consensus_grandpa::app::Signature>
   **/
  FinalityGrandpaEquivocationPrecommit: {
    roundNumber: 'u64',
    identity: 'SpConsensusGrandpaAppPublic',
    first: '(FinalityGrandpaPrecommit,SpConsensusGrandpaAppSignature)',
    second: '(FinalityGrandpaPrecommit,SpConsensusGrandpaAppSignature)'
  },
  /**
   * Lookup437: finality_grandpa::Precommit<primitive_types::H256, N>
   **/
  FinalityGrandpaPrecommit: {
    targetHash: 'H256',
    targetNumber: 'u32'
  },
  /**
   * Lookup439: pallet_im_online::pallet::Call<T>
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
   * Lookup440: pallet_im_online::Heartbeat<BlockNumber>
   **/
  PalletImOnlineHeartbeat: {
    blockNumber: 'u32',
    sessionIndex: 'u32',
    authorityIndex: 'u32',
    validatorsLen: 'u32'
  },
  /**
   * Lookup441: pallet_im_online::sr25519::app_sr25519::Signature
   **/
  PalletImOnlineSr25519AppSr25519Signature: '[u8;64]',
  /**
   * Lookup442: pallet_sudo::pallet::Call<T>
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
        call: 'Call',
      },
      remove_key: 'Null'
    }
  },
  /**
   * Lookup443: pallet_asset::pallet::Call<T>
   **/
  PalletAssetCall: {
    _enum: {
      register_unique_ticker: {
        ticker: 'PolymeshPrimitivesTicker',
      },
      accept_ticker_transfer: {
        authId: 'u64',
      },
      accept_asset_ownership_transfer: {
        authId: 'u64',
      },
      create_asset: {
        assetName: 'Bytes',
        divisible: 'bool',
        assetType: 'PolymeshPrimitivesAssetAssetType',
        assetIdentifiers: 'Vec<PolymeshPrimitivesAssetIdentifier>',
        fundingRoundName: 'Option<Bytes>',
      },
      freeze: {
        assetId: 'PolymeshPrimitivesAssetAssetId',
      },
      unfreeze: {
        assetId: 'PolymeshPrimitivesAssetAssetId',
      },
      rename_asset: {
        assetId: 'PolymeshPrimitivesAssetAssetId',
        assetName: 'Bytes',
      },
      issue: {
        assetId: 'PolymeshPrimitivesAssetAssetId',
        amount: 'u128',
        assetHolderKind: 'PolymeshPrimitivesAssetAssetHolderKind',
      },
      redeem: {
        assetId: 'PolymeshPrimitivesAssetAssetId',
        value: 'u128',
        assetHolderKind: 'PolymeshPrimitivesAssetAssetHolderKind',
      },
      make_divisible: {
        assetId: 'PolymeshPrimitivesAssetAssetId',
      },
      add_documents: {
        docs: 'Vec<PolymeshPrimitivesDocument>',
        assetId: 'PolymeshPrimitivesAssetAssetId',
      },
      remove_documents: {
        docsId: 'Vec<u32>',
        assetId: 'PolymeshPrimitivesAssetAssetId',
      },
      set_funding_round: {
        assetId: 'PolymeshPrimitivesAssetAssetId',
        fundingRoundName: 'Bytes',
      },
      update_identifiers: {
        assetId: 'PolymeshPrimitivesAssetAssetId',
        assetIdentifiers: 'Vec<PolymeshPrimitivesAssetIdentifier>',
      },
      controller_transfer: {
        assetId: 'PolymeshPrimitivesAssetAssetId',
        value: 'u128',
        assetHolder: 'PolymeshPrimitivesAssetAssetHolder',
      },
      register_custom_asset_type: {
        ty: 'Bytes',
      },
      create_asset_with_custom_type: {
        assetName: 'Bytes',
        divisible: 'bool',
        customAssetType: 'Bytes',
        assetIdentifiers: 'Vec<PolymeshPrimitivesAssetIdentifier>',
        fundingRoundName: 'Option<Bytes>',
      },
      set_asset_metadata: {
        assetId: 'PolymeshPrimitivesAssetAssetId',
        key: 'PolymeshPrimitivesAssetMetadataAssetMetadataKey',
        value: 'Bytes',
        detail: 'Option<PolymeshPrimitivesAssetMetadataAssetMetadataValueDetail>',
      },
      set_asset_metadata_details: {
        assetId: 'PolymeshPrimitivesAssetAssetId',
        key: 'PolymeshPrimitivesAssetMetadataAssetMetadataKey',
        detail: 'PolymeshPrimitivesAssetMetadataAssetMetadataValueDetail',
      },
      register_and_set_local_asset_metadata: {
        assetId: 'PolymeshPrimitivesAssetAssetId',
        name: 'Bytes',
        spec: 'PolymeshPrimitivesAssetMetadataAssetMetadataSpec',
        value: 'Bytes',
        detail: 'Option<PolymeshPrimitivesAssetMetadataAssetMetadataValueDetail>',
      },
      register_asset_metadata_local_type: {
        assetId: 'PolymeshPrimitivesAssetAssetId',
        name: 'Bytes',
        spec: 'PolymeshPrimitivesAssetMetadataAssetMetadataSpec',
      },
      register_asset_metadata_global_type: {
        name: 'Bytes',
        spec: 'PolymeshPrimitivesAssetMetadataAssetMetadataSpec',
      },
      update_asset_type: {
        assetId: 'PolymeshPrimitivesAssetAssetId',
        assetType: 'PolymeshPrimitivesAssetAssetType',
      },
      remove_local_metadata_key: {
        assetId: 'PolymeshPrimitivesAssetAssetId',
        localKey: 'u64',
      },
      remove_metadata_value: {
        assetId: 'PolymeshPrimitivesAssetAssetId',
        metadataKey: 'PolymeshPrimitivesAssetMetadataAssetMetadataKey',
      },
      exempt_asset_affirmation: {
        assetId: 'PolymeshPrimitivesAssetAssetId',
      },
      remove_asset_affirmation_exemption: {
        assetId: 'PolymeshPrimitivesAssetAssetId',
      },
      pre_approve_asset: {
        assetId: 'PolymeshPrimitivesAssetAssetId',
      },
      remove_asset_pre_approval: {
        assetId: 'PolymeshPrimitivesAssetAssetId',
      },
      add_mandatory_mediators: {
        assetId: 'PolymeshPrimitivesAssetAssetId',
        mediators: 'BTreeSet<PolymeshPrimitivesIdentityId>',
      },
      remove_mandatory_mediators: {
        assetId: 'PolymeshPrimitivesAssetAssetId',
        mediators: 'BTreeSet<PolymeshPrimitivesIdentityId>',
      },
      link_ticker_to_asset_id: {
        ticker: 'PolymeshPrimitivesTicker',
        assetId: 'PolymeshPrimitivesAssetAssetId',
      },
      unlink_ticker_from_asset_id: {
        ticker: 'PolymeshPrimitivesTicker',
        assetId: 'PolymeshPrimitivesAssetAssetId',
      },
      update_global_metadata_spec: {
        assetMetadataName: 'Bytes',
        assetMetadataSpec: 'PolymeshPrimitivesAssetMetadataAssetMetadataSpec',
      },
      transfer_asset: {
        assetId: 'PolymeshPrimitivesAssetAssetId',
        to: 'AccountId32',
        amount: 'u128',
        memo: 'Option<PolymeshPrimitivesMemo>',
      },
      receiver_affirm_asset_transfer: {
        transferId: 'u64',
      },
      reject_asset_transfer: {
        transferId: 'u64',
      },
      set_mandatory_receiver_affirmation: {
        require: 'bool'
      }
    }
  },
  /**
   * Lookup444: polymesh_primitives::asset::AssetHolderKind
   **/
  PolymeshPrimitivesAssetAssetHolderKind: {
    _enum: {
      Account: 'Null',
      DefaultPortfolio: 'Null',
      UserPortfolio: 'u64'
    }
  },
  /**
   * Lookup447: pallet_corporate_actions::distribution::pallet::Call<T>
   **/
  PalletCorporateActionsDistributionPalletCall: {
    _enum: {
      distribute: {
        caId: 'PalletCorporateActionsCaId',
        portfolio: 'Option<u64>',
        currency: 'PolymeshPrimitivesAssetAssetId',
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
   * Lookup449: pallet_asset::checkpoint::pallet::Call<T>
   **/
  PalletAssetCheckpointPalletCall: {
    _enum: {
      create_checkpoint: {
        assetId: 'PolymeshPrimitivesAssetAssetId',
      },
      set_schedules_max_complexity: {
        maxComplexity: 'u64',
      },
      create_schedule: {
        assetId: 'PolymeshPrimitivesAssetAssetId',
        schedule: 'PolymeshPrimitivesCheckpointScheduleCheckpoints',
      },
      remove_schedule: {
        assetId: 'PolymeshPrimitivesAssetAssetId',
        id: 'u64'
      }
    }
  },
  /**
   * Lookup450: pallet_compliance_manager::pallet::Call<T>
   **/
  PalletComplianceManagerCall: {
    _enum: {
      add_compliance_requirement: {
        assetId: 'PolymeshPrimitivesAssetAssetId',
        senderConditions: 'Vec<PolymeshPrimitivesCondition>',
        receiverConditions: 'Vec<PolymeshPrimitivesCondition>',
      },
      remove_compliance_requirement: {
        assetId: 'PolymeshPrimitivesAssetAssetId',
        id: 'u32',
      },
      replace_asset_compliance: {
        assetId: 'PolymeshPrimitivesAssetAssetId',
        assetCompliance: 'Vec<PolymeshPrimitivesComplianceManagerComplianceRequirement>',
      },
      reset_asset_compliance: {
        assetId: 'PolymeshPrimitivesAssetAssetId',
      },
      pause_asset_compliance: {
        assetId: 'PolymeshPrimitivesAssetAssetId',
      },
      resume_asset_compliance: {
        assetId: 'PolymeshPrimitivesAssetAssetId',
      },
      add_default_trusted_claim_issuer: {
        assetId: 'PolymeshPrimitivesAssetAssetId',
        issuer: 'PolymeshPrimitivesConditionTrustedIssuer',
      },
      remove_default_trusted_claim_issuer: {
        assetId: 'PolymeshPrimitivesAssetAssetId',
        issuer: 'PolymeshPrimitivesIdentityId',
      },
      change_compliance_requirement: {
        assetId: 'PolymeshPrimitivesAssetAssetId',
        newReq: 'PolymeshPrimitivesComplianceManagerComplianceRequirement'
      }
    }
  },
  /**
   * Lookup451: pallet_corporate_actions::pallet::Call<T>
   **/
  PalletCorporateActionsCall: {
    _enum: {
      set_max_details_length: {
        length: 'u32',
      },
      set_default_targets: {
        assetId: 'PolymeshPrimitivesAssetAssetId',
        targets: 'PalletCorporateActionsTargetIdentities',
      },
      set_default_withholding_tax: {
        assetId: 'PolymeshPrimitivesAssetAssetId',
        tax: 'Permill',
      },
      set_did_withholding_tax: {
        assetId: 'PolymeshPrimitivesAssetAssetId',
        taxedDid: 'PolymeshPrimitivesIdentityId',
        tax: 'Option<Permill>',
      },
      initiate_corporate_action: {
        assetId: 'PolymeshPrimitivesAssetAssetId',
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
        currency: 'PolymeshPrimitivesAssetAssetId',
        perShare: 'u128',
        amount: 'u128',
        paymentAt: 'u64',
        expiresAt: 'Option<u64>',
      },
      initiate_corporate_action_and_ballot: {
        caArgs: 'PalletCorporateActionsInitiateCorporateActionArgs',
        ballotTimeRange: 'PalletCorporateActionsBallotBallotTimeRange',
        ballotMeta: 'PalletCorporateActionsBallotBallotMeta',
        rcv: 'bool'
      }
    }
  },
  /**
   * Lookup453: pallet_corporate_actions::RecordDateSpec
   **/
  PalletCorporateActionsRecordDateSpec: {
    _enum: {
      Scheduled: 'u64',
      ExistingSchedule: 'u64',
      Existing: 'u64'
    }
  },
  /**
   * Lookup456: pallet_corporate_actions::InitiateCorporateActionArgs
   **/
  PalletCorporateActionsInitiateCorporateActionArgs: {
    assetId: 'PolymeshPrimitivesAssetAssetId',
    kind: 'PalletCorporateActionsCaKind',
    declDate: 'u64',
    recordDate: 'Option<PalletCorporateActionsRecordDateSpec>',
    details: 'Bytes',
    targets: 'Option<PalletCorporateActionsTargetIdentities>',
    defaultWithholdingTax: 'Option<Permill>',
    withholdingTax: 'Option<Vec<(PolymeshPrimitivesIdentityId,Permill)>>'
  },
  /**
   * Lookup457: pallet_corporate_actions::ballot::pallet::Call<T>
   **/
  PalletCorporateActionsBallotPalletCall: {
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
   * Lookup458: pallet_pips::pallet::Call<T>
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
        expiry: 'PolymeshPrimitivesMaybeBlock',
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
   * Lookup461: pallet_pips::types::SnapshotResult
   **/
  PalletPipsSnapshotResult: {
    _enum: ['Approve', 'Reject', 'Skip']
  },
  /**
   * Lookup462: pallet_portfolio::pallet::Call<T>
   **/
  PalletPortfolioCall: {
    _enum: {
      create_portfolio: {
        name: 'Bytes',
      },
      delete_portfolio: {
        portfolioNumber: 'u64',
      },
      rename_portfolio: {
        portfolioNumber: 'u64',
        newPortfolioName: 'Bytes',
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
        assetId: 'PolymeshPrimitivesAssetAssetId',
        portfolioId: 'PolymeshPrimitivesIdentityIdPortfolioId',
      },
      remove_portfolio_pre_approval: {
        assetId: 'PolymeshPrimitivesAssetAssetId',
        portfolioId: 'PolymeshPrimitivesIdentityIdPortfolioId',
      },
      allow_identity_to_create_portfolios: {
        trustedIdentity: 'PolymeshPrimitivesIdentityId',
      },
      revoke_create_portfolios_permission: {
        identity: 'PolymeshPrimitivesIdentityId',
      },
      create_custody_portfolio: {
        portfolioOwnerId: 'PolymeshPrimitivesIdentityId',
        portfolioName: 'Bytes'
      }
    }
  },
  /**
   * Lookup464: polymesh_primitives::portfolio::Fund
   **/
  PolymeshPrimitivesPortfolioFund: {
    description: 'PolymeshPrimitivesPortfolioFundDescription',
    memo: 'Option<PolymeshPrimitivesMemo>'
  },
  /**
   * Lookup465: pallet_protocol_fee::pallet::Call<T>
   **/
  PalletProtocolFeeCall: {
    _enum: {
      change_coefficient: {
        coefficient: 'PolymeshPrimitivesPosRatio',
      },
      change_base_fee: {
        op: 'PolymeshPrimitivesProtocolFeeProtocolOp',
        baseFee: 'u128'
      }
    }
  },
  /**
   * Lookup466: polymesh_primitives::protocol_fee::ProtocolOp
   **/
  PolymeshPrimitivesProtocolFeeProtocolOp: {
    _enum: ['AssetRegisterTicker', 'AssetIssue', 'AssetAddDocuments', 'AssetCreateAsset', 'CheckpointCreateSchedule', 'ComplianceManagerAddComplianceRequirement', 'IdentityCddRegisterDid', 'IdentityAddClaim', 'IdentityAddSecondaryKeysWithAuthorization', 'PipsPropose', 'ContractsPutCode', 'CorporateBallotAttachBallot', 'CapitalDistributionDistribute', 'NFTCreateCollection', 'NFTMint', 'IdentityCreateChildIdentity']
  },
  /**
   * Lookup467: pallet_scheduler::pallet::Call<T>
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
        call: 'Call',
      },
      set_retry: {
        task: '(u32,u32)',
        retries: 'u8',
        period: 'u32',
      },
      set_retry_named: {
        id: '[u8;32]',
        retries: 'u8',
        period: 'u32',
      },
      cancel_retry: {
        task: '(u32,u32)',
      },
      cancel_retry_named: {
        id: '[u8;32]'
      }
    }
  },
  /**
   * Lookup469: pallet_settlement::pallet::Call<T>
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
      affirm_with_receipts: {
        id: 'u64',
        receiptDetails: 'Vec<PolymeshPrimitivesSettlementReceiptDetails>',
        holderSet: 'BTreeSet<PolymeshPrimitivesAssetAssetHolder>',
      },
      set_venue_filtering: {
        assetId: 'PolymeshPrimitivesAssetAssetId',
        enabled: 'bool',
      },
      allow_venues: {
        assetId: 'PolymeshPrimitivesAssetAssetId',
        venues: 'Vec<u64>',
      },
      disallow_venues: {
        assetId: 'PolymeshPrimitivesAssetAssetId',
        venues: 'Vec<u64>',
      },
      update_venue_signers: {
        id: 'u64',
        signers: 'Vec<AccountId32>',
        addSigners: 'bool',
      },
      execute_manual_instruction: {
        id: 'u64',
        assetHolder: 'Option<PolymeshPrimitivesAssetAssetHolder>',
        fungibleTransfers: 'u32',
        nftsTransfers: 'u32',
        offchainTransfers: 'u32',
        weightLimit: 'Option<SpWeightsWeightV2Weight>',
      },
      add_instruction: {
        venueId: 'Option<u64>',
        settlementType: 'PolymeshPrimitivesSettlementSettlementType',
        tradeDate: 'Option<u64>',
        valueDate: 'Option<u64>',
        legs: 'Vec<PolymeshPrimitivesSettlementLeg>',
        instructionMemo: 'Option<PolymeshPrimitivesMemo>',
      },
      add_and_affirm_instruction: {
        venueId: 'Option<u64>',
        settlementType: 'PolymeshPrimitivesSettlementSettlementType',
        tradeDate: 'Option<u64>',
        valueDate: 'Option<u64>',
        legs: 'Vec<PolymeshPrimitivesSettlementLeg>',
        holderSet: 'BTreeSet<PolymeshPrimitivesAssetAssetHolder>',
        instructionMemo: 'Option<PolymeshPrimitivesMemo>',
      },
      affirm_instruction: {
        id: 'u64',
        holderSet: 'BTreeSet<PolymeshPrimitivesAssetAssetHolder>',
      },
      withdraw_affirmation: {
        id: 'u64',
        holderSet: 'BTreeSet<PolymeshPrimitivesAssetAssetHolder>',
      },
      reject_instruction: {
        id: 'u64',
        assetHolder: 'PolymeshPrimitivesAssetAssetHolder',
      },
      execute_scheduled_instruction: {
        id: 'u64',
        weightLimit: 'SpWeightsWeightV2Weight',
      },
      affirm_with_receipts_with_count: {
        id: 'u64',
        receiptDetails: 'Vec<PolymeshPrimitivesSettlementReceiptDetails>',
        holderSet: 'BTreeSet<PolymeshPrimitivesAssetAssetHolder>',
        numberOfAssets: 'Option<PolymeshPrimitivesSettlementAffirmationCount>',
      },
      affirm_instruction_with_count: {
        id: 'u64',
        holderSet: 'BTreeSet<PolymeshPrimitivesAssetAssetHolder>',
        numberOfAssets: 'Option<PolymeshPrimitivesSettlementAffirmationCount>',
      },
      reject_instruction_with_count: {
        id: 'u64',
        assetHolder: 'PolymeshPrimitivesAssetAssetHolder',
        numberOfAssets: 'Option<PolymeshPrimitivesSettlementAssetCount>',
      },
      withdraw_affirmation_with_count: {
        id: 'u64',
        holderSet: 'BTreeSet<PolymeshPrimitivesAssetAssetHolder>',
        numberOfAssets: 'Option<PolymeshPrimitivesSettlementAffirmationCount>',
      },
      add_instruction_with_mediators: {
        venueId: 'Option<u64>',
        settlementType: 'PolymeshPrimitivesSettlementSettlementType',
        tradeDate: 'Option<u64>',
        valueDate: 'Option<u64>',
        legs: 'Vec<PolymeshPrimitivesSettlementLeg>',
        instructionMemo: 'Option<PolymeshPrimitivesMemo>',
        mediators: 'BTreeSet<PolymeshPrimitivesIdentityId>',
      },
      add_and_affirm_with_mediators: {
        venueId: 'Option<u64>',
        settlementType: 'PolymeshPrimitivesSettlementSettlementType',
        tradeDate: 'Option<u64>',
        valueDate: 'Option<u64>',
        legs: 'Vec<PolymeshPrimitivesSettlementLeg>',
        holderSet: 'BTreeSet<PolymeshPrimitivesAssetAssetHolder>',
        instructionMemo: 'Option<PolymeshPrimitivesMemo>',
        mediators: 'BTreeSet<PolymeshPrimitivesIdentityId>',
      },
      affirm_instruction_as_mediator: {
        instructionId: 'u64',
        expiry: 'Option<u64>',
      },
      withdraw_affirmation_as_mediator: {
        instructionId: 'u64',
      },
      reject_instruction_as_mediator: {
        instructionId: 'u64',
        numberOfAssets: 'Option<PolymeshPrimitivesSettlementAssetCount>',
      },
      lock_instruction: {
        instId: 'u64',
        weightLimit: 'SpWeightsWeightV2Weight'
      }
    }
  },
  /**
   * Lookup471: polymesh_primitives::settlement::ReceiptDetails<sp_core::crypto::AccountId32, sp_runtime::MultiSignature, Moment>
   **/
  PolymeshPrimitivesSettlementReceiptDetails: {
    uid: 'u64',
    instructionId: 'u64',
    legId: 'u64',
    signer: 'AccountId32',
    signature: 'SpRuntimeMultiSignature',
    expiresAt: 'u64',
    metadata: 'Option<PolymeshPrimitivesSettlementReceiptMetadata>'
  },
  /**
   * Lookup472: sp_runtime::MultiSignature
   **/
  SpRuntimeMultiSignature: {
    _enum: {
      Ed25519: '[u8;64]',
      Sr25519: '[u8;64]',
      Ecdsa: '[u8;65]',
      Eth: '[u8;65]'
    }
  },
  /**
   * Lookup478: polymesh_primitives::settlement::AffirmationCount
   **/
  PolymeshPrimitivesSettlementAffirmationCount: {
    senderAssetCount: 'PolymeshPrimitivesSettlementAssetCount',
    receiverAssetCount: 'PolymeshPrimitivesSettlementAssetCount',
    offchainCount: 'u32'
  },
  /**
   * Lookup479: polymesh_primitives::settlement::AssetCount
   **/
  PolymeshPrimitivesSettlementAssetCount: {
    fungible: 'u32',
    nonFungible: 'u32',
    offChain: 'u32'
  },
  /**
   * Lookup482: pallet_statistics::pallet::Call<T>
   **/
  PalletStatisticsCall: {
    _enum: {
      set_active_asset_stats: {
        assetId: 'PolymeshPrimitivesAssetAssetId',
        statTypes: 'BTreeSet<PolymeshPrimitivesStatisticsStatType>',
      },
      batch_update_asset_stats: {
        assetId: 'PolymeshPrimitivesAssetAssetId',
        statType: 'PolymeshPrimitivesStatisticsStatType',
        values: 'BTreeSet<PolymeshPrimitivesStatisticsStatUpdate>',
      },
      set_asset_transfer_compliance: {
        assetId: 'PolymeshPrimitivesAssetAssetId',
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
   * Lookup486: pallet_sto::pallet::Call<T>
   **/
  PalletStoCall: {
    _enum: {
      create_fundraiser: {
        offeringPortfolio: 'PolymeshPrimitivesIdentityIdPortfolioId',
        offeringAsset: 'PolymeshPrimitivesAssetAssetId',
        raisingPortfolio: 'PolymeshPrimitivesIdentityIdPortfolioId',
        raisingAsset: 'PolymeshPrimitivesAssetAssetId',
        tiers: 'Vec<PalletStoPriceTier>',
        venueId: 'u64',
        start: 'Option<u64>',
        end: 'Option<u64>',
        minimumInvestment: 'u128',
        fundraiserName: 'Bytes',
      },
      invest: {
        offeringAsset: 'PolymeshPrimitivesAssetAssetId',
        fundraiserId: 'u64',
        investmentPortfolio: 'PolymeshPrimitivesIdentityIdPortfolioId',
        funding: 'PalletStoFundingMethod',
        purchaseAmount: 'u128',
        maxPrice: 'Option<u128>',
      },
      freeze_fundraiser: {
        offeringAsset: 'PolymeshPrimitivesAssetAssetId',
        fundraiserId: 'u64',
      },
      unfreeze_fundraiser: {
        offeringAsset: 'PolymeshPrimitivesAssetAssetId',
        fundraiserId: 'u64',
      },
      modify_fundraiser_window: {
        offeringAsset: 'PolymeshPrimitivesAssetAssetId',
        fundraiserId: 'u64',
        start: 'u64',
        end: 'Option<u64>',
      },
      stop: {
        offeringAsset: 'PolymeshPrimitivesAssetAssetId',
        fundraiserId: 'u64',
      },
      enable_offchain_funding: {
        offeringAsset: 'PolymeshPrimitivesAssetAssetId',
        fundraiserId: 'u64',
        ticker: 'PolymeshPrimitivesTicker'
      }
    }
  },
  /**
   * Lookup488: pallet_sto::PriceTier
   **/
  PalletStoPriceTier: {
    total: 'u128',
    price: 'u128'
  },
  /**
   * Lookup489: pallet_sto::FundingMethod<sp_core::crypto::AccountId32, sp_runtime::MultiSignature, Moment>
   **/
  PalletStoFundingMethod: {
    _enum: {
      OnChain: 'PolymeshPrimitivesIdentityIdPortfolioId',
      OffChain: 'PolymeshPrimitivesStoFundraiserReceiptDetails'
    }
  },
  /**
   * Lookup490: polymesh_primitives::sto::FundraiserReceiptDetails<sp_core::crypto::AccountId32, sp_runtime::MultiSignature, Moment>
   **/
  PolymeshPrimitivesStoFundraiserReceiptDetails: {
    uid: 'u64',
    signer: 'AccountId32',
    signature: 'SpRuntimeMultiSignature',
    expiresAt: 'u64',
    metadata: 'Option<PolymeshPrimitivesSettlementReceiptMetadata>'
  },
  /**
   * Lookup491: pallet_treasury::pallet::Call<T>
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
   * Lookup493: polymesh_primitives::Beneficiary<Balance>
   **/
  PolymeshPrimitivesBeneficiary: {
    id: 'PolymeshPrimitivesIdentityId',
    amount: 'u128'
  },
  /**
   * Lookup494: pallet_utility::pallet::Call<T>
   **/
  PalletUtilityCall: {
    _enum: {
      batch: {
        calls: 'Vec<Call>',
      },
      __Unused1: 'Null',
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
      __Unused6: 'Null',
      __Unused7: 'Null',
      __Unused8: 'Null',
      as_derivative: {
        index: 'u16',
        call: 'Call'
      }
    }
  },
  /**
   * Lookup496: polymesh_runtime_develop::runtime::OriginCaller
   **/
  PolymeshRuntimeDevelopRuntimeOriginCaller: {
    _enum: {
      system: 'FrameSupportDispatchRawOrigin',
      __Unused1: 'Null',
      __Unused2: 'Null',
      __Unused3: 'Null',
      __Unused4: 'Null',
      __Unused5: 'Null',
      __Unused6: 'Null',
      __Unused7: 'Null',
      __Unused8: 'Null',
      PolymeshCommittee: 'PalletCommitteeRawOrigin',
      __Unused10: 'Null',
      TechnicalCommittee: 'PalletCommitteeRawOrigin',
      __Unused12: 'Null',
      UpgradeCommittee: 'PalletCommitteeRawOrigin'
    }
  },
  /**
   * Lookup497: frame_support::dispatch::RawOrigin<sp_core::crypto::AccountId32>
   **/
  FrameSupportDispatchRawOrigin: {
    _enum: {
      Root: 'Null',
      Signed: 'AccountId32',
      None: 'Null',
      Authorized: 'Null'
    }
  },
  /**
   * Lookup498: pallet_committee::pallet::RawOrigin<sp_core::crypto::AccountId32, I>
   **/
  PalletCommitteeRawOrigin: {
    _enum: ['Endorsed']
  },
  /**
   * Lookup501: pallet_base::pallet::Call<T>
   **/
  PalletBaseCall: 'Null',
  /**
   * Lookup502: pallet_external_agents::pallet::Call<T>
   **/
  PalletExternalAgentsCall: {
    _enum: {
      create_group: {
        assetId: 'PolymeshPrimitivesAssetAssetId',
        perms: 'PolymeshPrimitivesSecondaryKeyExtrinsicPermissions',
      },
      set_group_permissions: {
        assetId: 'PolymeshPrimitivesAssetAssetId',
        id: 'u32',
        perms: 'PolymeshPrimitivesSecondaryKeyExtrinsicPermissions',
      },
      remove_agent: {
        assetId: 'PolymeshPrimitivesAssetAssetId',
        agent: 'PolymeshPrimitivesIdentityId',
      },
      abdicate: {
        assetId: 'PolymeshPrimitivesAssetAssetId',
      },
      change_group: {
        assetId: 'PolymeshPrimitivesAssetAssetId',
        agent: 'PolymeshPrimitivesIdentityId',
        group: 'PolymeshPrimitivesAgentAgentGroup',
      },
      accept_become_agent: {
        authId: 'u64',
      },
      create_group_and_add_auth: {
        assetId: 'PolymeshPrimitivesAssetAssetId',
        perms: 'PolymeshPrimitivesSecondaryKeyExtrinsicPermissions',
        target: 'PolymeshPrimitivesIdentityId',
        expiry: 'Option<u64>',
      },
      create_and_change_custom_group: {
        assetId: 'PolymeshPrimitivesAssetAssetId',
        perms: 'PolymeshPrimitivesSecondaryKeyExtrinsicPermissions',
        agent: 'PolymeshPrimitivesIdentityId'
      }
    }
  },
  /**
   * Lookup503: pallet_relayer::pallet::Call<T>
   **/
  PalletRelayerCall: {
    _enum: {
      approve_subsidy: {
        userKey: 'AccountId32',
        polyxLimit: 'u128',
      },
      revoke_subsidy: {
        userKey: 'AccountId32',
      },
      accept_subsidy: {
        payingKey: 'AccountId32',
      },
      remove_subsidy: {
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
        amount: 'u128',
      },
      relay_tx: {
        target: 'AccountId32',
        signature: 'SpRuntimeMultiSignature',
        call: 'Call',
        expiresAt: 'u64'
      }
    }
  },
  /**
   * Lookup504: pallet_contracts::pallet::Call<T>
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
        salt: 'Bytes',
      },
      migrate: {
        weightLimit: 'SpWeightsWeightV2Weight'
      }
    }
  },
  /**
   * Lookup506: pallet_contracts::wasm::Determinism
   **/
  PalletContractsWasmDeterminism: {
    _enum: ['Enforced', 'Relaxed']
  },
  /**
   * Lookup507: polymesh_contracts::pallet::Call<T>
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
        updates: 'Vec<(PolymeshContractsChainExtensionExtrinsicId,bool)>',
      },
      instantiate_with_code_as_primary_key: {
        endowment: 'u128',
        gasLimit: 'SpWeightsWeightV2Weight',
        storageDepositLimit: 'Option<u128>',
        code: 'Bytes',
        data: 'Bytes',
        salt: 'Bytes',
      },
      instantiate_with_hash_as_primary_key: {
        endowment: 'u128',
        gasLimit: 'SpWeightsWeightV2Weight',
        storageDepositLimit: 'Option<u128>',
        codeHash: 'H256',
        data: 'Bytes',
        salt: 'Bytes',
      },
      upgrade_api: {
        api: 'PolymeshContractsApi',
        nextUpgrade: 'PolymeshContractsNextUpgrade'
      }
    }
  },
  /**
   * Lookup510: polymesh_contracts::NextUpgrade<T>
   **/
  PolymeshContractsNextUpgrade: {
    chainVersion: 'PolymeshContractsChainVersion',
    apiHash: 'PolymeshContractsApiCodeHash'
  },
  /**
   * Lookup511: polymesh_contracts::ApiCodeHash<T>
   **/
  PolymeshContractsApiCodeHash: {
    _alias: {
      hash_: 'hash'
    },
    hash_: 'H256'
  },
  /**
   * Lookup512: pallet_preimage::pallet::Call<T>
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
        hash_: 'H256',
      },
      ensure_updated: {
        hashes: 'Vec<H256>'
      }
    }
  },
  /**
   * Lookup513: pallet_nft::pallet::Call<T>
   **/
  PalletNftCall: {
    _enum: {
      create_nft_collection: {
        assetId: 'Option<PolymeshPrimitivesAssetAssetId>',
        nftType: 'Option<PolymeshPrimitivesAssetNonFungibleType>',
        collectionKeys: 'PolymeshPrimitivesNftNftCollectionKeys',
      },
      issue_nft: {
        assetId: 'PolymeshPrimitivesAssetAssetId',
        nftMetadataAttributes: 'Vec<PolymeshPrimitivesNftNftMetadataAttribute>',
        holdingsKind: 'PolymeshPrimitivesAssetAssetHolderKind',
      },
      redeem_nft: {
        assetId: 'PolymeshPrimitivesAssetAssetId',
        nftId: 'u64',
        holdingsKind: 'PolymeshPrimitivesAssetAssetHolderKind',
        numberOfKeys: 'Option<u8>',
      },
      controller_transfer: {
        nfts: 'PolymeshPrimitivesNftNfTs',
        source: 'PolymeshPrimitivesAssetAssetHolder',
        callersHoldingsKind: 'PolymeshPrimitivesAssetAssetHolderKind'
      }
    }
  },
  /**
   * Lookup516: polymesh_primitives::nft::NFTCollectionKeys
   **/
  PolymeshPrimitivesNftNftCollectionKeys: 'Vec<PolymeshPrimitivesAssetMetadataAssetMetadataKey>',
  /**
   * Lookup519: polymesh_primitives::nft::NFTMetadataAttribute
   **/
  PolymeshPrimitivesNftNftMetadataAttribute: {
    key: 'PolymeshPrimitivesAssetMetadataAssetMetadataKey',
    value: 'Bytes'
  },
  /**
   * Lookup521: pallet_election_provider_multi_phase::pallet::Call<T>
   **/
  PalletElectionProviderMultiPhaseCall: {
    _enum: {
      submit_unsigned: {
        rawSolution: 'PalletElectionProviderMultiPhaseRawSolution',
        witness: 'PalletElectionProviderMultiPhaseSolutionOrSnapshotSize',
      },
      set_minimum_untrusted_score: {
        maybeNextScore: 'Option<SpNposElectionsElectionScore>',
      },
      set_emergency_election_result: {
        supports: 'Vec<(AccountId32,SpNposElectionsSupport)>',
      },
      submit: {
        rawSolution: 'PalletElectionProviderMultiPhaseRawSolution',
      },
      governance_fallback: 'Null'
    }
  },
  /**
   * Lookup522: pallet_election_provider_multi_phase::RawSolution<polymesh_runtime_common::NposSolution16>
   **/
  PalletElectionProviderMultiPhaseRawSolution: {
    solution: 'PolymeshRuntimeCommonNposSolution16',
    score: 'SpNposElectionsElectionScore',
    round: 'u32'
  },
  /**
   * Lookup523: polymesh_runtime_common::NposSolution16
   **/
  PolymeshRuntimeCommonNposSolution16: {
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
   * Lookup574: pallet_election_provider_multi_phase::SolutionOrSnapshotSize
   **/
  PalletElectionProviderMultiPhaseSolutionOrSnapshotSize: {
    voters: 'Compact<u32>',
    targets: 'Compact<u32>'
  },
  /**
   * Lookup578: sp_npos_elections::Support<sp_core::crypto::AccountId32>
   **/
  SpNposElectionsSupport: {
    total: 'u128',
    voters: 'Vec<(AccountId32,u128)>'
  },
  /**
   * Lookup581: polymesh_transaction_payment::pallet::Call<T>
   **/
  PolymeshTransactionPaymentCall: {
    _enum: {
      set_disable_fees: {
        value: 'bool'
      }
    }
  },
  /**
   * Lookup582: pallet_beefy::pallet::Call<T>
   **/
  PalletBeefyCall: {
    _enum: {
      report_double_voting: {
        equivocationProof: 'SpConsensusBeefyDoubleVotingProof',
        keyOwnerProof: 'SpSessionMembershipProof',
      },
      report_double_voting_unsigned: {
        equivocationProof: 'SpConsensusBeefyDoubleVotingProof',
        keyOwnerProof: 'SpSessionMembershipProof',
      },
      set_new_genesis: {
        delayInBlocks: 'u32',
      },
      report_fork_voting: {
        equivocationProof: 'SpConsensusBeefyForkVotingProof',
        keyOwnerProof: 'SpSessionMembershipProof',
      },
      report_fork_voting_unsigned: {
        equivocationProof: 'SpConsensusBeefyForkVotingProof',
        keyOwnerProof: 'SpSessionMembershipProof',
      },
      report_future_block_voting: {
        equivocationProof: 'SpConsensusBeefyFutureBlockVotingProof',
        keyOwnerProof: 'SpSessionMembershipProof',
      },
      report_future_block_voting_unsigned: {
        equivocationProof: 'SpConsensusBeefyFutureBlockVotingProof',
        keyOwnerProof: 'SpSessionMembershipProof'
      }
    }
  },
  /**
   * Lookup583: sp_consensus_beefy::DoubleVotingProof<Number, sp_consensus_beefy::ecdsa_crypto::Public, sp_consensus_beefy::ecdsa_crypto::Signature>
   **/
  SpConsensusBeefyDoubleVotingProof: {
    first: 'SpConsensusBeefyVoteMessage',
    second: 'SpConsensusBeefyVoteMessage'
  },
  /**
   * Lookup584: sp_consensus_beefy::ecdsa_crypto::Signature
   **/
  SpConsensusBeefyEcdsaCryptoSignature: '[u8;65]',
  /**
   * Lookup585: sp_consensus_beefy::VoteMessage<Number, sp_consensus_beefy::ecdsa_crypto::Public, sp_consensus_beefy::ecdsa_crypto::Signature>
   **/
  SpConsensusBeefyVoteMessage: {
    commitment: 'SpConsensusBeefyCommitment',
    id: 'SpConsensusBeefyEcdsaCryptoPublic',
    signature: 'SpConsensusBeefyEcdsaCryptoSignature'
  },
  /**
   * Lookup586: sp_consensus_beefy::commitment::Commitment<TBlockNumber>
   **/
  SpConsensusBeefyCommitment: {
    payload: 'SpConsensusBeefyPayload',
    blockNumber: 'u32',
    validatorSetId: 'u64'
  },
  /**
   * Lookup587: sp_consensus_beefy::payload::Payload
   **/
  SpConsensusBeefyPayload: 'Vec<([u8;2],Bytes)>',
  /**
   * Lookup591: sp_consensus_beefy::ForkVotingProof<sp_runtime::generic::header::Header<Number, Hash>, sp_consensus_beefy::ecdsa_crypto::Public, sp_mmr_primitives::AncestryProof<primitive_types::H256>>
   **/
  SpConsensusBeefyForkVotingProof: {
    vote: 'SpConsensusBeefyVoteMessage',
    ancestryProof: 'SpMmrPrimitivesAncestryProof',
    header: 'SpRuntimeHeader'
  },
  /**
   * Lookup592: sp_mmr_primitives::AncestryProof<primitive_types::H256>
   **/
  SpMmrPrimitivesAncestryProof: {
    prevPeaks: 'Vec<H256>',
    prevLeafCount: 'u64',
    leafCount: 'u64',
    items: 'Vec<(u64,H256)>'
  },
  /**
   * Lookup595: sp_consensus_beefy::FutureBlockVotingProof<Number, sp_consensus_beefy::ecdsa_crypto::Public>
   **/
  SpConsensusBeefyFutureBlockVotingProof: {
    vote: 'SpConsensusBeefyVoteMessage'
  },
  /**
   * Lookup596: pallet_committee::pallet::PolymeshVotes<BlockNumber>
   **/
  PalletCommitteePolymeshVotes: {
    index: 'u32',
    ayes: 'Vec<PolymeshPrimitivesIdentityId>',
    nays: 'Vec<PolymeshPrimitivesIdentityId>',
    expiry: 'PolymeshPrimitivesMaybeBlock'
  },
  /**
   * Lookup597: pallet_committee::pallet::Error<T, I>
   **/
  PalletCommitteeError: {
    _enum: ['DuplicateVote', 'NotAMember', 'NoSuchProposal', 'ProposalExpired', 'DuplicateProposal', 'MismatchedVotingIndex', 'InvalidProportion', 'FirstVoteReject', 'ProposalsLimitReached']
  },
  /**
   * Lookup606: polymesh_primitives::multisig::ProposalVoteCount
   **/
  PolymeshPrimitivesMultisigProposalVoteCount: {
    approvals: 'u64',
    rejections: 'u64'
  },
  /**
   * Lookup607: polymesh_primitives::multisig::ProposalState<Moment>
   **/
  PolymeshPrimitivesMultisigProposalState: {
    _enum: {
      Active: {
        until: 'Option<u64>',
      },
      ExecutionSuccessful: 'Null',
      ExecutionFailed: 'Null',
      Rejected: 'Null'
    }
  },
  /**
   * Lookup609: pallet_multisig::pallet::Error<T>
   **/
  PalletMultisigError: {
    _enum: ['ProposalMissing', 'DecodingError', 'RequiredSignersIsZero', 'NotASigner', 'NoSuchMultisig', 'NotEnoughSigners', 'NonceOverflow', 'AlreadyVoted', 'AlreadyASigner', 'IdentityNotAdmin', 'IdentityNotPayer', 'SignerAlreadyLinkedToMultisig', 'SignerAlreadyLinkedToIdentity', 'NestingNotAllowed', 'ProposalAlreadyRejected', 'ProposalExpired', 'ProposalAlreadyExecuted', 'MaxWeightTooLow', 'MultisigMissingIdentity', 'TooManySigners', 'NoPayingDid', 'InvalidExpiryDate', 'InvalidatedProposal', 'AdminNotFound', 'BadAuthorizationType']
  },
  /**
   * Lookup610: pallet_validators::types::PermissionedIdentityPrefs
   **/
  PalletValidatorsPermissionedIdentityPrefs: {
    intendedCount: 'u32',
    runningCount: 'u32'
  },
  /**
   * Lookup611: pallet_validators::pallet::Error<T>
   **/
  PalletValidatorsError: {
    _enum: ['StashIdentityDoesNotExist', 'StashIdentityNotPermissioned', 'IdentityIsAlreadyPermissioned', 'IdentityIsInactive', 'IntendedCountIsExceedingConsensusLimit', 'IdentityNotFound', 'ValidatorNotFound', 'CommissionTooHigh', 'CommissionUnchanged']
  },
  /**
   * Lookup612: pallet_staking::StakingLedger<T>
   **/
  PalletStakingStakingLedger: {
    stash: 'AccountId32',
    total: 'Compact<u128>',
    active: 'Compact<u128>',
    unlocking: 'Vec<PalletStakingUnlockChunk>',
    legacyClaimedRewards: 'Vec<u32>'
  },
  /**
   * Lookup614: pallet_staking::Nominations<T>
   **/
  PalletStakingNominations: {
    targets: 'Vec<AccountId32>',
    submittedIn: 'u32',
    suppressed: 'bool'
  },
  /**
   * Lookup616: pallet_staking::ActiveEraInfo
   **/
  PalletStakingActiveEraInfo: {
    index: 'u32',
    start: 'Option<u64>'
  },
  /**
   * Lookup618: sp_staking::PagedExposureMetadata<Balance>
   **/
  SpStakingPagedExposureMetadata: {
    total: 'Compact<u128>',
    own: 'Compact<u128>',
    nominatorCount: 'u32',
    pageCount: 'u32'
  },
  /**
   * Lookup620: sp_staking::ExposurePage<sp_core::crypto::AccountId32, Balance>
   **/
  SpStakingExposurePage: {
    pageTotal: 'Compact<u128>',
    others: 'Vec<SpStakingIndividualExposure>'
  },
  /**
   * Lookup621: pallet_staking::EraRewardPoints<sp_core::crypto::AccountId32>
   **/
  PalletStakingEraRewardPoints: {
    total: 'u32',
    individual: 'BTreeMap<AccountId32, u32>'
  },
  /**
   * Lookup626: pallet_staking::UnappliedSlash<sp_core::crypto::AccountId32, Balance>
   **/
  PalletStakingUnappliedSlash: {
    validator: 'AccountId32',
    own: 'u128',
    others: 'Vec<(AccountId32,u128)>',
    reporters: 'Vec<AccountId32>',
    payout: 'u128'
  },
  /**
   * Lookup628: pallet_staking::slashing::SlashingSpans
   **/
  PalletStakingSlashingSlashingSpans: {
    spanIndex: 'u32',
    lastStart: 'u32',
    lastNonzeroSlash: 'u32',
    prior: 'Vec<u32>'
  },
  /**
   * Lookup629: pallet_staking::slashing::SpanRecord<Balance>
   **/
  PalletStakingSlashingSpanRecord: {
    slashed: 'u128',
    paidOut: 'u128'
  },
  /**
   * Lookup630: pallet_staking::pallet::pallet::Error<T>
   **/
  PalletStakingPalletError: {
    _enum: ['NotController', 'NotStash', 'AlreadyBonded', 'AlreadyPaired', 'EmptyTargets', 'DuplicateIndex', 'InvalidSlashIndex', 'InsufficientBond', 'NoMoreChunks', 'NoUnlockChunk', 'FundedTarget', 'InvalidEraToReward', 'InvalidNumberOfNominations', 'NotSortedAndUnique', 'AlreadyClaimed', 'InvalidPage', 'IncorrectHistoryDepth', 'IncorrectSlashingSpans', 'BadState', 'TooManyTargets', 'BadTarget', 'CannotChillOther', 'TooManyNominators', 'TooManyValidators', 'CommissionTooLow', 'BoundNotMet', 'ControllerDeprecated', 'CannotRestoreLedger', 'RewardDestinationRestricted', 'NotEnoughFunds', 'VirtualStakerNotAllowed', 'CannotReapStash', 'AlreadyMigrated', 'Restricted']
  },
  /**
   * Lookup631: sp_staking::offence::OffenceDetails<sp_core::crypto::AccountId32, Offender>
   **/
  SpStakingOffenceOffenceDetails: {
    offender: '(AccountId32,SpStakingExposure)',
    reporters: 'Vec<AccountId32>'
  },
  /**
   * Lookup639: sp_core::crypto::KeyTypeId
   **/
  SpCoreCryptoKeyTypeId: '[u8;4]',
  /**
   * Lookup640: pallet_session::pallet::Error<T>
   **/
  PalletSessionError: {
    _enum: ['InvalidProof', 'NoAssociatedValidatorId', 'DuplicatedKey', 'NoKeys', 'NoAccount']
  },
  /**
   * Lookup643: pallet_grandpa::StoredState<N>
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
   * Lookup644: pallet_grandpa::StoredPendingChange<N, Limit>
   **/
  PalletGrandpaStoredPendingChange: {
    scheduledAt: 'u32',
    delay: 'u32',
    nextAuthorities: 'Vec<(SpConsensusGrandpaAppPublic,u64)>',
    forced: 'Option<u32>'
  },
  /**
   * Lookup646: pallet_grandpa::pallet::Error<T>
   **/
  PalletGrandpaError: {
    _enum: ['PauseFailed', 'ResumeFailed', 'ChangePending', 'TooSoon', 'InvalidKeyOwnershipProof', 'InvalidEquivocationProof', 'DuplicateOffenceReport']
  },
  /**
   * Lookup650: pallet_im_online::pallet::Error<T>
   **/
  PalletImOnlineError: {
    _enum: ['InvalidKey', 'DuplicatedHeartbeat']
  },
  /**
   * Lookup651: pallet_sudo::pallet::Error<T>
   **/
  PalletSudoError: {
    _enum: ['RequireSudo']
  },
  /**
   * Lookup652: pallet_asset::types::TickerRegistration<T>
   **/
  PalletAssetTickerRegistration: {
    owner: 'PolymeshPrimitivesIdentityId',
    expiry: 'Option<u64>'
  },
  /**
   * Lookup653: pallet_asset::types::TickerRegistrationConfig<T>
   **/
  PalletAssetTickerRegistrationConfig: {
    maxTickerLength: 'u8',
    registrationLength: 'Option<u64>'
  },
  /**
   * Lookup654: pallet_asset::types::AssetDetails
   **/
  PalletAssetAssetDetails: {
    totalSupply: 'u128',
    ownerDid: 'PolymeshPrimitivesIdentityId',
    divisible: 'bool',
    assetType: 'PolymeshPrimitivesAssetAssetType'
  },
  /**
   * Lookup665: pallet_asset::pallet::Error<T>
   **/
  PalletAssetError: {
    _enum: ['Unauthorized', 'AssetAlreadyCreated', 'TickerTooLong', 'TickerNotAlphanumeric', 'TickerAlreadyRegistered', 'TotalSupplyAboveLimit', 'NoSuchAsset', 'AlreadyFrozen', 'NotAnOwner', 'BalanceOverflow', 'TotalSupplyOverflow', 'InvalidGranularity', 'NotFrozen', 'InvalidTransfer', 'InsufficientBalance', 'AssetAlreadyDivisible', 'InvalidEthereumSignature', 'TickerRegistrationExpired', 'SenderSameAsReceiver', 'NoSuchDoc', 'MaxLengthOfAssetNameExceeded', 'FundingRoundNameMaxLengthExceeded', 'InvalidAssetIdentifier', 'InvestorUniquenessClaimNotAllowed', 'InvalidCustomAssetTypeId', 'AssetMetadataNameMaxLengthExceeded', 'AssetMetadataValueMaxLengthExceeded', 'AssetMetadataTypeDefMaxLengthExceeded', 'AssetMetadataKeyIsMissing', 'AssetMetadataValueIsLocked', 'AssetMetadataLocalKeyAlreadyExists', 'AssetMetadataGlobalKeyAlreadyExists', 'TickerFirstByteNotValid', 'UnexpectedNonFungibleToken', 'IncompatibleAssetTypeUpdate', 'AssetMetadataKeyBelongsToNFTCollection', 'AssetMetadataValueIsEmpty', 'NumberOfAssetMediatorsExceeded', 'InvalidTickerCharacter', 'InvalidTransferFrozenAsset', 'InvalidTransferComplianceFailure', 'InvalidTransferInvalidReceiverDID', 'TickerRegistrationNotFound', 'TickerIsAlreadyLinkedToAnAsset', 'AssetIdGenerationError', 'TickerNotRegisteredToCaller', 'AssetIsAlreadyLinkedToATicker', 'TickerIsNotLinkedToTheAsset', 'BadAuthorizationType', 'UnauthorizedHolderKey', 'KeyNotFoundForDid', 'InsufficientTokensLocked']
  },
  /**
   * Lookup668: pallet_corporate_actions::distribution::pallet::Error<T>
   **/
  PalletCorporateActionsDistributionPalletError: {
    _enum: ['CANotBenefit', 'AlreadyExists', 'ExpiryBeforePayment', 'HolderAlreadyPaid', 'NoSuchDistribution', 'CannotClaimBeforeStart', 'CannotClaimAfterExpiry', 'BalancePerShareProductOverflowed', 'NotDistributionCreator', 'AlreadyReclaimed', 'NotExpired', 'DistributionStarted', 'InsufficientRemainingAmount', 'DistributionAmountIsZero', 'DistributionPerShareIsZero']
  },
  /**
   * Lookup672: polymesh_primitives::checkpoint::NextCheckpoints
   **/
  PolymeshPrimitivesCheckpointNextCheckpoints: {
    nextAt: 'u64',
    totalPending: 'u64',
    schedules: 'BTreeMap<u64, u64>'
  },
  /**
   * Lookup678: pallet_asset::checkpoint::pallet::Error<T>
   **/
  PalletAssetCheckpointPalletError: {
    _enum: ['NoSuchSchedule', 'ScheduleNotRemovable', 'SchedulesOverMaxComplexity', 'ScheduleIsEmpty', 'ScheduleFinished', 'ScheduleHasExpiredCheckpoints']
  },
  /**
   * Lookup679: polymesh_primitives::compliance_manager::AssetCompliance
   **/
  PolymeshPrimitivesComplianceManagerAssetCompliance: {
    paused: 'bool',
    requirements: 'Vec<PolymeshPrimitivesComplianceManagerComplianceRequirement>'
  },
  /**
   * Lookup681: pallet_compliance_manager::pallet::Error<T>
   **/
  PalletComplianceManagerError: {
    _enum: ['Unauthorized', 'DidNotExist', 'InvalidComplianceRequirementId', 'IncorrectOperationOnTrustedIssuer', 'DuplicateComplianceRequirements', 'ComplianceRequirementTooComplex', 'WeightLimitExceeded']
  },
  /**
   * Lookup684: pallet_corporate_actions::pallet::Error<T>
   **/
  PalletCorporateActionsError: {
    _enum: ['DetailsTooLong', 'DuplicateDidTax', 'TooManyDidTaxes', 'TooManyTargetIds', 'NoSuchCheckpointId', 'NoSuchCA', 'NoRecordDate', 'RecordDateAfterStart', 'DeclDateAfterRecordDate', 'DeclDateInFuture', 'NotTargetedByCA']
  },
  /**
   * Lookup688: pallet_corporate_actions::ballot::pallet::Error<T>
   **/
  PalletCorporateActionsBallotPalletError: {
    _enum: ['CANotNotice', 'AlreadyExists', 'NoSuchBallot', 'StartAfterEnd', 'NowAfterEnd', 'NumberOfChoicesOverflow', 'VotingAlreadyStarted', 'VotingNotStarted', 'VotingAlreadyEnded', 'WrongVoteCount', 'InsufficientVotes', 'NoSuchRCVFallback', 'RCVSelfCycle', 'RCVNotAllowed']
  },
  /**
   * Lookup689: pallet_permissions::pallet::Error<T>
   **/
  PalletPermissionsError: {
    _enum: ['UnauthorizedCaller']
  },
  /**
   * Lookup690: pallet_pips::types::PipsMetadata<BlockNumber>
   **/
  PalletPipsPipsMetadata: {
    id: 'u32',
    url: 'Option<Bytes>',
    description: 'Option<Bytes>',
    createdAt: 'u32',
    transactionVersion: 'u32',
    expiry: 'PolymeshPrimitivesMaybeBlock'
  },
  /**
   * Lookup692: pallet_pips::types::DepositInfo<sp_core::crypto::AccountId32>
   **/
  PalletPipsDepositInfo: {
    owner: 'AccountId32',
    amount: 'u128'
  },
  /**
   * Lookup693: pallet_pips::types::Pip<polymesh_runtime_develop::runtime::RuntimeCall, sp_core::crypto::AccountId32>
   **/
  PalletPipsPip: {
    id: 'u32',
    proposal: 'Call',
    proposer: 'PalletPipsProposer'
  },
  /**
   * Lookup694: pallet_pips::types::VotingResult
   **/
  PalletPipsVotingResult: {
    ayesCount: 'u32',
    ayesStake: 'u128',
    naysCount: 'u32',
    naysStake: 'u128'
  },
  /**
   * Lookup695: pallet_pips::types::Vote
   **/
  PalletPipsVote: '(bool,u128)',
  /**
   * Lookup696: pallet_pips::types::SnapshotMetadata<BlockNumber, sp_core::crypto::AccountId32>
   **/
  PalletPipsSnapshotMetadata: {
    createdAt: 'u32',
    madeBy: 'AccountId32',
    id: 'u32'
  },
  /**
   * Lookup698: pallet_pips::pallet::Error<T>
   **/
  PalletPipsError: {
    _enum: ['RescheduleNotByReleaseCoordinator', 'NotFromCommunity', 'NotByCommittee', 'TooManyActivePips', 'IncorrectDeposit', 'InsufficientDeposit', 'NoSuchProposal', 'NotACommitteeMember', 'InvalidFutureBlockNumber', 'NumberOfVotesExceeded', 'StakeAmountOfVotesExceeded', 'MissingCurrentIdentity', 'IncorrectProposalState', 'CannotSkipPip', 'SnapshotResultTooLarge', 'SnapshotIdMismatch', 'ScheduledProposalDoesntExist', 'ProposalNotInScheduledState', 'InvalidPipId', 'InvalidTaskName']
  },
  /**
   * Lookup706: pallet_portfolio::pallet::Error<T>
   **/
  PalletPortfolioError: {
    _enum: ['PortfolioDoesNotExist', 'InsufficientPortfolioBalance', 'DestinationIsSamePortfolio', 'PortfolioNameAlreadyInUse', 'SecondaryKeyNotAuthorizedForPortfolio', 'UnauthorizedCustodian', 'InsufficientTokensLocked', 'PortfolioNotEmpty', 'DifferentIdentityPortfolios', 'NoDuplicateAssetsAllowed', 'NFTNotFoundInPortfolio', 'NFTAlreadyLocked', 'NFTNotLocked', 'InvalidTransferNFTNotOwned', 'InvalidTransferNFTIsLocked', 'EmptyTransfer', 'MissingOwnersPermission', 'SelfAdditionNotAllowed', 'BadAuthorizationType', 'DefaultPortfoliosCannotHaveCustodians', 'UnauthorizedPortfolioKey', 'KeyNotFoundForCaller', 'InsufficientBalance']
  },
  /**
   * Lookup707: pallet_protocol_fee::pallet::Error<T>
   **/
  PalletProtocolFeeError: {
    _enum: ['InsufficientAccountBalance', 'UnHandledImbalances', 'InsufficientSubsidyBalance']
  },
  /**
   * Lookup710: pallet_scheduler::Scheduled<Name, frame_support::traits::preimages::Bounded<polymesh_runtime_develop::runtime::RuntimeCall, sp_runtime::traits::BlakeTwo256>, BlockNumber, polymesh_runtime_develop::runtime::OriginCaller, sp_core::crypto::AccountId32>
   **/
  PalletSchedulerScheduled: {
    maybeId: 'Option<[u8;32]>',
    priority: 'u8',
    call: 'FrameSupportPreimagesBounded',
    maybePeriodic: 'Option<(u32,u32)>',
    origin: 'PolymeshRuntimeDevelopRuntimeOriginCaller'
  },
  /**
   * Lookup711: frame_support::traits::preimages::Bounded<polymesh_runtime_develop::runtime::RuntimeCall, sp_runtime::traits::BlakeTwo256>
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
   * Lookup712: sp_runtime::traits::BlakeTwo256
   **/
  SpRuntimeBlakeTwo256: 'Null',
  /**
   * Lookup715: pallet_scheduler::RetryConfig<Period>
   **/
  PalletSchedulerRetryConfig: {
    totalRetries: 'u8',
    remaining: 'u8',
    period: 'u32'
  },
  /**
   * Lookup716: pallet_scheduler::pallet::Error<T>
   **/
  PalletSchedulerError: {
    _enum: ['FailedToSchedule', 'NotFound', 'TargetBlockNumberInPast', 'RescheduleNoChange', 'Named']
  },
  /**
   * Lookup717: polymesh_primitives::settlement::Venue
   **/
  PolymeshPrimitivesSettlementVenue: {
    creator: 'PolymeshPrimitivesIdentityId',
    venueType: 'PolymeshPrimitivesSettlementVenueType'
  },
  /**
   * Lookup721: polymesh_primitives::settlement::Instruction<Moment, BlockNumber>
   **/
  PolymeshPrimitivesSettlementInstruction: {
    instructionId: 'u64',
    venueId: 'Option<u64>',
    settlementType: 'PolymeshPrimitivesSettlementSettlementType',
    createdAt: 'Option<u64>',
    tradeDate: 'Option<u64>',
    valueDate: 'Option<u64>'
  },
  /**
   * Lookup723: polymesh_primitives::settlement::LegStatus<sp_core::crypto::AccountId32>
   **/
  PolymeshPrimitivesSettlementLegStatus: {
    _enum: {
      PendingTokenLock: 'Null',
      ExecutionPending: 'Null',
      ExecutionToBeSkipped: '(AccountId32,u64)'
    }
  },
  /**
   * Lookup725: polymesh_primitives::settlement::AffirmationStatus
   **/
  PolymeshPrimitivesSettlementAffirmationStatus: {
    _enum: ['Unknown', 'Pending', 'Affirmed']
  },
  /**
   * Lookup728: polymesh_primitives::settlement::InstructionStatus<BlockNumber>
   **/
  PolymeshPrimitivesSettlementInstructionStatus: {
    _enum: {
      Unknown: 'Null',
      Pending: 'Null',
      Failed: 'Null',
      Success: 'u32',
      Rejected: 'u32',
      LockedForExecution: 'Null'
    }
  },
  /**
   * Lookup730: polymesh_primitives::settlement::MediatorAffirmationStatus<T>
   **/
  PolymeshPrimitivesSettlementMediatorAffirmationStatus: {
    _enum: {
      Unknown: 'Null',
      Pending: 'Null',
      Affirmed: {
        expiry: 'Option<u64>'
      }
    }
  },
  /**
   * Lookup731: pallet_settlement::pallet::Error<T>
   **/
  PalletSettlementError: {
    _enum: ['InvalidVenue', 'Unauthorized', 'InstructionNotAffirmed', 'UnauthorizedSigner', 'ReceiptAlreadyClaimed', 'UnauthorizedVenue', 'InstructionDatesInvalid', 'InstructionSettleBlockPassed', 'InvalidSignature', 'SameSenderReceiver', 'SettleOnPastBlock', 'UnexpectedAffirmationStatus', 'FailedToSchedule', 'UnknownInstruction', 'SignerAlreadyExists', 'SignerDoesNotExist', 'ZeroAmount', 'InstructionSettleBlockNotReached', 'CallerIsNotAParty', 'MaxNumberOfNFTsExceeded', 'NumberOfTransferredNFTsUnderestimated', 'ReceiptForInvalidLegType', 'WeightLimitExceeded', 'MaxNumberOfFungibleAssetsExceeded', 'MaxNumberOfOffChainAssetsExceeded', 'NumberOfFungibleTransfersUnderestimated', 'UnexpectedOFFChainAsset', 'OffChainAssetCantBeLocked', 'NumberOfOffChainTransfersUnderestimated', 'LegNotFound', 'InputWeightIsLessThanMinimum', 'MaxNumberOfReceiptsExceeded', 'NotAllAffirmationsHaveBeenReceived', 'InvalidInstructionStatusForExecution', 'FailedToReleaseLockOrTransferAssets', 'DuplicateReceiptUid', 'ReceiptInstructionIdMissmatch', 'MultipleReceiptsForOneLeg', 'UnexpectedLegStatus', 'NumberOfVenueSignersExceeded', 'CallerIsNotAMediator', 'InvalidExpiryDate', 'MediatorAffirmationExpired', 'OffChainAssetsMustHaveAVenue', 'UnexpectedSettlementType', 'InvalidInstructionStatusForRejection', 'LockTimestampNotFound', 'ExceededMaximumLockingPeriod', 'FailedAssetTransferringConditions', 'InvalidInstructionStatusForWithdrawal', 'ReceiverIdentityNotFound', 'InvalidAccountId', 'InvalidTaskName', 'ReceiptExpired']
  },
  /**
   * Lookup734: polymesh_primitives::statistics::Stat1stKey
   **/
  PolymeshPrimitivesStatisticsStat1stKey: {
    assetId: 'PolymeshPrimitivesAssetAssetId',
    statType: 'PolymeshPrimitivesStatisticsStatType'
  },
  /**
   * Lookup735: polymesh_primitives::transfer_compliance::AssetTransferCompliance<S>
   **/
  PolymeshPrimitivesTransferComplianceAssetTransferCompliance: {
    paused: 'bool',
    requirements: 'BTreeSet<PolymeshPrimitivesTransferComplianceTransferCondition>'
  },
  /**
   * Lookup739: pallet_statistics::pallet::Error<T>
   **/
  PalletStatisticsError: {
    _enum: ['InvalidTransferStatisticsFailure', 'StatTypeMissing', 'StatTypeNeededByTransferCondition', 'CannotRemoveStatTypeInUse', 'StatTypeLimitReached', 'TransferConditionLimitReached', 'WeightLimitExceeded']
  },
  /**
   * Lookup742: pallet_sto::pallet::Error<T>
   **/
  PalletStoError: {
    _enum: ['Unauthorized', 'Overflow', 'InsufficientTokensRemaining', 'FundraiserNotFound', 'FundraiserNotLive', 'FundraiserClosed', 'FundraiserExpired', 'InvalidVenue', 'InvalidPriceTiers', 'InvalidOfferingWindow', 'MaxPriceExceeded', 'InvestmentAmountTooLow', 'InvalidSignature', 'OffchainFundingNotAllowed', 'ReceiptExpired']
  },
  /**
   * Lookup743: pallet_treasury::pallet::Error<T>
   **/
  PalletTreasuryError: {
    _enum: ['InsufficientBalance', 'InvalidIdentity']
  },
  /**
   * Lookup744: pallet_utility::pallet::Error<T>
   **/
  PalletUtilityError: {
    _enum: ['TooManyCalls', 'UnableToDeriveAccountId']
  },
  /**
   * Lookup745: pallet_base::pallet::Error<T>
   **/
  PalletBaseError: {
    _enum: ['TooLong', 'CounterOverflow', 'InvalidAccountId']
  },
  /**
   * Lookup748: pallet_external_agents::pallet::Error<T>
   **/
  PalletExternalAgentsError: {
    _enum: ['NoSuchAG', 'UnauthorizedAgent', 'AlreadyAnAgent', 'NotAnAgent', 'RemovingLastFullAgent', 'SecondaryKeyNotAuthorizedForAsset', 'BadAuthorizationType', 'ExceptPermissionsNotAllowed']
  },
  /**
   * Lookup749: pallet_relayer::pallet::Subsidy<sp_core::crypto::AccountId32>
   **/
  PalletRelayerSubsidy: {
    payingKey: 'AccountId32',
    remaining: 'u128'
  },
  /**
   * Lookup750: pallet_relayer::pallet::Error<T>
   **/
  PalletRelayerError: {
    _enum: ['NoPayingKey', 'NotPayingKey', 'NotAuthorized', 'Overflow', 'NoPendingSubsidy', 'InvalidSignature', 'ExpiredRelayTx']
  },
  /**
   * Lookup752: pallet_contracts::wasm::CodeInfo<T>
   **/
  PalletContractsWasmCodeInfo: {
    owner: 'AccountId32',
    deposit: 'Compact<u128>',
    refcount: 'Compact<u64>',
    determinism: 'PalletContractsWasmDeterminism',
    codeLen: 'u32'
  },
  /**
   * Lookup753: pallet_contracts::storage::ContractInfo<T>
   **/
  PalletContractsStorageContractInfo: {
    trieId: 'Bytes',
    codeHash: 'H256',
    storageBytes: 'u32',
    storageItems: 'u32',
    storageByteDeposit: 'u128',
    storageItemDeposit: 'u128',
    storageBaseDeposit: 'u128',
    delegateDependencies: 'BTreeMap<H256, u128>'
  },
  /**
   * Lookup758: pallet_contracts::storage::DeletionQueueManager<T>
   **/
  PalletContractsStorageDeletionQueueManager: {
    insertCounter: 'u32',
    deleteCounter: 'u32'
  },
  /**
   * Lookup760: pallet_contracts::schedule::Schedule<T>
   **/
  PalletContractsSchedule: {
    limits: 'PalletContractsScheduleLimits',
    instructionWeights: 'PalletContractsScheduleInstructionWeights'
  },
  /**
   * Lookup761: pallet_contracts::schedule::Limits
   **/
  PalletContractsScheduleLimits: {
    eventTopics: 'u32',
    memoryPages: 'u32',
    subjectLen: 'u32',
    payloadLen: 'u32',
    runtimeMemory: 'u32',
    validatorRuntimeMemory: 'u32',
    eventRefTime: 'u64'
  },
  /**
   * Lookup762: pallet_contracts::schedule::InstructionWeights<T>
   **/
  PalletContractsScheduleInstructionWeights: {
    base: 'u32'
  },
  /**
   * Lookup763: pallet_contracts::Environment<T>
   **/
  PalletContractsEnvironment: {
    _alias: {
      hash_: 'hash'
    },
    accountId: 'PalletContractsEnvironmentTypeAccountId32',
    balance: 'PalletContractsEnvironmentTypeU128',
    hash_: 'PalletContractsEnvironmentTypeH256',
    hasher: 'PalletContractsEnvironmentTypeBlakeTwo256',
    timestamp: 'PalletContractsEnvironmentTypeU64',
    blockNumber: 'PalletContractsEnvironmentTypeU32'
  },
  /**
   * Lookup764: pallet_contracts::EnvironmentType<sp_core::crypto::AccountId32>
   **/
  PalletContractsEnvironmentTypeAccountId32: 'Null',
  /**
   * Lookup765: pallet_contracts::EnvironmentType<T>
   **/
  PalletContractsEnvironmentTypeU128: 'Null',
  /**
   * Lookup766: pallet_contracts::EnvironmentType<primitive_types::H256>
   **/
  PalletContractsEnvironmentTypeH256: 'Null',
  /**
   * Lookup767: pallet_contracts::EnvironmentType<sp_runtime::traits::BlakeTwo256>
   **/
  PalletContractsEnvironmentTypeBlakeTwo256: 'Null',
  /**
   * Lookup768: pallet_contracts::EnvironmentType<T>
   **/
  PalletContractsEnvironmentTypeU64: 'Null',
  /**
   * Lookup769: pallet_contracts::EnvironmentType<T>
   **/
  PalletContractsEnvironmentTypeU32: 'Null',
  /**
   * Lookup771: pallet_contracts::pallet::Error<T>
   **/
  PalletContractsError: {
    _enum: ['InvalidSchedule', 'InvalidCallFlags', 'OutOfGas', 'OutputBufferTooSmall', 'TransferFailed', 'MaxCallDepthReached', 'ContractNotFound', 'CodeTooLarge', 'CodeNotFound', 'CodeInfoNotFound', 'OutOfBounds', 'DecodingFailed', 'ContractTrapped', 'ValueTooLarge', 'TerminatedWhileReentrant', 'InputForwarded', 'RandomSubjectTooLong', 'TooManyTopics', 'NoChainExtension', 'XCMDecodeFailed', 'DuplicateContract', 'TerminatedInConstructor', 'ReentranceDenied', 'StateChangeDenied', 'StorageDepositNotEnoughFunds', 'StorageDepositLimitExhausted', 'CodeInUse', 'ContractReverted', 'CodeRejected', 'Indeterministic', 'MigrationInProgress', 'NoMigrationPerformed', 'MaxDelegateDependenciesReached', 'DelegateDependencyNotFound', 'DelegateDependencyAlreadyExists', 'CannotAddSelfAsDelegateDependency', 'OutOfTransientStorage']
  },
  /**
   * Lookup773: polymesh_contracts::pallet::Error<T>
   **/
  PolymeshContractsError: {
    _enum: ['InvalidFuncId', 'InvalidRuntimeCall', 'ReadStorageFailed', 'DataLeftAfterDecoding', 'InLenTooLarge', 'OutLenTooLarge', 'InstantiatorWithNoIdentity', 'RuntimeCallDenied', 'CallerNotAPrimaryKey', 'MissingKeyPermissions', 'InvalidChainVersion', 'NoUpgradesSupported']
  },
  /**
   * Lookup774: pallet_preimage::OldRequestStatus<sp_core::crypto::AccountId32, Balance>
   **/
  PalletPreimageOldRequestStatus: {
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
   * Lookup776: pallet_preimage::RequestStatus<sp_core::crypto::AccountId32, frame_support::traits::tokens::fungible::HoldConsideration<A, F, R, D, Fp>>
   **/
  PalletPreimageRequestStatus: {
    _enum: {
      Unrequested: {
        ticket: '(AccountId32,u128)',
        len: 'u32',
      },
      Requested: {
        maybeTicket: 'Option<(AccountId32,u128)>',
        count: 'u32',
        maybeLen: 'Option<u32>'
      }
    }
  },
  /**
   * Lookup781: pallet_preimage::pallet::Error<T>
   **/
  PalletPreimageError: {
    _enum: ['TooBig', 'AlreadyNoted', 'NotAuthorized', 'NotNoted', 'Requested', 'NotRequested', 'TooMany', 'TooFew']
  },
  /**
   * Lookup782: polymesh_primitives::nft::NFTCollection
   **/
  PolymeshPrimitivesNftNftCollection: {
    id: 'u64',
    assetId: 'PolymeshPrimitivesAssetAssetId'
  },
  /**
   * Lookup787: polymesh_primitives::nft::NFTOwnerStatus
   **/
  PolymeshPrimitivesNftNftOwnerStatus: {
    _enum: ['NotOwned', 'Owner', 'OwnerLocked']
  },
  /**
   * Lookup788: pallet_nft::pallet::Error<T>
   **/
  PalletNftError: {
    _enum: ['BalanceOverflow', 'BalanceUnderflow', 'CollectionAlredyRegistered', 'CollectionNotFound', 'DuplicateMetadataKey', 'DuplicatedNFTId', 'InvalidAssetType', 'InvalidMetadataAttribute', 'InvalidNFTTransferCollectionNotFound', 'InvalidNFTTransferSamePortfolio', 'InvalidNFTTransferNFTNotOwned', 'InvalidNFTTransferCountOverflow', 'InvalidNFTTransferComplianceFailure', 'InvalidNFTTransferFrozenAsset', 'InvalidNFTTransferInsufficientCount', 'MaxNumberOfKeysExceeded', 'MaxNumberOfNFTsPerLegExceeded', 'NFTNotFound', 'UnregisteredMetadataKey', 'ZeroCount', 'SupplyOverflow', 'SupplyUnderflow', 'InvalidNFTTransferNFTIsLocked', 'InvalidNFTTransferSenderDidMatchesReceiverDid', 'InvalidNFTTransferInvalidReceiverDID', 'InvalidAssetId', 'NFTIsLocked', 'NumberOfKeysIsLessThanExpected', 'NFTIsNotLocked']
  },
  /**
   * Lookup789: pallet_election_provider_multi_phase::ReadySolution<AccountId, MaxWinners, MaxBackersPerWinner>
   **/
  PalletElectionProviderMultiPhaseReadySolution: {
    supports: 'FrameElectionProviderSupportBoundedSupports',
    score: 'SpNposElectionsElectionScore',
    compute: 'PalletElectionProviderMultiPhaseElectionCompute'
  },
  /**
   * Lookup790: frame_election_provider_support::BoundedSupports<sp_core::crypto::AccountId32, BOuter, BInner>
   **/
  FrameElectionProviderSupportBoundedSupports: 'Vec<(AccountId32,FrameElectionProviderSupportBoundedSupport)>',
  /**
   * Lookup793: frame_election_provider_support::BoundedSupport<sp_core::crypto::AccountId32, Bound>
   **/
  FrameElectionProviderSupportBoundedSupport: {
    total: 'u128',
    voters: 'Vec<(AccountId32,u128)>'
  },
  /**
   * Lookup796: pallet_election_provider_multi_phase::RoundSnapshot<sp_core::crypto::AccountId32, VoterType>
   **/
  PalletElectionProviderMultiPhaseRoundSnapshot: {
    voters: 'Vec<(AccountId32,u64,Vec<AccountId32>)>',
    targets: 'Vec<AccountId32>'
  },
  /**
   * Lookup802: pallet_election_provider_multi_phase::signed::SignedSubmission<sp_core::crypto::AccountId32, Balance, polymesh_runtime_common::NposSolution16>
   **/
  PalletElectionProviderMultiPhaseSignedSignedSubmission: {
    who: 'AccountId32',
    deposit: 'u128',
    rawSolution: 'PalletElectionProviderMultiPhaseRawSolution',
    callFee: 'u128'
  },
  /**
   * Lookup803: pallet_election_provider_multi_phase::pallet::Error<T>
   **/
  PalletElectionProviderMultiPhaseError: {
    _enum: ['PreDispatchEarlySubmission', 'PreDispatchWrongWinnerCount', 'PreDispatchWeakSubmission', 'SignedQueueFull', 'SignedCannotPayDeposit', 'SignedInvalidWitness', 'SignedTooMuchWeight', 'OcwCallWrongEra', 'MissingSnapshotMetadata', 'InvalidSubmissionIndex', 'CallNotAllowed', 'FallbackFailed', 'BoundNotMet', 'TooManyWinners', 'PreDispatchDifferentRound']
  },
  /**
   * Lookup806: pallet_beefy::pallet::Error<T>
   **/
  PalletBeefyError: {
    _enum: ['InvalidKeyOwnershipProof', 'InvalidDoubleVotingProof', 'InvalidForkVotingProof', 'InvalidFutureBlockVotingProof', 'InvalidEquivocationProofSession', 'InvalidEquivocationProofSessionMember', 'DuplicateOffenceReport', 'InvalidConfiguration']
  },
  /**
   * Lookup807: sp_consensus_beefy::mmr::BeefyAuthoritySet<primitive_types::H256>
   **/
  SpConsensusBeefyMmrBeefyAuthoritySet: {
    id: 'u64',
    len: 'u32',
    keysetCommitment: 'H256'
  },
  /**
   * Lookup810: frame_system::extensions::authorize_call::AuthorizeCall<T>
   **/
  FrameSystemExtensionsAuthorizeCall: 'Null',
  /**
   * Lookup811: frame_system::extensions::check_non_zero_sender::CheckNonZeroSender<T>
   **/
  FrameSystemExtensionsCheckNonZeroSender: 'Null',
  /**
   * Lookup812: frame_system::extensions::check_spec_version::CheckSpecVersion<T>
   **/
  FrameSystemExtensionsCheckSpecVersion: 'Null',
  /**
   * Lookup813: frame_system::extensions::check_tx_version::CheckTxVersion<T>
   **/
  FrameSystemExtensionsCheckTxVersion: 'Null',
  /**
   * Lookup814: frame_system::extensions::check_genesis::CheckGenesis<T>
   **/
  FrameSystemExtensionsCheckGenesis: 'Null',
  /**
   * Lookup817: frame_system::extensions::check_nonce::CheckNonce<T>
   **/
  FrameSystemExtensionsCheckNonce: 'Compact<u32>',
  /**
   * Lookup818: frame_system::extensions::check_weight::CheckWeight<T>
   **/
  FrameSystemExtensionsCheckWeight: 'Null',
  /**
   * Lookup819: polymesh_transaction_payment::ChargeTransactionPayment<T>
   **/
  PolymeshTransactionPaymentChargeTransactionPayment: 'Compact<u128>',
  /**
   * Lookup820: pallet_permissions::StoreCallMetadata<T>
   **/
  PalletPermissionsStoreCallMetadata: 'Null',
  /**
   * Lookup821: frame_metadata_hash_extension::CheckMetadataHash<T>
   **/
  FrameMetadataHashExtensionCheckMetadataHash: {
    mode: 'FrameMetadataHashExtensionMode'
  },
  /**
   * Lookup822: frame_metadata_hash_extension::Mode
   **/
  FrameMetadataHashExtensionMode: {
    _enum: ['Disabled', 'Enabled']
  },
  /**
   * Lookup823: frame_system::extensions::weight_reclaim::WeightReclaim<T>
   **/
  FrameSystemExtensionsWeightReclaim: 'Null'
};
