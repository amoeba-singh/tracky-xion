#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{
    BuyCreditsMessage, ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg, RegisterUserMessage,
    TransactionsResponse, UseCreditsMessage, UserResponse,
};
use crate::state::{Config, Transaction, User, CONFIG, TRANSACTIONS, USERS};

const CONTRACT_NAME: &str = "crates.io:tracky";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // Validate admin address from message or default to sender
    let admin_addr = msg.admin.map_or(info.sender.clone(), |addr| {
        deps.api.addr_validate(&addr).unwrap_or(info.sender.clone())
    });

    // Initialize contract configuration with admin address
    let config = Config { admin: admin_addr };

    // Set contract version for migration purposes
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    // Store configuration in contract storage
    CONFIG.save(deps.storage, &config)?;

    // Return successful response with initialization metadata
    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("admin", info.sender)
        .add_attribute("credits", (4096 * 4).to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    // Route execution to appropriate handler based on message type
    match msg {
        // ExecuteMsg::RegisterUser(msg) => execute_register_user(deps, env, info, msg),
        ExecuteMsg::IncreaseCredits(msg) => execute_increase_credits(deps, env, info, msg),
        ExecuteMsg::UseCredits(msg) => execute_use_credits(deps, env, info, msg),
    }
}

// pub fn execute_register_user(
//     deps: DepsMut,
//     _env: Env,
//     info:MessageInfo,
//     _msg: RegisterUserMessage,
// ) -> Result<Response, ContractError> {
//     let user_addr = info.sender.clone();
//     if USERS.has(deps.storage, &user_addr) {
//         return Err(ContractError::UserExists {});
//     }

//     let new_user = User {
//         credit_balance: 10,
//     };

//     USERS.save(deps.storage, &user_addr, &new_user)?;

//     Ok(Response::new()
//         .add_attribute("action", "register_user")
//         .add_attribute("user", user_addr.to_string()))
// }

pub fn execute_increase_credits(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: IncreaseCreditsMessage,
) -> Result<Response, ContractError> {
    let buyer_addr = info.sender.clone();

    let credits_to_be_added: u128;

    let mut user = USERS
        .load(deps.storage, &buyer_addr)
        .map_err(|_| ContractError::UserNotFound {})?;

    credits_to_be_added = 5;
    let amount_required = 0.01;

    let payment = info
        .funds
        .iter()
        .find(|coin| coin.denom == "uxion")
        .ok_or_else(|| ContractError::InsufficientAmount {})?;

    let required_amount = (amount_required * 1_000_000.0) as u128;

    if payment.amount.u128() < required_amount {
        return Err(ContractError::InsufficientAmount {});
    }

    user.credit_balance += credits_to_be_added;
    USERS.save(deps.storage, &buyer_addr, &user)?;

    let new_transaction = Transaction {
        credits: credits_to_be_added,
        label: String::from("BOUGHT"),
        timestamp: env.block.time.seconds() as u128,
        amount_used: 0,
    };

    let transactions = TRANSACTIONS
        .may_load(deps.storage, &buyer_addr)?
        .unwrap_or_default();

    let mut updated_transactions = transactions;
    updated_transactions.push(new_transaction);

    TRANSACTIONS.save(deps.storage, &buyer_addr, &updated_transactions)?;

    Ok(Response::new()
        .add_attribute("action", "increase_credits")
        .add_attribute("credits", user.credit_balance.to_string())
        .add_attribute("buyer", buyer_addr.to_string()))
}

pub fn execute_use_credits(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: UseCreditsMessage,
) -> Result<Response, ContractError> {
    let credit_user_addr = info.sender.clone();

    let mut user = USERS
        .load(deps.storage, &credit_user_addr)
        .map_err(|_| ContractError::UserNotFound {})?;

    if user.credit_balance < msg.credits {
        return Err(ContractError::InsufficientCredits {});
    }

    user.credit_balance -= msg.credits;

    USERS.save(deps.storage, &credit_user_addr, &user)?;

    let new_transaction = Transaction {
        credits: msg.credits,
        label: String::from("USED"),
        timestamp: env.block.time.seconds() as u128,
        amount_used: 0,
    };

    let transactions = TRANSACTIONS
        .may_load(deps.storage, &credit_user_addr)?
        .unwrap_or_default();

    let mut updated_transactions = transactions;
    updated_transactions.push(new_transaction);

    TRANSACTIONS.save(deps.storage, &credit_user_addr, &updated_transactions)?;

    Ok(Response::new()
        .add_attribute("action", "used_credits")
        .add_attribute("credits", msg.credits.to_string())
        .add_attribute("credit user", credit_user_addr.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig {} => query_config(deps),
        QueryMsg::GetUser { address } => query_user(deps, address),
        QueryMsg::GetTransactions { address } => query_transactions(deps, address),
    }
}
fn query_config(deps: Deps) -> StdResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    to_json_binary(&ConfigResponse { config })
}

fn query_user(deps: Deps, address: Addr) -> StdResult<Binary> {
    let user = USERS.load(deps.storage, &address)?;
    to_json_binary(&UserResponse { user })
}

fn query_transactions(deps: Deps, address: Addr) -> StdResult<Binary> {
    let transactions = TRANSACTIONS.load(deps.storage, &address)?;
    to_json_binary(&TransactionsResponse { transactions })
}
