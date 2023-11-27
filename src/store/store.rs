use rusqlite::{Connection, Result};

// TODO(nw)
//
// Mutex this alongside comm/client::Inner so that 
// orderbook ops + sql ops are always done under one mutex
// this ensures complete synchronous order of events
//
// You could consider doing things like getting users to be more granular 
// by adding a separate mutex to repo and comm/client::Inner
// but that's up to you to prevent deadlock
//
// Combining this with the comm/client::Inner into an api should be straightforward
// Was too sleepy to do implementation tonight
pub struct Repo {
    con: Connection
}

// -- users table
// CREATE TABLE IF NOT EXISTS users (
//  id SERIAL PRIMARY KEY,
//  sub TEXT NOT NULL,
//  balance INT NOT NULL
// );

// -- orders table
// CREATE TABLE IF NOT EXISTS user_orders (
//  id SERIAL PRIMARY KEY, -- corresponds to oid in order book
//  book_id INT NOT NULL, -- corresponds to the book (hard code for now)
//  price INT NOT NULL,
//  qty INT NOT NULL,
//  user_fk INT NOT NULL REFERENCES users(id), -- fk to user
// );

// -- contracts table
// CREATE TABLE IF NOT EXISTS contracts (
//  id SERIAL PRIMARY KEY,
//  user_no_fk INT NOT NULL REFERENCES users(id),
//  user_yes_fk INT NOT NULL REFERENCES users(id),
//  qty INT NOT NULL
// );

impl Repo {
    pub fn new() -> Result<Self> {
        let con = Connection::open_in_memory()?;
        Ok(Self{
            con
        })
    }
    // create user with default balance
    // INSERT INTO users (sub, balance) VALUES (?1, ?2);
    pub fn create_user() {
    }
    // get the user object by sub
    // SELECT * FROM users WHERE sub = ?1;
    pub fn get_user() {
    }
    // subtract user balance by #
    // UPDATE users SET balance = balance - ?2 WHERE id = ?1;
    pub fn subtract_user_balance() {
    }
    // add user balance by #
    // UPDATE users SET balance = balance + ?2 WHERE id = ?1;
    pub fn add_user_balance() {
    }
    // add order to order table referencing user id (should have) (Add order msg)
    // INSERT INTO user_orders (id,book_id, price, qty, user_fk) VALUES (?1, ?2, ?3, ?4, ?5); 
    // -- ?1 is just the oid
    pub fn add_order_to_user() {
    }
    // create contract from user id and other order id [ which references the other user ]
    // also delete other order
    //
    // We don't have to touch the user which received executing order; all oid and user fks are
    // from the resting order except in query 2.) where we have to put both users
    //
    // 1.) SELECT * FROM orders WHERE id = ?1; -- ?1 is oid
    // 2.) INSERT INTO contracts (user_no_fk, user_yes_fk, qty) VALUES (?1, ?2, ?3); -- qty given
    //   by OBResponse.execute yes and no you determine
    // 3.) UPDATE orders SET qty = qty - ?2 WHERE id = ?1; -- ?1 is oid
    pub fn create_contract() {
    }
    // get a list of contracts and do payouts
    // SELECT * FROM contracts;
    // -- use for paying out when outcome is known; make api request for setting outcome
    pub fn get_contracts() {
    }
    // drop all orders; done every 5 mins with contract payout
    // DELETE FROM user_orders;
    pub fn drop_orders() {
    }
}
