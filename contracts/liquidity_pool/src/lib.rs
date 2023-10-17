#![no_std]

mod test;
mod lptoken;


use num_integer::Roots;
use soroban_sdk::{
    contract, contractimpl, contractmeta, Address, BytesN, ConversionError, Env, IntoVal,
    TryFromVal, Val, token, String,
};
use lptoken::create_contract;

const MINIMUN_LIQUIDITY:i128 = 1000;

#[derive(Clone, Copy)]
#[repr(u32)]
pub enum DataKey {
    TokenA = 0,
    TokenB = 1,
    TokenShare = 2,
    TotalShares = 3,
    ReserveA = 4,
    ReserveB = 5,
    KLast = 6,
    IsPoolInitialize=7
}




impl TryFromVal<Env, DataKey> for Val {
    type Error = ConversionError;

    fn try_from_val(_env: &Env, v: &DataKey) -> Result<Self, Self::Error> {
        Ok((*v as u32).into())
    }
}

fn check_pool_initialize(e: &Env){
    
    let is_init = e.storage().instance().has(&DataKey::IsPoolInitialize);

    if is_init {
        panic!("Pool already initialized");
    }else{

        e.storage().instance().set(&DataKey::IsPoolInitialize, &true);
    }
    
}

fn get_token_a(e: &Env) -> Address {
    e.storage().instance().get(&DataKey::TokenA).unwrap()
    
}

fn get_token_b(e: &Env) -> Address {
    e.storage().instance().get(&DataKey::TokenB).unwrap()
}

fn get_token_share(e: &Env) -> Address {
    e.storage().instance().get(&DataKey::TokenShare).unwrap()
}

fn get_total_shares(e: &Env) -> i128 {
    e.storage().instance().get(&DataKey::TotalShares).unwrap()
}

fn get_reserve_a(e: &Env) -> i128 {
    e.storage().instance().get(&DataKey::ReserveA).unwrap()
}

fn get_reserve_b(e: &Env) -> i128 {
    e.storage().instance().get(&DataKey::ReserveB).unwrap()
}

fn get_balance(e: &Env, contract: Address) -> i128 {
    token::Client::new(e, &contract).balance(&e.current_contract_address())
}

fn get_balance_a(e: &Env) -> i128 {
    get_balance(e, get_token_a(e))
}

fn get_balance_b(e: &Env) -> i128 {
    get_balance(e, get_token_b(e))
}

fn get_balance_shares(e: &Env) -> i128 {
    get_balance(e, get_token_share(e))
}

fn get_lp_balance(e:&Env,id:Address)->i128{
    token::Client::new(e,&get_token_share(&e)).balance(&id)
}

fn get_k_last(e: &Env)->i128 {
    e.storage().instance().get(&DataKey::KLast).unwrap()
}


fn put_token_a(e: &Env, contract: Address) {
    e.storage().instance().set(&DataKey::TokenA, &contract);
}

fn put_token_b(e: &Env, contract: Address) {
    e.storage().instance().set(&DataKey::TokenB, &contract);
}

fn put_token_share(e: &Env, contract: Address) {
    e.storage().instance().set(&DataKey::TokenShare, &contract);
}

fn put_total_shares(e: &Env, amount: i128) {
    e.storage().instance().set(&DataKey::TotalShares, &amount)
}


fn put_reserve_a(e: &Env, amount: i128) {
    e.storage().instance().set(&DataKey::ReserveA, &amount)
}

fn put_reserve_b(e: &Env, amount: i128) {
    e.storage().instance().set(&DataKey::ReserveB, &amount)
}

fn put_k_last(e: &Env, amount: i128) {
    e.storage().instance().set(&DataKey::KLast, &amount)
}

fn burn_shares(e: &Env,id:Address, amount: i128) {
    let total = get_total_shares(e);
    let share_contract = get_token_share(e);

    lptoken::Client::new(e, &share_contract).burn(&id, &amount);
    
    put_total_shares(e, total - amount);
}

fn mint_shares(e: &Env, to: Address, amount: i128) {
    let total = get_total_shares(e);
    let share_contract_id = get_token_share(e);

    lptoken::Client::new(e, &share_contract_id).mint(&to, &amount);
    put_total_shares(e, total + amount);
}

fn transfer(e: &Env, token: Address, to: Address, amount: i128) {
    token::Client::new(e, &token).transfer(&e.current_contract_address(), &to, &amount);
}

