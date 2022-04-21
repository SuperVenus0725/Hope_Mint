use cosmwasm_std::{
    entry_point, from_binary, to_binary,   CosmosMsg, Deps, DepsMut,Binary,Decimal,
    Env, MessageInfo, QueryRequest, Response, StdResult, Uint128, WasmMsg, WasmQuery, 
};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, HopeMintMsg, InstantiateMsg, QueryMsg};
use crate::state::{
    config, config_read, read_maximum_nft, read_nft_address, read_token_address, read_token_count,
    read_user_info, read_users, store_maximum_nft, store_nft_address, store_token_address,
    store_token_count, store_users, Metadata, State, TokenCount, UserInfo,
};
use cw20::{BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg, Cw20ReceiveMsg};
use cw721::{ContractInfoResponse, TokensResponse};
use cw721_base::{ExecuteMsg as Cw721BaseExecuteMsg, MintMsg, QueryMsg as Cw721BaseQueryMsg};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        total_nft: Uint128::new(0),
        owner: info.sender.to_string(),
    };
    config(deps.storage).save(&state)?;
    Ok(Response::default())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Receive(cw20_receive_msg) => execute_receive(deps, env, info, cw20_receive_msg),
        ExecuteMsg::SetTokenAddress { address } => execute_set_address(deps, info, address),
        ExecuteMsg::SetNftAddress { address } => execute_set_nft_address(deps, info, address),
        ExecuteMsg::BuyToken { amount } => execute_buy_token(deps, info, amount),
        ExecuteMsg::SetMaximumNft { amount } => execute_maxium_nft(deps, info, amount),
    }
}

fn execute_receive(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    cw20_receive_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let mut state = config_read(deps.storage).load()?;

    if state.total_nft >= Uint128::new(2000) {
        return Err(ContractError::MintEnded {});
    }

    //nft number of user(for the user who first mint)
    let mut user_token_count = Uint128::new(0);
    //address of sender who will possess nft
    let sender_address = deps.api.addr_validate(cw20_receive_msg.sender.as_str())?;
    let token_count = read_token_count(deps.storage, &sender_address);

    let maximum_nft = read_maximum_nft(deps.storage)?;

    if token_count != None {
        let user_token_number = token_count.unwrap();
        if user_token_number >= maximum_nft {
            return Err(ContractError::MintExceeded {});
        } else {
            user_token_count = user_token_number;
        }
    }

    user_token_count += Uint128::new(1);

    let token_address = read_token_address(deps.storage)?;
    let nft_address = read_nft_address(deps.storage)?;

    let msg: HopeMintMsg = from_binary(&cw20_receive_msg.msg)?;

    let token_id: String = ["Hope".to_string(), state.total_nft.to_string()].join(".");
    state.total_nft += Uint128::new(1);
    config(deps.storage).save(&state)?;

    let user_nft = read_user_info(deps.storage, &sender_address);

    if user_nft == None {
        let mut owned_nft = vec![];
        owned_nft.push(token_id.clone());
        store_users(
            deps.storage,
            &sender_address,
            UserInfo {
                address: sender_address.to_string(),
                nft: owned_nft,
            },
        )?;
    } else {
        let mut owned_nft = read_user_info(deps.storage, &sender_address).unwrap();
        owned_nft.nft.push(token_id.clone());
        store_users(
            deps.storage,
            &sender_address,
            UserInfo {
                address: sender_address.to_string(),
                nft: owned_nft.nft,
            },
        )?;
    }

    store_token_count(deps.storage, &sender_address, user_token_count)?;

    let meta_data = Metadata {
        name: msg.name,
        description: msg.description,
        external_link: msg.external_link,
        royalties: msg.royalties,
        init_price: msg.init_price,
    };

    Ok(Response::new()
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: token_address.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: state.owner,
                amount: cw20_receive_msg.amount,
            })?,
        }))
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: nft_address.to_string(),
            msg: to_binary(&Cw721BaseExecuteMsg::Mint(MintMsg {
                //::<Metadata>
                token_id: token_id.clone(),
                owner: cw20_receive_msg.sender,
                token_uri: msg.image_uri,
                extension: meta_data.clone(),
            }))?,
            funds: vec![],
        })))
}

fn execute_maxium_nft(
    deps: DepsMut,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let state = config_read(deps.storage).load()?;
    if state.owner != info.sender.to_string() {
        return Err(ContractError::Unauthorized {});
    }
    store_maximum_nft(deps.storage, &amount)?;
    Ok(Response::default())
}

