#![no_std]
use soroban_sdk::{contract, contractimpl, token, vec, Address, Env, IntoVal, String, Vec};

#[contract]
pub struct Dex;

#[contractimpl]
impl Dex {
  pub fn hello(env: Env, to: String) -> Vec<String> {
    vec![&env, String::from_str(&env, "Hello"), to]
  }

  pub fn swap(
    env: Env,
    a: Address,
    b: Address,
    token_a: Address,
    token_b: Address,
    amount_a: i128,
    min_b_for_a: i128,
    amount_b: i128,
    min_a_for_b: i128,
  ) {
    if amount_b < min_b_for_a {
      panic!("not enough tokens B for token A");
    }

    if amount_a < min_a_for_b {
      panic!("not enough tokens A for token B");
    }

    a.require_auth_for_args(
      (token_a.clone(), token_b.clone(), amount_a, min_b_for_a).into_val(&env),
    );

    b.require_auth_for_args(
      (token_b.clone(), token_a.clone(), amount_b, min_a_for_b).into_val(&env),
    );
    move_token(&env, &token_a, &a, &b, amount_a, min_a_for_b);
    move_token(&env, &token_b, &b, &a, amount_b, min_b_for_a);
  }
}

fn move_token(
  env: &Env,
  token: &Address,
  from: &Address,
  to: &Address,
  max_spend_amount: i128,
  transfer_amount: i128,
) {
  let token = token::Client::new(env, token);
  let contract_address = env.current_contract_address();

  token.transfer(from, &contract_address, &max_spend_amount);
  token.transfer(&contract_address, to, &transfer_amount);
  token.transfer(
    &contract_address,
    from,
    &(max_spend_amount - transfer_amount),
  );
}

mod test;
