#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, CosmosMsg, WasmMsg, BankMsg, Coin};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{AddressesResponse, RewardsResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{State, STATE, REWARDS};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:faucet-contract";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        auction_address: msg.auction_contract,
        total_addresses: Uint128::zero(),
        total_rewards: Uint128::zero(),
        allowed: false,
        owner: info.sender.to_string()
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("auction_address", msg.auction_contract))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::AddRewards {} => add_rewards(deps, info),
        ExecuteMsg::ReleaseRewards {} => release_rewards(deps, info),
        ExecuteMsg::Register {address} => register_address(deps,info, address),
        ExecuteMsg::AllowRelease {} => allow_release(deps,info),
        ExecuteMsg::RefuseRelease {} => refuse_release(deps,info),

    }
}
pub fn add_rewards(deps: DepsMut, info: MessageInfo ) -> Result<Response, ContractError> {
    // Add rewards to the total pool of rewards
    // Can only be called by admin
    let state = STATE.load(deps.storage)?;
    // Make sure CHT is sents
    if info.funds.first().unwrap().denom != "cgas" {
        return Err(ContractError::WrongDenom{})
    }
    // Make sure that release is not yet allowed 
    if state.allowed { 
        return Err(ContractError::Allowed{})
    }
    let amount = info.funds.first().unwrap().amount;
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        state.total_rewards += amount;
        Ok(state)
    })?;

    Ok(Response::new().add_attribute("method", "add_rewards")
    .add_attribute("amount", amount))
}

pub fn allow_release(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    // Allows release of funds from users registered
    // Can only be called by admin
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if info.sender.to_string() != state.owner {
            return Err(ContractError::Unauthorized{})
        }
        state.allowed = true;
        Ok(state)
    })?;

    Ok(Response::new().add_attribute("method", "try_increment"))
}

pub fn refuse_release(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    // Opposite of allow_release
    // Can only be called by admin
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if info.sender.to_string() != state.owner {
            return Err(ContractError::Unauthorized{})
        }
        state.allowed = false;
        Ok(state)
    })?;

    Ok(Response::new().add_attribute("method", "try_increment"))
}

pub fn release_rewards(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    // Checks if sender is in rewards Map
    let sender = REWARDS.may_load(deps.storage, info.sender.to_string())?;
    let sender = match sender {
        Some(a) => a,
        None => {return Err(ContractError::Unauthorized{})}
    };
    let state = STATE.load(deps.storage)?;
    if !state.allowed {
        return Err(ContractError::Unauthorized{})
    }
    let amount_sender = state.total_rewards.checked_div(state.total_addresses).unwrap();

    Ok(Response::new().add_attribute("method", "try_increment")
    .add_message(CosmosMsg::Bank(BankMsg::Send{
        amount: vec![Coin {amount: amount_sender, denom: "cgas".to_string()}],
        to_address: info.sender.to_string()
    })))
}

pub fn register_address(deps: DepsMut, info: MessageInfo, address: String) -> Result<Response, ContractError> {
    // registers address
    // Only auction smart contract is allowed to do that 
    // Check that we are not allowing withdrawals 
    let state = STATE.load(deps.storage)?;

    if info.sender.to_string() != state.auction_address {
        return Err(ContractError::Unauthorized{})
    }

    if state.allowed { 
        return Err(ContractError::Allowed{})
    }

    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        state.total_addresses += Uint128::from(1u64);
        Ok(state)
    })?;
    REWARDS.save(deps.storage, address, &true)?;

    Ok(Response::new().add_attribute("method", "register_address"))
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetAddresses {} => to_binary(&query_addresses(deps)?),
        QueryMsg::GetRewards {} => to_binary(&query_rewards(deps)?),

    }
}

fn query_addresses(deps: Deps) -> StdResult<AddressesResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(AddressesResponse { total_addresses: state.total_addresses })
}


fn query_rewards(deps: Deps) -> StdResult<RewardsResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(RewardsResponse { total_rewards: state.total_rewards })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies_with_balance, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(17, value.count);
    }
}
