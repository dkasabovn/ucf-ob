use std::io;
use rusqlite::{Connection,Result,params};
use crate::comm::domain::*;

const USER_BALANCE_DEFAULT: i32 = 10000;

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
pub struct InnerRepo {
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

impl InnerRepo {
    pub fn new() -> io::Result<Self> {
        let con: Connection = match Connection::open_in_memory() {
            Ok(con) => io::Result::Ok(con),
            Err(e) => io::Result::Err(io::Error::new(io::ErrorKind::Other, e)),
        }?;

        match con.execute_batch(
            "BEGIN;
             CREATE TABLE IF NOT EXISTS users (
                id SERIAL PRIMARY KEY,
                sub TEXT NOT NULL UNIQUE,
                balance INT NOT NULL
             );
             CREATE TABLE IF NOT EXISTS user_orders (
                id SERIAL PRIMARY KEY,
                book_id INT NOT NULL,
                price INT NOT NULL,
                qty INT NOT NULL,
                user_fk INT NOT NULL REFERENCES users(id)
             );
             CREATE TABLE IF NOT EXISTS contracts (
                id SERIAL PRIMARY KEY,
                user_no_fk INT NOT NULL REFERENCES users(id),
                user_yes_fk INT NOT NULL REFERENCES users(id),
                qty INT NOT NULL
             );
             COMMIT;",
        ) {
            Ok(_) => io::Result::Ok(()),
            Err(e) => io::Result::Err(io::Error::new(io::ErrorKind::Other, e)),
        }?;

        Ok(Self{
            con
        })
    }
    // create user with default balance
    // INSERT INTO users (sub, balance) VALUES (?1, ?2);
    pub fn create_user(&mut self, sub: String) -> Result<()> {
        self.con.execute(
            "INSERT INTO users (sub, balance) VALUES (?1, ?2)",
            (&sub, &USER_BALANCE_DEFAULT),
        )?;
        Ok(())
    }
    // get the user object by sub
    // SELECT * FROM users WHERE sub = ?1;
    pub fn get_user(&mut self, sub: String) -> Result<User> {
        self.con.query_row_and_then(
            "SELECT * FROM users WHERE sub = ?1",
            params![sub],
            |row| {
                Ok(User{
                    id: row.get(0)?,
                    sub: row.get(1)?,
                    balance: row.get(2)?,
                })
            }
        )
    }
    // subtract user balance by #
    // UPDATE users SET balance = balance - ?2 WHERE id = ?1;
    pub fn modify_user_balance(&mut self, uid: i32, amt: i32) -> Result<()> {
        self.con.execute(
            "UPDATE users SET balance = balance + ?2 WHERE id = ?1",
            (&uid, &amt),
        )?;
        Ok(())
    }
    // add order to order table referencing user id (should have) (Add order msg)
    // INSERT INTO user_orders (id,book_id, price, qty, user_fk) VALUES (?1, ?2, ?3, ?4, ?5); 
    // -- ?1 is just the oid
    pub fn add_order_to_user(&mut self, oid: usize, book_id: u16, price: i8, qty: u64, uid: i32) -> Result<()> {
        self.con.execute(
            "INSERT INTO user_orders (id, book_id, price, qty, user_fk) VALUES (?1, ?2, ?3, ?4, ?5)",
            (&(oid as i32), &(book_id as i32), &(price as i32), &(qty as i32), &uid),
        )?;
        Ok(())
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
    pub fn create_contract(&mut self, user_uid: i32, other_oid: usize, qty: u64) -> Result<()> {
        let other_order: Result<UserOrder> = self.con.query_row_and_then(
            "SELECT * FROM user_orders WHERE id = ?1",
            params![&(other_oid as i32)],
            |row| {
                Ok(UserOrder{
                    id: row.get(0)?,
                    book_id: row.get(1)?,
                    price: row.get(2)?,
                    qty: row.get(3)?,
                    user_fk: row.get(4)?,
                })
            },
        );
        let other_order = other_order?;

        let contract_yes = if other_order.price < 0 { other_order.user_fk } else { user_uid };
        let contract_no = if other_order.price < 0 { user_uid } else { other_order.user_fk };
        
        self.con.execute(
            "INSERT INTO contracts (user_no_fk, user_yes_fk, qty) VALUES (?1, ?2, ?3)",
            (&contract_no, &contract_yes, &qty),
        )?;

        self.con.execute(
            "UPDATE user_orders SET qty = qty - ?2 WHERE id = ?1",
            (&(other_oid as i32), &qty),
        )?;

        Ok(())
    }
    // get a list of contracts and do payouts
    // SELECT * FROM contracts;
    // -- use for paying out when outcome is known; make api request for setting outcome
    pub fn get_contracts(&mut self) -> Result<Vec<Contract>> {
        let mut stmt = self.con.prepare("SELECT user_yes_fk, user_no_fk, qty FROM contracts")?;
        let rows = stmt.query_map([], |row| Ok(Contract{
            yes_holder: row.get(0)?,
            no_holder: row.get(1)?,
            qty: row.get(2)?,
        }))?;

        let ret: Vec<Contract> = rows.into_iter().map(|x| x.unwrap_or(Contract{
            yes_holder: -1,
            no_holder: -1,
            qty: -1
        })).collect();
        
        Ok(ret)
    }
    // drop all orders; done every 5 mins with contract payout
    // DELETE FROM user_orders;
    pub fn drop_orders(&mut self) -> Result<()> {
        self.con.execute_batch(
            "BEGIN;
             DELETE FROM user_orders;
             DELETE FROM contracts;
             COMMIT;",
        )?;
        Ok(())
    }

    pub fn delete_order(&mut self, oid: usize) -> Result<()> {
        self.con.execute(
            "DELETE FROM user_orders WHERE id = ?1",
            params![&(oid as i32)],
        )?;
        Ok(())
    }
}
