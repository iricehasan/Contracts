#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, wasm_instantiate, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Reply,
    Response, StdResult, SubMsg, Uint128, WasmMsg,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{ADMIN, TOKEN};

use cw_utils::parse_reply_instantiate_data;

use cw20::{Cw20ExecuteMsg, MinterResponse};
use cw20_base::msg::InstantiateMsg as Cw20InstantaiteMsg;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw20-minter";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const INSTANTIATE_REPLY: u64 = 1;
pub const MINT_REPLY: u64 = 2;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let admin = msg
        .admin
        .and_then(|s| deps.api.addr_validate(s.as_str()).ok())
        .unwrap_or(info.sender);

    ADMIN.save(deps.storage, &admin)?;

    let cw20_init_msg = Cw20InstantaiteMsg {
        name: msg.name,
        symbol: msg.symbol,
        decimals: msg.decimals,
        initial_balances: msg.initial_balances,
        mint: Some(MinterResponse {
            minter: env.contract.address.to_string(),
            cap: msg.cap,
        }),
        marketing: msg.marketing,
    };

    let submsg = SubMsg::reply_on_success(
        wasm_instantiate(
            msg.cw20_code_id,
            &cw20_init_msg,
            vec![],
            "Cw20 Contract".to_owned(),
        )
        .unwrap(),
        INSTANTIATE_REPLY,
    );

    TOKEN.save(deps.storage, &Addr::unchecked(""))?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_submessage(submsg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::MintToken { to, amount } => {
            if amount == Uint128::zero() {
                return Err(ContractError::ZeroAmount {});
            }

            let admin = ADMIN.load(deps.storage)?;

            if info.sender != admin {
                return Err(ContractError::Unauthorized {});
            }

            let token_addr = TOKEN.load(deps.storage)?;

            let submsg_mint = SubMsg::reply_on_success(
                WasmMsg::Execute {
                    contract_addr: token_addr.clone().to_string(),
                    msg: to_json_binary(&Cw20ExecuteMsg::Mint {
                        recipient: to,
                        amount,
                    })?,
                    funds: vec![],
                },
                MINT_REPLY,
            );

            Ok(Response::new()
                .add_attribute("action", "mint")
                .add_submessage(submsg_mint))
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, reply: Reply) -> Result<Response, ContractError> {
    match reply.id {
        INSTANTIATE_REPLY => {
            let res = parse_reply_instantiate_data(reply).unwrap();
            let contract_address = deps.api.addr_validate(&res.contract_address).unwrap();
            TOKEN.save(deps.storage, &contract_address)?;

            Ok(Response::default())
        }
        MINT_REPLY => Ok(Response::new().add_attribute("Operation", "mint")),
        _ => Err(ContractError::UnrecognizedReply {}),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Token {} => Ok(to_json_binary(&TOKEN.load(deps.storage)?)?),
    }
}
