use std::cmp;
use uuid::Uuid;
type Price = usize;
type TraderID = u8;
// for loading csv test files
use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::fs::File;
use std::process::{self, Output};
use serde::Deserialize;

type Record = (String, String, Option<u64>, f64, f64);


#[derive(Debug, Clone, Copy, Deserialize)]
enum OrderType {
    Buy,
    Sell,
}
#[derive(Debug, Copy, Clone, Deserialize)]
enum TickerSymbol {
    AAPL,
}

#[derive(Debug)]
struct OrderBook {
    /// Struct representing a double sided order book for a single product.
    symbol: TickerSymbol,
    // buy side in increasing price order
    buy_side_limit_levels: Vec<LimitLevel>,
    // sell side in increasing price order
    sell_side_limit_levels: Vec<LimitLevel>,
    current_high_buy_price: Price,
    current_low_sell_price: Price,
}
#[derive(Debug)]
struct LimitLevel {
    /// Struct representing one price level in the orderbook, containing a vector of Orders at this price
    price: Price,
    orders: Vec<Order>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct OrderRequest {    
    /// Struct representing an incoming request which has not yet been added to the orderbook
    amount: i32,
    price: Price,
    order_type: OrderType,
    trader_id: TraderID,
    symbol: TickerSymbol,
}

#[derive(Debug)]
struct Order {
    /// Struct representing an existing order in the order book
    order_id: Uuid,
    trader_id: TraderID,
    symbol: TickerSymbol,
    amount: i32,
    price: Price,
    order_type: OrderType,
}
#[derive(Debug, Copy, Clone)]
struct Trader {
    id: TraderID,
}

#[derive(Debug)]
struct Fill {
    /// Struct representing an order fill event, used to update credit limits, communicate orderbook status etc.
    sell_trader_id: TraderID,
    buy_trader_id: TraderID,
    amount: i32,
    price: Price,
    symbol: TickerSymbol,
    trade_time: u8,
}
impl OrderBook {
    fn add_order_to_book(&mut self, new_order_request: OrderRequest) -> Uuid {
        let order_id = Uuid::new_v4();
        let new_order = Order {
            order_id: order_id,
            trader_id: new_order_request.trader_id,
            symbol: new_order_request.symbol,
            amount: new_order_request.amount,
            price: new_order_request.price,
            order_type: new_order_request.order_type,
        };
        match new_order.order_type {
            OrderType::Buy => {
                if self.current_high_buy_price < new_order.price {
                    self.current_high_buy_price = new_order.price;
                };
                self.buy_side_limit_levels[new_order.price]
                    .orders
                    .push(new_order);
            }
            OrderType::Sell => {
                if self.current_low_sell_price > new_order.price {
                    self.current_low_sell_price = new_order.price;
                };
                self.sell_side_limit_levels[new_order.price]
                    .orders
                    .push(new_order);
            }
        }
        order_id
    }



    fn remove_order_by_uuid(
        &mut self,
        order_id: Uuid,
        price: usize,
        side: OrderType,
    ) -> Option<Order> {
        match side {
            OrderType::Buy => {
                let mut index = 0;
                while index < self.buy_side_limit_levels[price].orders.len() {
                    if self.buy_side_limit_levels[price].orders[index].order_id == order_id {
                        return Some(self.buy_side_limit_levels[price].orders.remove(index));
                    }
                    index += 1;
                }
            }
            OrderType::Sell => {
                let mut index = 0;
                while index < self.sell_side_limit_levels[price].orders.len() {
                    if self.sell_side_limit_levels[price].orders[index].order_id == order_id {
                        return Some(self.sell_side_limit_levels[price].orders.remove(index));
                    }
                    index += 1;
                }
            }
        }
        None
    }
    
