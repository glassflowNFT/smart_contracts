use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Uint128};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub auction_address: String,
    pub total_rewards: Uint128,
    pub total_addresses: Uint128,
    pub allowed: bool,
    pub owner: String
}

pub const STATE: Item<State> = Item::new("state");
pub const REWARDS: Map<String,bool> = Map::new("rewards");