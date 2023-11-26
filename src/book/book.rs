use crate::book::pool::{BasicArena, MemArena};
use crate::book::bump::BumpAllocator;
use std::cmp;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
pub struct OrderChain {
    qty: u64,
    level_id: usize,
    book_id: u16,
    next: usize, // this pointer should be in the bump arena 
    prev: usize
}

impl OrderChain {
    fn new(qty: u64, book_id: u16) -> Self {
        OrderChain {
            qty: qty,
            level_id: usize::MAX,
            book_id: book_id,
            next: usize::MAX,
            prev: usize::MAX
        }
    }
    fn next(self: &OrderChain) -> usize {
        self.next
    }
    fn set_next(self: &mut OrderChain, elem: usize) {
        self.next = elem;
    }
}

pub struct Level {
    pub head: usize, // this pointer should be in the bump arena
    pub tail: usize, // this pointer should be in the bump arena
    price: i8,
    qty: u64
}

impl Level {
    pub fn new(price: i8, qty: u64) -> Self {
        Level {
            head: usize::MAX,
            tail: usize::MAX,
            price: price,
            qty: qty
        }
    }
}

pub struct PriceLevel {
    price: i8,
    level_id: usize
}

pub struct Orderbook {
    // just generally its easier to think of everything as a 'yes' 
    // where no's sell/buy with other yes' to form a contract
    // 
    // Really at the end of the day forming a contract is just a matter of agreeing on a yes price

    // These sorted vecs should be small enough to perma be in cache; sorted in asc order
    // 
    // sorted {price, qty, chain}
    sorted_yes: Vec<PriceLevel>,
    // sorted {price, qty, chain}
    sorted_no: Vec<PriceLevel>,

    // map[oid] Order
    order_arena: Rc<RefCell<BumpAllocator<OrderChain>>>,
    // map[level_id] price, qty, chain
    level_arena: BasicArena<Level>
}