    fn handle_incoming_order_request (&mut self, new_order_request: OrderRequest) -> Option<Uuid> {
        match new_order_request.order_type {
            OrderType::Buy => {
                self.handle_incoming_buy(new_order_request)
            }
            OrderType::Sell => {
                self.handle_incoming_sell(new_order_request)
            }
        }
    }
    fn handle_incoming_sell(&mut self, mut sell_order: OrderRequest) -> Option<Uuid> {
        if sell_order.price <= self.current_high_buy_price {
            let mut current_price_level = self.current_high_buy_price;
            while (sell_order.amount > 0) & (current_price_level > sell_order.price) {
                let mut order_index = 0;
                while order_index < self.buy_side_limit_levels[current_price_level].orders.len() {
                    
                    // let order_considering =
                    //     &self.buy_side_limit_levels[current_price_level].orders[order_index];

                    let trade_price =    self.buy_side_limit_levels[current_price_level].orders[order_index].price;
                    let buy_trader_id =    self.buy_side_limit_levels[current_price_level].orders[order_index].trader_id; 
                    
                    let amount_to_be_traded = cmp::min(sell_order.amount, self.buy_side_limit_levels[current_price_level].orders[order_index].amount);
                   
                    self.handle_fill_event(Fill{
                        sell_trader_id: sell_order.trader_id, 
                        buy_trader_id: buy_trader_id,
                        symbol: self.symbol,
                        amount: amount_to_be_traded,
                        price: trade_price,
                        trade_time: 1,
                    });
                    // TODO: create "sell" function that can handle calls to allocate credit etc.
                    sell_order.amount -= amount_to_be_traded;
                    self.buy_side_limit_levels[current_price_level].orders[order_index].amount -= amount_to_be_traded;
                    if self.buy_side_limit_levels[current_price_level].orders[order_index].amount == 0 {
                        self.sell_side_limit_levels[current_price_level]
                            .orders
                            .remove(order_index);
                    }
                    order_index += 1;
                }
                current_price_level += 1;
            }
            while current_price_level < self.buy_side_limit_levels.len() {
                if self.buy_side_limit_levels[current_price_level]
                    .orders
                    .len()
                    > 0
                {
                    self.current_high_buy_price = current_price_level;
                    break;
                }
                current_price_level += 1;
            }
            self.current_high_buy_price = current_price_level;
        }

        if sell_order.amount > 0 {
            return Some(self.add_order_to_book(sell_order));
        } else {
            return None;
        }
    }
    fn handle_incoming_buy(&mut self, mut buy_order: OrderRequest) -> Option<Uuid> {
        if buy_order.price >= self.current_low_sell_price {
            let mut current_price_level = self.current_low_sell_price;
            while (buy_order.amount > 0) & (current_price_level < buy_order.price) {
                let mut order_index = 0;
                while order_index
                    < self.sell_side_limit_levels[current_price_level]
                        .orders
                        .len()
                {
                    let trade_price = self.sell_side_limit_levels[current_price_level].orders[order_index].price;
                    let buy_trader_id = self.sell_side_limit_levels[current_price_level].orders[order_index].trader_id; 
                    
                    let amount_to_be_traded = cmp::min(buy_order.amount, self.sell_side_limit_levels[current_price_level].orders[order_index].amount);
                   
                    self.handle_fill_event(Fill{
                        buy_trader_id: buy_order.trader_id, 
                        sell_trader_id: buy_trader_id,
                        symbol: self.symbol,
                        amount: amount_to_be_traded,
                        price: trade_price,
                        trade_time: 1,
                    });
                    
                    // TODO: create "sell" function that can handle calls to allocate credit etc.
                    buy_order.amount -= amount_to_be_traded;
                    self.sell_side_limit_levels[current_price_level].orders[order_index].amount -= amount_to_be_traded;
                    if self.sell_side_limit_levels[current_price_level].orders[order_index].amount == 0 {
                        self.sell_side_limit_levels[current_price_level]
                            .orders
                            .remove(order_index);
                    }
                    order_index += 1;
                }
                current_price_level += 1;
            }
            // in the event that a price level has been completely bought, update lowest sell price
            while current_price_level < self.sell_side_limit_levels.len() {
                if self.sell_side_limit_levels[current_price_level]
                    .orders
                    .len()
                    > 0
                {
                    self.current_low_sell_price = current_price_level;
                    break;
                }
                current_price_level += 1;
            }
            self.current_low_sell_price = current_price_level;
        }
        if buy_order.amount > 0 {
            return Some(self.add_order_to_book(buy_order));
        } else {
            return None;
        }
    }

    fn handle_fill_event(&self, fill_event: Fill){
        println!(
            "{:?} sells to {:?}: {:?} lots of {:?} @ ${:?}",
            fill_event.sell_trader_id, fill_event.buy_trader_id, fill_event.amount, fill_event.symbol, fill_event.price
        );
    }

    fn print_book_state(&self) {
        for price_level_index in 0 .. self.buy_side_limit_levels.len() {
            let mut outstanding_sell_orders: i32 = 0;
            let mut outstanding_buy_orders: i32 = 0;
            for order in self.sell_side_limit_levels[price_level_index].orders.iter(){
                outstanding_sell_orders += order.amount;
            };
            for order in self.buy_side_limit_levels[price_level_index].orders.iter(){
                outstanding_buy_orders += order.amount;
            };
            let mut string_out = String::from("");
            for _ in 0..outstanding_sell_orders {
                string_out = string_out + "S"
            };
            for _ in 0..outstanding_buy_orders {
                string_out = string_out + "B"
            };
            println!("${}: {}", self.buy_side_limit_levels[price_level_index].price, string_out);
            // book_ouput[price_level_index] = outstanding_orders;
        };
    }

    fn load_csv_test_data(&mut self, file_path: OsString) -> Result<(), Box<dyn Error>> {
        println!("Loading order data from {:?}", file_path);
        let file = File::open(file_path)?;
        let mut rdr = csv::Reader::from_reader(file);
        for result in rdr.deserialize() {
            let new_order_request: OrderRequest = result?;
            self.handle_incoming_order_request(new_order_request);
        }
        Ok(())
    }
}

fn quickstart_order_book (symbol: TickerSymbol, min_price: Price, max_price:Price) -> OrderBook {
    OrderBook {
        symbol: TickerSymbol::from(symbol),
        buy_side_limit_levels: (min_price..max_price)
            .map(|x| LimitLevel {
                price: x,
                orders: Vec::new(),
            })
            .collect(),
        sell_side_limit_levels:(min_price..max_price)
            .map(|x| LimitLevel {
                price: x,
                orders: Vec::new(),
            })
            .collect(),
        current_high_buy_price: min_price,
        current_low_sell_price: max_price,
    }
}

fn main() {
    let mut order_book = quickstart_order_book(TickerSymbol::AAPL, 0, 11);
    if let Err(err) = order_book.load_csv_test_data("test_orders.csv".into()) {
        println!("{}", err);
        process::exit(1);
    }
    order_book.print_book_state();
    // println!("{:#?}", order_book);
}