fn execute_set_address(
    deps: DepsMut,
    info: MessageInfo,
    address: String,
) -> Result<Response, ContractError> {
    let state = config_read(deps.storage).load()?;
    if state.owner != info.sender.to_string() {
        return Err(ContractError::Unauthorized {});
    }
    let token_address = deps.api.addr_validate(&address)?;
    store_token_address(deps.storage, &token_address)?;
    Ok(Response::default())
}

fn execute_set_nft_address(
    deps: DepsMut,
    info: MessageInfo,
    address: String,
) -> Result<Response, ContractError> {
    let state = config_read(deps.storage).load()?;
    if state.owner != info.sender.to_string() {
        return Err(ContractError::Unauthorized {});
    }
    let nft_address = deps.api.addr_validate(&address)?;
    store_nft_address(deps.storage, &nft_address)?;
    Ok(Response::default())
}

pub fn execute_buy_token(
    deps: DepsMut,
    // env:Env,
    info: MessageInfo,
    amount: i32,
) -> Result<Response, ContractError> {
    let token_address = read_token_address(deps.storage)?;
    let res = Response::new()
        .add_attribute("action", "buy")
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: token_address.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Mint {
                recipient: String::from(info.sender.as_str()),
                amount: Uint128::new(amount as u128),
            })?,
        }));
    Ok(res)
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetBalance { address } => to_binary(&query_get_balance(deps, address)?),
        QueryMsg::GetTokenAddress {} => to_binary(&query_get_address(deps)?),
        QueryMsg::GetAllUsers {} => to_binary(&query_get_users(deps)?),
        QueryMsg::GetUserInfo { address } => to_binary(&query_user_info(deps, address)?),
        QueryMsg::GetContractInfo {} => to_binary(&query_contract_info(deps)?),
        QueryMsg::GetTokenInfo { address } => to_binary(&query_token_info(deps, address)?),
        QueryMsg::GetTokenCount { address } => to_binary(&query_token_count(deps, address)?),
        QueryMsg::GetNftAddress {} => to_binary(&query_nft_address(deps)?),
        QueryMsg::GetMaximumNft {} => to_binary(&query_maximum_nft(deps)?),
        QueryMsg::GetStateInfo {} => to_binary(& query_get_info(deps)?)
    }
}

pub fn query_contract_info(deps: Deps) -> StdResult<ContractInfoResponse> {
    let nft_address = read_nft_address(deps.storage)?;
    let contract_info = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: nft_address.to_string(),
        msg: to_binary(&Cw721BaseQueryMsg::ContractInfo {})?,
    }))?;
    Ok(contract_info)
}

pub fn query_token_count(deps: Deps, address: String) -> StdResult<TokenCount> {
    let user_address = deps.api.addr_validate(address.as_str())?;
    let state = config_read(deps.storage).load()?;
    let result = read_token_count(deps.storage, &user_address);
    if result == None {
        Ok(TokenCount {
            total_nft: state.total_nft,
            owned_nft_number: Uint128::new(0),
        })
    } else {
        Ok(TokenCount {
            total_nft: state.total_nft,
            owned_nft_number: result.unwrap(),
        })
    }
}

pub fn query_token_info(deps: Deps, address: String) -> StdResult<TokensResponse> {
    let owner = deps.api.addr_validate(&address)?;
    let nft_address = read_nft_address(deps.storage)?;
    let balance = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: nft_address.to_string(),
        msg: to_binary(&Cw721BaseQueryMsg::Tokens {
            owner: owner.to_string(),
            start_after: None,
            limit: None,
        })?,
    }))?;
    Ok(balance)
}

pub fn query_get_balance(deps: Deps, address: String) -> StdResult<BalanceResponse> {
    deps.api.addr_validate(&address)?;
    let token_address = read_token_address(deps.storage)?;
    let balance = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: token_address.to_string(),
        msg: to_binary(&Cw20QueryMsg::Balance { address })?,
    }))?;
    Ok(balance)
}

pub fn query_get_address(deps: Deps) -> StdResult<String> {
    let token_address = read_token_address(deps.storage)?;
    let result = token_address.to_string();
    Ok(result)
}

pub fn query_nft_address(deps: Deps) -> StdResult<String> {
    let nft_address = read_nft_address(deps.storage)?;
    let result = nft_address.to_string();
    Ok(result)
}

pub fn query_get_users(deps: Deps) -> StdResult<Vec<String>> {
    let users = read_users(deps.storage)?;
    Ok(users)
}

pub fn query_user_info(deps: Deps, address: String) -> StdResult<UserInfo> {
    let user = read_user_info(deps.storage, &deps.api.addr_validate(&address)?).unwrap();
    Ok(user)
}

