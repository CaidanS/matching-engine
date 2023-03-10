use core::fmt::Debug;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use uuid::Uuid;
use crate::accounts::TraderAccount;
use crate::orderbook::OrderBook;
// use crate::orderbook::TraderId;

macro_rules! generate_enum {
    ([$($name:ident),*]) => {
        #[derive(Debug, Copy, Clone, Deserialize, Serialize)]
        pub enum TickerSymbol {
            $($name, )*
        }
    };
}

macro_rules! generate_accounts_enum {
    ([$($name:ident),*]) => {
        #[derive(Debug, Copy, Clone, Deserialize, Serialize)]
        pub enum TraderId {
            $($name, )*
        }
    };
}

macro_rules! generate_account_balances_struct {
    ([$($name:ident),*]) => {
        #[derive(Debug)]
        pub struct AssetBalances {
            $($name: Mutex<usize>, )*
        }    

        impl AssetBalances {
            pub fn index_ref (&self, symbol:&TickerSymbol) -> &Mutex<usize>{
                match symbol {
                    $(TickerSymbol::$name => {&self.$name}, )*
                }
            }     
            
            pub fn new() -> Self {
                Self { 
                    $($name: Mutex::new(0), )*
                 }
            }
               
        }
    };
}

macro_rules! generate_global_state {
    ([$($name:ident),*], [$($account_id:ident),*]) => {
        #[derive(Debug)]
        pub struct GlobalOrderBookState {
            $(pub $name: Mutex<crate::orderbook::OrderBook>, )*
        }
        
        impl GlobalOrderBookState {
            pub fn index_ref (&self, symbol:&TickerSymbol) -> &Mutex<crate::orderbook::OrderBook>{
                match symbol {
                    $(TickerSymbol::$name => {&self.$name}, )*
                }
            }
        }
        
        pub struct GlobalAccountState {
            $(pub $account_id: Mutex<crate::accounts::TraderAccount>, )*
        }

        impl GlobalAccountState {
            pub fn index_ref (&self, account_id:crate::macro_calls::TraderId,) -> &Mutex<crate::accounts::TraderAccount>{
                match account_id {
                    $(TraderId::$account_id => {&self.$account_id}, )*
                }
            }                
        }
    };
}
        
generate_enum!([
        AAPL,
        JNJ
    ]);
generate_account_balances_struct!([
        AAPL,
        JNJ
    ]);
generate_global_state!([
        AAPL,
        JNJ
    ], [
        Columbia_A,
        Columbia_B
    ]);
generate_accounts_enum!([
        Columbia_A,
        Columbia_B
    ]);