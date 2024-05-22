use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};
use cw20::Cw20Coin;
use cw20_base::msg::InstantiateMarketingInfo;

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Option<String>,
    pub cw20_code_id: u64,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub initial_balances: Vec<Cw20Coin>,
    pub marketing: Option<InstantiateMarketingInfo>,
    pub cap: Option<Uint128>,
}

#[cw_serde]
pub enum ExecuteMsg {
    MintToken { to: String, amount: Uint128 },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Addr)]
    Token {},
}