pub fn query_maximum_nft(deps: Deps) -> StdResult<Uint128> {
    let maximum_nft = read_maximum_nft(deps.storage)?;
    Ok(maximum_nft)
}

pub fn query_get_info(deps:Deps) -> StdResult<State>{
    let state =  config_read(deps.storage).load()?;
    Ok(state)
}

#[cfg(test)]
mod tests {
    use crate::state::Royalty;

    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{CosmosMsg};

    #[test]
    fn buy_token() {
        let mut deps = mock_dependencies();
        let instantiate_msg = InstantiateMsg {};
        let info = mock_info("creator", &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();
        assert_eq!(0, res.messages.len());

        let info = mock_info("creator", &[]);
        let address = String::from("token_address");
        let message = ExecuteMsg::SetTokenAddress { address: address };
        execute(deps.as_mut(), mock_env(), info, message).unwrap();

        let res = query_get_address(deps.as_ref()).unwrap();
        assert_eq!(res, "token_address");

        let info = mock_info("sender", &[]);
        let amount = 100;
        let message = ExecuteMsg::BuyToken { amount };
        let res = execute(deps.as_mut(), mock_env(), info, message).unwrap();
        let message = res.messages[0].clone().msg;
        assert_eq!(
            message,
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: String::from("token_address"),
                msg: to_binary(&Cw20ExecuteMsg::Mint {
                    recipient: String::from("sender"),
                    amount: Uint128::new(100)
                })
                .unwrap(),
                funds: vec![]
            })
        );
    }

    #[test]
    fn receive() {
        let mut deps = mock_dependencies();
        let instantiate_msg = InstantiateMsg {};
        let info = mock_info("creator", &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();
        assert_eq!(0, res.messages.len());

        let token_count = query_token_count(deps.as_ref(), "sender".to_string()).unwrap();
        assert_eq!(
            token_count,
            TokenCount {
                owned_nft_number: Uint128::new(0),
                total_nft: Uint128::new(0)
            }
        );

        let info = mock_info("creator", &[]);
        let msg = ExecuteMsg::SetTokenAddress {
            address: "token_contract".to_string(),
        };
        execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::SetNftAddress {
            address: "nft_contract".to_string(),
        };
        execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::SetMaximumNft {
            amount: Uint128::new(5),
        };
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        let maximum_nft = query_maximum_nft(deps.as_ref()).unwrap();
        assert_eq!(maximum_nft, Uint128::new(5));

        let info = mock_info("token_contract", &[]);
        let mint_msg = HopeMintMsg {
            name: Some("hope1".to_string()),
            image_uri: Some("https://hope1".to_string()),
            external_link: Some("https://hope1".to_string()),
            description: Some("Galaxy NFT".to_string()),
            init_price: Some(Uint128::new(1)),
            nft_addr: Some("Nft_address".to_string()),
            royalties: Some(vec![Royalty {
                address: "creator".to_string(),
                royalty_rate: Decimal::from_atomics(3u64, 1).unwrap(),
            }]),
        };
        let meta_data = Metadata {
            name: mint_msg.clone().name,
            description: mint_msg.clone().description,
            external_link: mint_msg.clone().external_link,
        
            royalties: mint_msg.clone().royalties,
            init_price: mint_msg.clone().init_price,
        };

        let message = ExecuteMsg::Receive(Cw20ReceiveMsg {
            sender: "sender".to_string(),
            amount: Uint128::new(1),
            msg: to_binary(&mint_msg).unwrap(),
        });
        let res = execute(deps.as_mut(), mock_env(), info, message).unwrap();
        assert_eq!(res.messages.len(), 2);
        assert_eq!(
            res.messages[0].msg,
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "token_contract".to_string(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: "creator".to_string(),
                    amount: Uint128::new(1),
                })
                .unwrap()
            })
        );
        assert_eq!(
            res.messages[1].msg,
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "nft_contract".to_string(),
                msg: to_binary(&Cw721BaseExecuteMsg::Mint(MintMsg {
                    //::<Metadata>
                    token_id: "Hope.0".to_string(),
                    owner: "sender".to_string(),
                    token_uri: mint_msg.image_uri,
                    extension: meta_data.clone(),
                }))
                .unwrap(),
                funds: vec![],
            })
        );
        let token_count = query_token_count(deps.as_ref(), "sender".to_string()).unwrap();
        assert_eq!(
            token_count,
            TokenCount {
                total_nft: Uint128::new(1),
                owned_nft_number: Uint128::new(1)
            }
        );
        for i in 0..4 {
            let info = mock_info("token_contract", &[]);
            let mint_msg = HopeMintMsg {
                name: Some("hope1".to_string()),
                image_uri: Some("https://hope1".to_string()),
                external_link: Some("https://hope1".to_string()),
                description: Some("Galaxy NFT".to_string()),
            
                init_price: Some(Uint128::new(1)),
                nft_addr: Some("Nft_address".to_string()),
                royalties: Some(vec![Royalty {
                    address: "creator".to_string(),
                    royalty_rate: Decimal::from_atomics(3u64, 1).unwrap(),
                }]),
            };
              Metadata {
                name: mint_msg.clone().name,
                description: mint_msg.clone().description,
                external_link: mint_msg.clone().external_link,
               
                royalties: mint_msg.clone().royalties,
                init_price: mint_msg.clone().init_price,
            };

            let message = ExecuteMsg::Receive(Cw20ReceiveMsg {
                sender: "sender".to_string(),
                amount: Uint128::new(1),
                msg: to_binary(&mint_msg).unwrap(),
            });
             execute(deps.as_mut(), mock_env(), info, message).unwrap();
        }
        let token_count = query_token_count(deps.as_ref(), "sender".to_string()).unwrap();
        assert_eq!(
            token_count,
            TokenCount {
                total_nft: Uint128::new(5),
                owned_nft_number: Uint128::new(5)
            }
        );
        let info = mock_info("token_contract", &[]);
        let mint_msg = HopeMintMsg {
            name: Some("hope1".to_string()),
            image_uri: Some("https://hope1".to_string()),
            external_link: Some("https://hope1".to_string()),
            description: Some("Galaxy NFT".to_string()),
            init_price: Some(Uint128::new(1)),
            nft_addr: Some("Nft_address".to_string()),
            royalties: Some(vec![Royalty {
                address: "creator".to_string(),
                royalty_rate: Decimal::from_atomics(3u64, 1).unwrap(),
            }]),
        };
        Metadata {
            name: mint_msg.clone().name,
            description: mint_msg.clone().description,
            external_link: mint_msg.clone().external_link,
        
            royalties: mint_msg.clone().royalties,
            init_price: mint_msg.clone().init_price,
        };

        let message = ExecuteMsg::Receive(Cw20ReceiveMsg {
            sender: "sender1".to_string(),
            amount: Uint128::new(1),
            msg: to_binary(&mint_msg).unwrap(),
        });
        execute(deps.as_mut(), mock_env(), info, message).unwrap();
        let token_count = query_token_count(deps.as_ref(), "sender".to_string()).unwrap();
        assert_eq!(
            token_count,
            TokenCount {
                total_nft: Uint128::new(6),
                owned_nft_number: Uint128::new(5)
            }
        );

        for i in 7..2001 {
            let info = mock_info("token_contract", &[]);
            let mint_msg = HopeMintMsg {
                name: Some("hope1".to_string()),
                image_uri: Some("https://hope1".to_string()),
                external_link: Some("https://hope1".to_string()),
                description: Some("Galaxy NFT".to_string()),
            
                init_price: Some(Uint128::new(1)),
                nft_addr: Some("Nft_address".to_string()),
                royalties: Some(vec![Royalty {
                    address: "creator".to_string(),
                    royalty_rate: Decimal::from_atomics(3u64, 1).unwrap(),
                }]),
            };
            Metadata {
                name: mint_msg.clone().name,
                description: mint_msg.clone().description,
                external_link: mint_msg.clone().external_link,
               
                royalties: mint_msg.clone().royalties,
                init_price: mint_msg.clone().init_price,
            };

            let message = ExecuteMsg::Receive(Cw20ReceiveMsg {
                sender: i.to_string() + "sender",
                amount: Uint128::new(1),
                msg: to_binary(&mint_msg).unwrap(),
            });
            execute(deps.as_mut(), mock_env(), info, message).unwrap();
        }

        let users = query_get_users(deps.as_ref()).unwrap();
        assert_eq!(users.len(), 1996);

        let token_count = query_token_count(deps.as_ref(), "2000sender".to_string()).unwrap();
        assert_eq!(
            token_count,
            TokenCount {
                total_nft: Uint128::new(2000),
                owned_nft_number: Uint128::new(1)
            }
        );

        let state = query_get_info(deps.as_ref()).unwrap();
        assert_eq!(state,State{
            owner:"creator".to_string(),
            total_nft:Uint128::new(2000)
        })
    }
}