impl Orderbook {
    pub fn with_capacities(order_arena: Rc<RefCell<BumpAllocator<OrderChain>>>, level_capacity: usize) -> Self {
        Orderbook {
            sorted_no: Vec::new(),
            sorted_yes: Vec::new(),
            order_arena: order_arena,
            level_arena: BasicArena::with_capacity(level_capacity),
        }
    }
    fn insert_order(self: &mut Self, order_id: usize, price: i8) {
        let mut order_arena = self.order_arena.borrow_mut();
        let order = order_arena.get(order_id);
        let sorted_levels = if price < 0 { &mut self.sorted_yes } else { &mut self.sorted_no };
        let mut insertion_idx: i32 = sorted_levels.len() as i32 - 1;
        let mut found = false;

        for level in sorted_levels.iter().rev() {
            if level.price == price {
                order.level_id = level.level_id;
                found = true;
                break;
            }
            if level.price < price {
                break;
            }
            insertion_idx -= 1;
        }

        if !found {
            let level_idx = self.level_arena.alloc();
            order.level_id = level_idx;
            self.level_arena.set(level_idx, Level::new(price, 0));
            insertion_idx += 1;
            sorted_levels.insert(insertion_idx as usize, PriceLevel{
                price: price,
                level_id: level_idx
            });
        }

        let level = &mut self.level_arena[order.level_id];
        level.qty += order.qty;
    }
    fn add_to_order_chain(self: &mut Self, order_id: usize) {
        let mut order_arena = self.order_arena.borrow_mut();
        let order = order_arena.get(order_id); 
        let level = &mut self.level_arena[order.level_id];

        if level.head == usize::MAX {
            level.head = order_id;
            level.tail = order_id;
        } else {
            let mut cur_order_idx = level.head;
            let mut next_order_idx = order_arena.get(level.head).next;
            while next_order_idx != usize::MAX {
                cur_order_idx = next_order_idx;
                next_order_idx = order_arena.get(cur_order_idx).next;
            }
            order_arena.get(cur_order_idx).next = order_id;
            order_arena.get(order_id).prev = cur_order_idx;
            level.tail = order_id;
        }
    }
    fn reduce_order(self: &mut Self, order_id: usize, qty: u64) {
        let mut order_arena = self.order_arena.borrow_mut();
        let order = order_arena.get(order_id);
        debug_assert!(order.qty >= qty);
        let level = &mut self.level_arena[order.level_id];
        level.qty -= qty;
        order.qty -= qty;

        if level.qty == 0 {
            let sorted_levels = if level.price < 0 { &mut self.sorted_yes } else { &mut self.sorted_no };
            let idx = sorted_levels.iter().rev().position(|x| x.price == level.price);
            debug_assert!(idx.is_some());
            sorted_levels.remove(idx.unwrap());
            self.level_arena.free(order.level_id);
        } else if order.qty == 0 {
            // we don't have to do this if we lose the level because references to these order will
            // be lost
            let prev = order_arena.get(order_id).prev;
            let next = order_arena.get(order_id).next;
            if prev == usize::MAX {
                level.head = next;
                order_arena.get(next).prev = usize::MAX;
                // if prev is NULL and tail is the same as prev
                // this must be the only order at the level
                // which can't be because the above branch didn't execute
                // thus we only have to modify head
            } else {
                // prev is not null, we don't care if next is null
                // unlink the order
                order_arena.get(prev).next = next;
            }
        }
    }
    pub fn delete(self: &mut Self, order_id: usize) {
        let order_qty = self.order_arena.borrow_mut().get(order_id).qty;
        self.reduce_order(order_id, order_qty);
    }
    pub fn reduce(self: &mut Self, order_id: usize, qty: u64) {
        self.reduce_order(order_id, qty);
    }
    pub fn add(self: &mut Self, qty: u64, book_id: u16, price: i8) -> usize {
        let order_id = self.order_arena.borrow_mut().write(OrderChain::new(qty, book_id));
        self.insert_order(order_id, price);
        self.add_to_order_chain(order_id);
        order_id
    }
    fn best_order(self: &Self, price: i8) -> Option<&Level> {
        // get the best level for a particular price
        // doesn't guarantee a match just checks price sign for getting the order

        let sorted_levels = if price < 0 { &self.sorted_no } else { &self.sorted_yes };
        let price_level = sorted_levels.last();

        match price_level {
            None => None,
            Some(price_level) => {
                Some(&self.level_arena[price_level.level_id])
            }
        }
    }
    pub fn match_order(self: &mut Self, mut qty: u64, book_id: u16, price: i8) -> usize {
        while let Some(level) = self.best_order(price) {
            if level.price < price { // TODO: fix this line and we're gtg i think
                let transaction_qty = cmp::min(qty, level.qty);
                self.reduce_order(level.head, transaction_qty);
                qty -= transaction_qty;

                // TODO: report execution
            }
        }

        if qty > 0 {
            self.add(qty, book_id, price)
        } else {
            usize::MAX
        }
    }
    pub fn print(self: &Self) {
        let mut order_arena = self.order_arena.borrow_mut();
        println!("YES");
        for pl in self.sorted_yes.iter().rev() {
            let level_id = pl.level_id;
            let (qty, head) = {
                let level = &self.level_arena[level_id];
                (level.qty, level.head)
            };
            println!("| $0.{} @ {}", pl.price.abs(), qty);

            let mut cur_idx = head;

            while cur_idx != usize::MAX {
                let order = order_arena.get(cur_idx);
                println!("|---- #{} @ {}", cur_idx, order.qty);
                cur_idx = order.next;
            }
        }

        println!("NO");
        for pl in self.sorted_no.iter().rev() {
            let level_id = pl.level_id;
            let (qty, head) = {
                let level = &self.level_arena[level_id];
                (level.qty, level.head)
            };
            println!("| $0.{} @ {}", pl.price.abs(), qty);

            let mut cur_idx = head;

            while cur_idx != usize::MAX {
                let order = order_arena.get(cur_idx);
                println!("|---- #{} @ {}", cur_idx, order.qty);
                cur_idx = order.next;
            }
        }
    }
}