fn transfer_a(e: &Env, to: Address, amount: i128) {
    transfer(e, get_token_a(e), to, amount);
}

fn transfer_b(e: &Env, to: Address, amount: i128) {
    transfer(e, get_token_b(e), to, amount);
}

fn total_lp_supply(e:&Env)->i128{
    let share_contract_id = get_token_share(e);
    lptoken::Client::new(e, &share_contract_id).total_supply()
}




pub fn get_amount_out(
    amount_in: i128,
    reserve_in: i128,
    reserve_out: i128
)->i128 {
    assert!(amount_in > 0, "Amount must be grater then 0");
    assert!(reserve_in > 0 && reserve_out > 0, "ERROR_INSUFFICIENT_LIQUIDITY");

    let amount_in_with_fee = amount_in * 9975;
    let numerator = amount_in_with_fee * reserve_out;
    let denominator = reserve_in * 10000 + amount_in_with_fee;

    numerator / denominator
}

fn get_deposit_amounts(
    desired_a: i128,
    min_a: i128,
    desired_b: i128,
    min_b: i128,
    reserve_a: i128,
    reserve_b: i128,
) -> (i128, i128) {
    if reserve_a == 0 && reserve_b == 0 {
        return (desired_a, desired_b);
    }

    let amount_b = desired_a * reserve_b / reserve_a;
    if amount_b <= desired_b {
        if amount_b < min_b {
            panic!("amount_b less than min")
        }
        (desired_a, amount_b)
    } else {
        let amount_a = desired_b * reserve_a / reserve_b;
        if amount_a > desired_a || desired_a < min_a {
            panic!("amount_a invalid")
        }
        (amount_a, desired_b)
    }
}

// Metadata that is added on to the WASM custom section
contractmeta!(
    key = "Description",
    val = "Constant product AMM with a .3% swap fee"
);

pub trait LiquidityPoolTrait {
    // Sets the token contract addresses for this pool
    fn initialize(e: Env, token_wasm_hash: BytesN<32>, token_a: Address, token_b: Address,lptokenname:String,lptokensymbol:String);

    // Returns the token contract address for the pool share token
    fn share_id(e: Env) -> Address;

    // Deposits token_a and token_b. Also mints pool shares for the "to" Identifier. The amount minted
    // is determined based on the difference between the reserves stored by this contract, and
    // the actual balance of token_a and token_b for this contract.
    fn deposit(e: Env, to: Address, desired_a: i128, min_a: i128, desired_b: i128, min_b: i128)->(i128,i128,i128,i128,i128);

    // If "swap_x_to_y" is true, the swap will buy token_a and sell token_b. This is flipped if "swap_x_to_y" is false.
    // "out" is the amount being bought, with in_max being a safety to make sure you receive at least that amount.
    // swap will transfer the selling token "to" to this contract, and then the contract will transfer the buying token to "to".
    fn swap_exact_input(e: Env, to: Address, swap_x_to_y: bool, out: i128, in_max: i128)->i128;
    
    fn swap_exact_output(e: Env, to: Address, swap_x_to_y: bool, out: i128, in_max: i128)->i128;
    // transfers share_amount of pool share tokens to this contract, burns all pools share tokens in this contracts, and sends the
    // corresponding amount of token_a and token_b to "to".
    // Returns amount of both tokens withdrawn
    fn withdraw(e: Env, to: Address, share_amount: i128, min_a: i128, min_b: i128) -> (i128, i128);

    fn get_rsrvs(e: Env) -> (i128, i128);

    fn get_lptoken_balance(e: Env,id:Address)->i128;

    fn get_contract_lptoken_balance(e: Env)->i128;

    fn get_k(e: Env) -> i128;

}

#[contract]
struct LiquidityPool;

#[contractimpl]
impl LiquidityPoolTrait for LiquidityPool {
    fn initialize(e: Env, token_wasm_hash: BytesN<32>, token_a: Address, token_b: Address,lptokenname:String,lptokensymbol:String) {
        // if token_a >= token_b {
        //     panic!("token_a must be less than token_b");
        // }
        
        check_pool_initialize(&e);
    
        let share_contract = create_contract(&e, token_wasm_hash, &token_a, &token_b);
        lptoken::Client::new(&e, &share_contract).initialize(
            &e.current_contract_address(),
            &8u32,
            &lptokenname.into_val(&e),
            &&lptokensymbol.into_val(&e),
        );

        put_token_a(&e, token_a);
        put_token_b(&e, token_b);
        put_token_share(&e, share_contract.try_into().unwrap());
        put_total_shares(&e, 0);
        put_reserve_a(&e, 0);
        put_reserve_b(&e, 0);
        put_k_last(&e, 0);
    }

