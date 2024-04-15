// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
// Copyright (c) 2023 Polymesh Association

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3.

// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

//! Runtime API definition for Settlement module.

use frame_support::dispatch::DispatchError;
use sp_std::vec::Vec;

use polymesh_primitives::settlement::{
    AffirmationCount, ExecuteInstructionInfo, InstructionId, Leg,
};
use polymesh_primitives::PortfolioId;

sp_api::decl_runtime_apis! {
    pub trait SettlementApi {
        /// Returns an [`ExecuteInstructionInfo`] instance containing the consumed weight and the number of fungible and non fungible
        /// tokens in the instruction. Executing an instruction includes verifying the compliance and transfer restrictions of all assets
        /// in the instruction, unlocking all assets, pruning the instruction, updating the statistics for each asset and more.
        ///
        /// ```ignore
        /// curl http://localhost:9933 -H "Content-Type: application/json" -d '{
        ///     "id":1,
        ///     "jsonrpc":"2.0",
        ///     "method": "settlement_getExecuteInstructionInfo",
        ///     "params": [1]
        ///   }'
        /// ```
        fn get_execute_instruction_info(instruction_id: &InstructionId) -> ExecuteInstructionInfo;

        /// Returns an [`AffirmationCount`] instance containing the number of assets being sent/received from `portfolios`,
        /// and the number of off-chain assets in the instruction.
        ///
        /// ```ignore
        /// curl http://localhost:9933 -H "Content-Type: application/json" -d '{
        ///     "id":1,
        ///     "jsonrpc":"2.0",
        ///     "method": "settlement_getAffirmationCount",
        ///     "params": [1, [{ "did": "0x0100000000000000000000000000000000000000000000000000000000000000", "kind": "Default"}]]
        ///   }'
        /// ```
        fn get_affirmation_count(instruction_id: InstructionId, portfolios: Vec<PortfolioId>) -> AffirmationCount;

        /// Returns `Ok` if the leg can be transferred. Otherwise, returns a vector containing all errors for the transfer.
        ///
        /// ```ignore
        /// curl http://localhost:9933 -H "Content-Type: application/json" -d '{
        ///     "id":1,
        ///     "jsonrpc":"2.0",
        ///     "method": "settlement_getTransferReport",
        ///     "params": [
        ///         {
        ///            "NonFungible":
        ///                {
        ///                    "sender": { "did": "0x0100000000000000000000000000000000000000000000000000000000000000", "kind": "Default"},
        ///                    "receiver": { "did": "0x0100000000000000000000000000000000000000000000000000000000000000", "kind": "Default"},
        ///                    "nfts": { "ticker": "0x5449434B4552303030303031", "ids": [1]}
        ///                }
        ///         },
        ///         false
        ///     ]
        /// }'
        /// ```
        fn get_transfer_report(leg: Leg, skip_locked_check: bool) -> Result<(), Vec<DispatchError>>;

        /// Returns `Ok` if the instruction can be executed. Otherwise, returns a vector containing all errors for the execution.
        ///
        /// ```ignore
        /// curl http://localhost:9933 -H "Content-Type: application/json" -d '{
        ///     "id":1,
        ///     "jsonrpc":"2.0",
        ///     "method": "settlement_getExecuteIntructionReport",
        ///     "params": [1]]
        ///   }'
        /// ```
        fn get_execute_instruction_report(instruction_id: InstructionId) -> Result<(), Vec<DispatchError>>;
    }
}
