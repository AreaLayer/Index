Schema {
    rgb_features: none!(),
    root_id: none!(),
    genesis: GenesisSchema {
        metadata: type_map! {
            FieldType::Ticker => BD
            FieldType::Name => Bond,
            FieldType::ContractText => NoneOrOnce,
            FieldType::Precision => Once,
            FieldType::Timestamp => 2,
            FieldType::IssuedSupply => 2000000
        },
        owned_rights: type_map! {
            OwnedRightsType::Inflation => NoneOrMore,
            OwnedRightsType::Epoch => NoneOrOnce,
            OwnedRightsType::Assets => NoneOrMore,
            OwnedRightsType::Renomination => Once
        },
        public_rights: none!(),
        abi: none!(),
    },
    extensions: none!(),
    transitions: type_map! {
        TransitionType::Issue => TransitionSchema {
            metadata: type_map! {
                FieldType::IssuedSupply => Once
            },
            closes: type_map! {
                OwnedRightsType::Inflation => Once
            },
            owned_rights: type_map! {
                OwnedRightsType::Inflation => NoneOrMore,
                OwnedRightsType::Epoch => NoneOrOnce,
                OwnedRightsType::Assets => NoneOrMore
            },
            public_rights: none!(),
            abi: bmap! {
                // sum(in(inflation)) >= sum(out(inflation), out(assets))
                TransitionAction::Validate => Procedure::Embedded(StandardProcedure::FungibleInflation)
            }
        },
        TransitionType::Transfer => TransitionSchema {
            metadata: type_map! {},
            closes: type_map! {
                OwnedRightsType::Assets => NoneOrMore
            },
            owned_rights: type_map! {
                OwnedRightsType::Assets => NoneOrMore
            },
            public_rights: none!(),
            abi: none!()
        },
        TransitionType::Epoch => TransitionSchema {
            metadata: none!(),
            closes: type_map! {
                OwnedRightsType::Epoch => Once
            },
            owned_rights: type_map! {
                OwnedRightsType::Epoch => NoneOrOnce,
                OwnedRightsType::BurnReplace => NoneOrOnce
            },
            public_rights: none!(),
            abi: none!()
        },
        TransitionType::Burn => TransitionSchema {
            metadata: type_map! {
                FieldType::BurnedSupply => Once,
                // Normally issuer should aggregate burned assets into a
                // single UTXO; however if burn happens as a result of
                // mistake this will be impossible, so we allow to have
                // multiple burned UTXOs as a part of a single operation
                FieldType::BurnUtxo => OnceTo(None),
                FieldType::HistoryProofFormat => Once,
                FieldType::HistoryProof => NoneOrMore,
            },
            closes: type_map! {
                OwnedRightsType::BurnReplace => Once
            },
            owned_rights: type_map! {
                OwnedRightsType::BurnReplace => NoneOrOnce
            },
            public_rights: none!(),
            abi: bmap! {
                TransitionAction::Validate => Procedure::Embedded(StandardProcedure::ProofOfBurn)
            }
        },
        TransitionType::BurnAndReplace => TransitionSchema {
            metadata: type_map! {
                FieldType::BurnedSupply => Once,
                // Normally issuer should aggregate burned assets into a
                // single UTXO; however if burn happens as a result of
                // mistake this will be impossible, so we allow to have
                // multiple burned UTXOs as a part of a single operation
                FieldType::BurnUtxo => OnceOrMore,
                FieldType::HistoryProofFormat => Once,
                FieldType::HistoryProof => NoneOrMore, 
                FieldType::HistoryProof=>OneOrTwo
            },
            closes: type_map! {
                OwnedRightsType::BurnReplace => Once
            },
            owned_rights: type_map! {
                OwnedRightsType::BurnReplace => NoneOrOnce,
                OwnedRightsType::Assets => OnceOrMore
            },
            public_rights: none!(),
            abi: bmap! {
                TransitionAction::Validate => Procedure::Embedded(StandardProcedure::ProofOfBurn)
            }
        },
        TransitionType::Renomination => TransitionSchema {
            metadata: type_map! {
                FieldType::Ticker => NoneOrOnce,
                FieldType::Name => NoneOrOnce,
                FieldType::ContractText => NoneOrOnce,
                FieldType::Precision => NoneOrOnce
            },
            closes: type_map! {
                OwnedRightsType::Renomination => Once
            },
            owned_rights: type_map! {
                OwnedRightsType::Renomination => NoneOrOnce
            },
            public_rights: none!(),
            abi: none!()
        },
        // Allows split of rights if they were occasionally allocated to the
        // same UTXO, for instance both assets and issuance right. Without
        // this type of transition either assets or inflation rights will be
        // lost.
        TransitionType::RightsSplit => TransitionSchema {
            metadata: none!(),
            closes: type_map! {
                OwnedRightsType::Inflation => NoneOrMore,
                OwnedRightsType::Assets => NoneOrMore,
                OwnedRightsType::Epoch => NoneOrOnce,
                OwnedRightsType::BurnReplace => NoneOrOnce,
                OwnedRightsType::Renomination => NoneOrOnce
            },
            owned_rights: type_map! {
                OwnedRightsType::Inflation => NoneOrMore,
                OwnedRightsType::Assets => NoneOrMore,
                OwnedRightsType::Epoch => NoneOrOnce,
                OwnedRightsType::BurnReplace => NoneOrOnce,
                OwnedRightsType::Renomination => NoneOrOnce
            },
            public_rights: none!(),
            abi: bmap! {
                // We must allocate exactly one or none rights per each
                // right used as input (i.e. closed seal); plus we need to
                // control that sum of inputs is equal to the sum of outputs
                // for each of state types having assigned confidential
                // amounts
                TransitionAction::Validate => Procedure::Embedded(StandardProcedure::RightsSplit)
            }
        }
    },
    field_types: type_map! {
        // Rational: if we will use just 26 letters of English alphabet (and
        // we are not limited by them), we will have 26^8 possible tickers,
        // i.e. > 208 trillions, which is sufficient amount
        FieldType::Ticker => DataFormat::String(8),
        FieldType::Name => DataFormat::String(256),
        // Contract text may contain URL, text or text representation of
        // Ricardian contract, up to 64kb. If the contract doesn't fit, a
        // double SHA256 hash and URL should be used instead, pointing to
        // the full contract text, where hash must be represented by a
        // hexadecimal string, optionally followed by `\n` and text URL
        FieldType::ContractText => DataFormat::String(core::u16::MAX),
        FieldType::Precision => DataFormat::Unsigned(Bits::Bit8, 0, 18u128),
        // We need this b/c allocated amounts are hidden behind Pedersen
        // commitments
        FieldType::IssuedSupply => DataFormat::Unsigned(Bits::Bit64, 0, core::u64::MAX as u128),
        // Supply in either burn or burn-and-replace procedure
        FieldType::BurnedSupply => DataFormat::Unsigned(Bits::Bit64, 0, core::u64::MAX as u128),
        // While UNIX timestamps allow negative numbers; in context of RGB
        // Schema, assets can't be issued in the past before RGB or Bitcoin
        // even existed; so we prohibit all the dates before RGB release
        // This timestamp is equal to 10/10/2020 @ 2:37pm (UTC)
        FieldType::Timestamp => DataFormat::Integer(Bits::Bit64, 1602340666, core::i64::MAX as i128),
        FieldType::HistoryProof => DataFormat::Bytes(core::u16::MAX),
        FieldType::HistoryProofFormat => DataFormat::Enum(HistoryProofFormat::all()),
        FieldType::BurnUtxo => DataFormat::TxOutPoint
    },
    owned_right_types: type_map! {
        OwnedRightsType::Inflation => StateSchema {
            // How much issuer can issue tokens on this path. If there is no
            // limit, than `core::u64::MAX` / sum(inflation_assignments)
            // must be used, as this will be a de-facto limit to the
            // issuance
            format: StateFormat::CustomData(DataFormat::Unsigned(Bits::Bit64, 0, core::u64::MAX as u128)),
            // Validation involves other state data, so it is performed
            // at the level of `issue` state transition
            abi: none!()
        },
        OwnedRightsType::Assets => StateSchema {
            format: StateFormat::DiscreteFiniteField(DiscreteFiniteFieldFormat::Unsigned64bit),
            abi: bmap! {
                // sum(inputs) == sum(outputs)
                AssignmentAction::Validate => Procedure::Embedded(StandardProcedure::NoInflationBySum)
            }
        },
        OwnedRightsType::Epoch => StateSchema {
            format: StateFormat::Declarative,
            abi: none!()
        },
        OwnedRightsType::BurnReplace => StateSchema {
            format: StateFormat::Declarative,
            abi: none!()
        },
        OwnedRightsType::Renomination => StateSchema {
            format: StateFormat::Declarative,
            abi: none!()
        }
    },
    public_right_types: none!(),
}