    fn share_id(e: Env) -> Address {
        get_token_share(&e)
    }

    fn deposit(e: Env, to: Address, desired_a: i128, min_a: i128, desired_b: i128, min_b: i128)->(i128,i128,i128,i128,i128){
        // Depositor needs to authorize the deposit
        to.require_auth();

        let (reserve_a, reserve_b) = (get_reserve_a(&e), get_reserve_b(&e));

        // Calculate deposit amounts
        let amounts = get_deposit_amounts(desired_a, min_a, desired_b, min_b, reserve_a, reserve_b);

        let token_a_client = token::Client::new(&e, &get_token_a(&e));
        let token_b_client = token::Client::new(&e, &get_token_b(&e));

        token_a_client.transfer(&to, &e.current_contract_address(), &amounts.0);
        token_b_client.transfer(&to, &e.current_contract_address(), &amounts.1);

        // Now calculate how many new pool shares to mint
        let (balance_a, balance_b) = (get_balance_a(&e), get_balance_b(&e));
        let total_shares = total_lp_supply(&e);

        let zero = 0;
        let new_total_shares = if total_shares > zero {
            let shares_a = ((balance_a - reserve_a) * total_shares) / reserve_a;
            let shares_b = ((balance_b - reserve_b) * total_shares) / reserve_b;
            let shares = shares_a.min(shares_b);
            assert!(shares > 0, "ERROR_INSUFFICIENT_LIQUIDITY_MINTED");
            shares
        } else {
            let shares = (balance_a * balance_b).sqrt();
            assert!(shares > MINIMUN_LIQUIDITY, "ERROR_INSUFFICIENT_LIQUIDITY_MINTED");
            mint_shares(&e, e.current_contract_address(),  MINIMUN_LIQUIDITY);
            shares - MINIMUN_LIQUIDITY
        };

        mint_shares(&e, to, new_total_shares);
        put_reserve_a(&e, balance_a);
        put_reserve_b(&e, balance_b);
        put_k_last(&e, balance_a * balance_b);
        (new_total_shares,amounts.0,amounts.1,balance_a,balance_b)
    }

    fn swap_exact_input(e: Env, to: Address, swap_x_to_y: bool, x_in: i128,y_min_out: i128)->i128{
        
        to.require_auth();

        let (reserve_a, reserve_b) = (get_reserve_a(&e), get_reserve_b(&e));
        let (reserve_x, reserve_y) = if swap_x_to_y {
            (reserve_a, reserve_b)
        } else {
            (reserve_b, reserve_a)
        };


        let amount_out = if swap_x_to_y{
            get_amount_out(x_in, reserve_x, reserve_y)
        }else{
            get_amount_out(x_in, reserve_x, reserve_y)
        };


        if amount_out < y_min_out {
            panic!("Not satisfied minimum out")
        }

        // Transfer the amount being sold to the contract
        let sell_token = if swap_x_to_y {
            get_token_a(&e)
        } else {
            get_token_b(&e)
        };

        let sell_token_client = token::Client::new(&e, &sell_token);
        sell_token_client.transfer(&to, &e.current_contract_address(), &x_in);

        let (balance_a, balance_b) = (get_balance_a(&e), get_balance_b(&e));

        // residue_numerator and residue_denominator are the amount that the invariant considers after
        // deducting the fee, scaled up by 1000 to avoid fractions
        let residue_numerator = 9975;
        let residue_denominator = 10000;
        let zero = 0;

        let new_invariant_factor = |balance: i128, reserve: i128, out: i128| {
            let delta = balance - reserve - out;
            let adj_delta = if delta > zero {
                residue_numerator * delta
            } else {
                residue_denominator * delta
            };
            residue_denominator * reserve + adj_delta
        };

        let (out_a, out_b) = if swap_x_to_y { (0,amount_out) } else { (amount_out, 0) };

        let new_inv_a = new_invariant_factor(balance_a, reserve_a, out_a);
        let new_inv_b = new_invariant_factor(balance_b, reserve_b, out_b);

        let old_inv_a = residue_denominator * reserve_a;
        let old_inv_b = residue_denominator * reserve_b;



        if new_inv_a * new_inv_b < old_inv_a * old_inv_b {
            panic!("constant product invariant does not hold");
        }

        if swap_x_to_y {
            transfer_b(&e, to, out_b);
        } else {
            transfer_a(&e, to, out_a);
        }

        put_reserve_a(&e, balance_a - out_a);
        put_reserve_b(&e, balance_b - out_b);
        amount_out
    }

