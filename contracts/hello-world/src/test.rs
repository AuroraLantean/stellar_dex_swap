#![cfg(test)]
extern crate std;

use super::*;
use soroban_sdk::{
  symbol_short,
  testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
  token, vec, Address, Env, IntoVal, String,
};
use token::Client as TokenClient;
use token::StellarAssetClient as TokenAdminClient;

fn make_token_contract<'a>(env: &Env, admin: &Address) -> (TokenClient<'a>, TokenAdminClient<'a>) {
  let sac = env.register_stellar_asset_contract_v2(admin.clone());
  (
    token::Client::new(env, &sac.address()),
    token::StellarAssetClient::new(env, &sac.address()),
  )
}

fn make_atomic_swap_contract(env: &Env) -> DexClient {
  DexClient::new(env, &env.register_contract(None, Dex))
}

#[test]
fn test() {
  let env = Env::default();
  let contract_id = env.register_contract(None, Dex);
  let client = DexClient::new(&env, &contract_id);

  let words = client.hello(&String::from_str(&env, "Dev"));
  assert_eq!(
    words,
    vec![
      &env,
      String::from_str(&env, "Hello"),
      String::from_str(&env, "Dev"),
    ]
  );
}

#[test]
fn test_dex_swap() {
  let env = Env::default();
  env.mock_all_auths();

  let a = Address::generate(&env);
  let b = Address::generate(&env);

  let token_admin = Address::generate(&env);

  let (token_a, token_a_admin) = make_token_contract(&env, &token_admin);
  let (token_b, token_b_admin) = make_token_contract(&env, &token_admin);
  token_a_admin.mint(&a, &1000);
  token_b_admin.mint(&b, &5000);

  let contract = make_atomic_swap_contract(&env);

  contract.swap(
    &a,
    &b,
    &token_a.address,
    &token_b.address,
    &1000,
    &4500,
    &5000,
    &950,
  );

  assert_eq!(
    env.auths(),
    std::vec![
      (
        a.clone(),
        AuthorizedInvocation {
          function: AuthorizedFunction::Contract((
            contract.address.clone(),
            symbol_short!("swap"),
            (
              token_a.address.clone(),
              token_b.address.clone(),
              1000_i128,
              4500_i128
            )
              .into_val(&env),
          )),
          sub_invocations: std::vec![AuthorizedInvocation {
            function: AuthorizedFunction::Contract((
              token_a.address.clone(),
              symbol_short!("transfer"),
              (a.clone(), contract.address.clone(), 1000_i128,).into_val(&env),
            )),
            sub_invocations: std::vec![]
          }]
        }
      ),
      (
        b.clone(),
        AuthorizedInvocation {
          function: AuthorizedFunction::Contract((
            contract.address.clone(),
            symbol_short!("swap"),
            (
              token_b.address.clone(),
              token_a.address.clone(),
              5000_i128,
              950_i128
            )
              .into_val(&env),
          )),
          sub_invocations: std::vec![AuthorizedInvocation {
            function: AuthorizedFunction::Contract((
              token_b.address.clone(),
              symbol_short!("transfer"),
              (b.clone(), contract.address.clone(), 5000_i128,).into_val(&env),
            )),
            sub_invocations: std::vec![]
          }]
        }
      ),
    ]
  );

  assert_eq!(token_a.balance(&a), 50); //1000-950 = 50 a-token balance
  assert_eq!(token_a.balance(&b), 950); // 950 a-token balance

  assert_eq!(token_b.balance(&a), 4500); //4500  b-token
  assert_eq!(token_b.balance(&b), 500); //5000 - 4500 = 500  b-token
}
