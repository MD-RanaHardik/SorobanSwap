#![no_std]

mod test;

use soroban_sdk::{
    contract, contractimpl, contractmeta, Address, Env, Vec,

};


mod liquiditypool{

    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/soroban_liquidity_pool_contract.wasm"
    );
}


// Metadata that is added on to the WASM custom section
contractmeta!(
    key = "Multihop",
    val = "For swap"
);

pub trait MultihopSwapTrait {
   fn swap_exact_input_doublehop(env:Env,to:Address,pools:Vec<Address>,swap_x_to_y:bool,swap_y_to_z:bool,x_in:i128, z_min_out:i128)->i128;
   fn swap_exact_input_triplehop(env:Env,to:Address,pools:Vec<Address>,swap_x_to_y:bool,swap_y_to_z:bool,swap_z_to_a:bool,x_in:i128, a_min_out:i128)->i128;
   fn swap_exact_input_quadruplehop(env:Env,to:Address,pools:Vec<Address>,swap_x_to_y:bool,swap_y_to_z:bool,swap_z_to_a:bool,swap_a_to_b:bool,x_in:i128, b_min_out:i128)->i128;

   
}

#[contract]
struct MultihopSwap;

#[contractimpl]
impl MultihopSwapTrait for MultihopSwap {
    
    fn swap_exact_input_doublehop(env:Env,to:Address,pools:Vec<Address>,swap_x_to_y:bool,swap_y_to_z:bool,x_in:i128, z_min_out:i128)->i128{

        to.require_auth();
        
        let first_pool = liquiditypool::Client::new(&env, &pools.get(0).unwrap());
        let second_pool = liquiditypool::Client::new(&env, &pools.get(1).unwrap());
        
        
        let y_out = if swap_x_to_y {
            first_pool.swap_exact_input(&to, &true, &x_in, &0)
        }else{
            first_pool.swap_exact_input(&to, &false, &x_in, &0)
        };

        let z_out = if swap_y_to_z { 
            second_pool.swap_exact_input(&to, &true, &y_out, &z_min_out)
        }else{
            second_pool.swap_exact_input(&to, &false, &y_out, &z_min_out)
        };

        
        z_out

    }

    fn swap_exact_input_triplehop(env:Env,to:Address,pools:Vec<Address>,swap_x_to_y:bool,swap_y_to_z:bool,swap_z_to_a:bool,x_in:i128, a_min_out:i128)->i128{
        
        to.require_auth();

        let first_pool = liquiditypool::Client::new(&env, &pools.get(0).unwrap());
        let second_pool = liquiditypool::Client::new(&env, &pools.get(1).unwrap());
        let third_pool = liquiditypool::Client::new(&env, &pools.get(2).unwrap());

        let y_out = if swap_x_to_y {
            first_pool.swap_exact_input(&to, &true, &x_in, &0)
        }else{
            first_pool.swap_exact_input(&to, &false, &x_in, &0)
        };

        let z_out = if swap_y_to_z {
            second_pool.swap_exact_input(&to, &true, &y_out, &0)
        }else{
            second_pool.swap_exact_input(&to, &false, &y_out, &0)
        };

        let a_out = if swap_z_to_a { 
            third_pool.swap_exact_input(&to, &true, &z_out, &a_min_out)
        }else{
            third_pool.swap_exact_input(&to, &false, &z_out, &a_min_out)
        };

        
        a_out
    }

    fn swap_exact_input_quadruplehop(env:Env,to:Address,pools:Vec<Address>,swap_x_to_y:bool,swap_y_to_z:bool,swap_z_to_a:bool,swap_a_to_b:bool,x_in:i128, b_min_out:i128)->i128{
        
        to.require_auth();

        let first_pool = liquiditypool::Client::new(&env, &pools.get(0).unwrap());
        let second_pool = liquiditypool::Client::new(&env, &pools.get(1).unwrap());
        let third_pool = liquiditypool::Client::new(&env, &pools.get(2).unwrap());
        let fourth_pool = liquiditypool::Client::new(&env, &pools.get(3).unwrap());

        let y_out = if swap_x_to_y {
            first_pool.swap_exact_input(&to, &true, &x_in, &0)
        }else{
            first_pool.swap_exact_input(&to, &false, &x_in, &0)
        };

        let z_out = if swap_y_to_z {
            second_pool.swap_exact_input(&to, &true, &y_out, &0)
        }else{
            second_pool.swap_exact_input(&to, &false, &y_out, &0)
        };

        let a_out = if swap_z_to_a { 
            third_pool.swap_exact_input(&to, &true, &z_out, &0)
        }else{
            third_pool.swap_exact_input(&to, &false, &z_out, &0)
        };

        let b_out = if swap_a_to_b { 
            fourth_pool.swap_exact_input(&to, &true, &a_out, &b_min_out)
        }else{
            fourth_pool.swap_exact_input(&to, &false, &a_out, &b_min_out)
        };

        
        b_out
    }

    
}