    // x * y =k

    // (x + dx) * (y - dy )= k


    fn swap_exact_output(e: Env, to: Address, buy_a: bool, y_out: i128, x_max_in: i128)->i128 {
        to.require_auth();

        let (reserve_a, reserve_b) = (get_reserve_a(&e), get_reserve_b(&e));
        let (reserve_sell, reserve_buy) = if buy_a {
            (reserve_b, reserve_a)
        } else {
            (reserve_a, reserve_b)
        };

        // First calculate how much needs to be sold to buy amount out from the pool
        let n = reserve_sell * y_out * 10000;
        let d = (reserve_buy - y_out) * 9975;
        let sell_amount = (n / d) + 1;
        if sell_amount > x_max_in {
            panic!("in amount is over max")
        }

        // Transfer the amount being sold to the contract
        let sell_token = if buy_a {
            get_token_b(&e)
        } else {
            get_token_a(&e)
        };
        let sell_token_client = token::Client::new(&e, &sell_token);
        sell_token_client.transfer(&to, &e.current_contract_address(), &sell_amount);

        let (balance_a, balance_b) = (get_balance_a(&e), get_balance_b(&e));

        // residue_numerator and residue_denominator are the amount that the invariant considers after
        // deducting the fee, scaled up by 1000 to avoid fractions
        let residue_numerator = 9975;
        let residue_denominator = 10000;
        let zero = 0;

        let new_invariant_factor = |balance: i128, reserve: i128, out: i128| {
            let delta = balance - reserve - out;
            let adj_delta = if delta > zero {
                residue_numerator * delta
            } else {
                residue_denominator * delta
            };
            residue_denominator * reserve + adj_delta
        };

        let (out_a, out_b) = if buy_a { (y_out, 0) } else { (0, y_out) };

        let new_inv_a = new_invariant_factor(balance_a, reserve_a, out_a);
        let new_inv_b = new_invariant_factor(balance_b, reserve_b, out_b);
        let old_inv_a = residue_denominator * reserve_a;
        let old_inv_b = residue_denominator * reserve_b;

        if new_inv_a * new_inv_b < old_inv_a * old_inv_b {
            panic!("constant product invariant does not hold");
        }

        if buy_a {
            transfer_a(&e, to, out_a);
        } else {
            transfer_b(&e, to, out_b);
        }

        put_reserve_a(&e, balance_a - out_a);
        put_reserve_b(&e, balance_b - out_b);

        sell_amount
    }


    fn withdraw(e: Env, to: Address, liquidity: i128,amount_x_min: i128,amount_y_min: i128)->(i128,i128)  {
        to.require_auth();

        // First transfer the pool shares that need to be redeemed
        let share_token_client = token::Client::new(&e, &get_token_share(&e));

        let (balance_a, balance_b) = (get_balance_a(&e), get_balance_b(&e));

        

        assert!(share_token_client.balance(&to) >= liquidity,"Don't have a enough liquidity to remove");

        let total_shares = total_lp_supply(&e);

        // Now calculate the withdraw amounts
        let out_a = (balance_a * liquidity) / total_shares;
        let out_b = (balance_b * liquidity) / total_shares;

        if out_a < amount_x_min || out_b < amount_y_min {
            panic!("min not satisfied");
        }


        burn_shares(&e, to.clone(),liquidity);
        transfer_a(&e, to.clone(), out_a);
        transfer_b(&e, to, out_b);
        put_reserve_a(&e, balance_a - out_a);
        put_reserve_b(&e, balance_b - out_b);
        put_k_last(&e,get_reserve_a(&e) * get_reserve_a(&e));

        (out_a, out_b)
        
    }

    fn get_rsrvs(e: Env) -> (i128, i128) {
        (get_reserve_a(&e), get_reserve_b(&e))
    }

    fn get_k(e: Env) -> i128 {
        get_k_last(&e)
    }

    fn get_lptoken_balance(e: Env,id:Address)->i128{
        get_lp_balance(&e, id)
    }

    fn get_contract_lptoken_balance(e: Env)->i128{
        total_lp_supply(&e)
    }
}
