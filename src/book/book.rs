use crate::book::bump::BumpAllocator;
use crate::book::pool::{BasicArena, MemArena};
use crate::comm::urcp::*;
use std::cell::RefCell;
use std::cmp;
use std::rc::Rc;

#[derive(Debug)]
pub struct OrderChain {
    qty: u64,
    level_id: usize,
    next: usize, // this pointer should be in the bump arena
    prev: usize,
}

impl OrderChain {
    fn new(qty: u64) -> Self {
        OrderChain {
            qty: qty,
            level_id: usize::MAX,
            next: usize::MAX,
            prev: usize::MAX,
        }
    }
}

pub struct Level {
    pub head: usize, // this pointer should be in the bump arena
    price: i8,
    qty: u64,
}

impl Level {
    pub fn new(price: i8, qty: u64) -> Self {
        Level {
            head: usize::MAX,
            price: price,
            qty: qty,
        }
    }
}

#[derive(Debug)]
pub struct PriceLevel {
    price: i8,
    level_id: usize,
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
    level_arena: BasicArena<Level>,
}

impl Orderbook {
    pub fn clear(&mut self) {
        self.sorted_yes.clear();
        self.sorted_no.clear();
        self.order_arena.borrow_mut().clear();
        self.level_arena.clear();
    }

    pub fn with_capacities(
        order_arena: Rc<RefCell<BumpAllocator<OrderChain>>>,
        level_capacity: usize,
    ) -> Self {
        Orderbook {
            sorted_no: Vec::new(),
            sorted_yes: Vec::new(),
            order_arena: order_arena,
            level_arena: BasicArena::with_capacity(level_capacity),
        }
    }
    fn insert_order(self: &mut Self, order_id: usize, price: i8) -> (i8, i64) {
        let mut order_arena = self.order_arena.borrow_mut();
        let order = order_arena.get(order_id);
        let sorted_levels = if price < 0 {
            &mut self.sorted_yes
        } else {
            &mut self.sorted_no
        };
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
            sorted_levels.insert(
                insertion_idx as usize,
                PriceLevel {
                    price: price,
                    level_id: level_idx,
                },
            );
        }

        let level = &mut self.level_arena[order.level_id];
        level.qty += order.qty;

        (level.price, order.qty as i64)
    }
    fn add_to_order_chain(self: &mut Self, order_id: usize) {
        let mut order_arena = self.order_arena.borrow_mut();
        let order = order_arena.get(order_id);
        let level = &mut self.level_arena[order.level_id];

        if level.head == usize::MAX {
            level.head = order_id;
        } else {
            let mut cur_order_idx = level.head;
            let mut next_order_idx = order_arena.get(level.head).next;
            while next_order_idx != usize::MAX {
                cur_order_idx = next_order_idx;
                next_order_idx = order_arena.get(cur_order_idx).next;
            }
            order_arena.get(cur_order_idx).next = order_id;
            order_arena.get(order_id).prev = cur_order_idx;
        }
    }
    fn reduce_order(self: &mut Self, order_id: usize, mut qty: u64) -> PriceLevelResponse {
        let mut order_arena = self.order_arena.borrow_mut();
        let order = order_arena.get(order_id);
        // debug_assert!(order.qty >= qty);
        // qty = cmp::min(order.qty, qty); // prevents underflow errors in case of user tardation
        let level = &mut self.level_arena[order.level_id];
        level.qty -= qty;
        order.qty -= qty;
        
        let ret = PriceLevelResponse {
            price: level.price,
            delta: -(qty as i64),
        };

        if level.qty == 0 {
            let sorted_levels: &mut Vec<PriceLevel> = if level.price < 0 {
                &mut self.sorted_yes
            } else {
                &mut self.sorted_no
            };
            let idx = sorted_levels
                .iter()
                .rev()
                .position(|x| x.level_id == order.level_id);
            debug_assert!(idx.is_some());
            sorted_levels.remove(sorted_levels.len() - 1 - idx.unwrap());
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
                if next != usize::MAX {
                    order_arena.get(next).prev = prev;
                }
            }
        }

        ret
    }
    pub fn delete(self: &mut Self, order_id: usize) -> PriceLevelResponse {
        let order_qty = self.order_arena.borrow_mut().get(order_id).qty;
        self.reduce_order(order_id, order_qty)
    }
    pub fn reduce(self: &mut Self, order_id: usize, qty: u64) -> PriceLevelResponse {
        self.reduce_order(order_id, qty)
    }
    pub fn add(self: &mut Self, qty: u64, price: i8) -> usize {
        let order_id = self.order_arena.borrow_mut().write(OrderChain::new(qty));
        self.insert_order(order_id, price);
        self.add_to_order_chain(order_id);
        order_id
    }
    fn best_order(self: &Self, price: i8) -> Option<(usize, i8)> {
        // get the best level for a particular price
        // doesn't guarantee a match just checks price sign for getting the order

        let sorted_levels = if price < 0 {
            &self.sorted_no
        } else {
            &self.sorted_yes
        };
        let price_level = sorted_levels.last();

        match price_level {
            None => None,
            Some(price_level) => Some({
                let level = &self.level_arena[price_level.level_id];
                (level.head, level.price)
            }),
        }
    }
    pub fn match_order(self: &mut Self, mut qty: u64, price: i8) -> Vec<OBResponseWrapper> {
        let mut actions: Vec<OBResponseWrapper> = Vec::new();

        while let Some((lh, lp)) = self.best_order(price) {
            let head_qty = self.order_arena.borrow_mut().get(lh).qty;
            if lp.abs() <= price.abs() {
                // TODO: fix this line and we're gtg i think
                let transaction_qty = cmp::min(qty, head_qty);
                let price_delta = self.reduce_order(lh, transaction_qty);
                qty -= transaction_qty;

                actions.push(OBResponseWrapper {
                    resp: OBResponse {
                        execute: ExecuteResponse::new(lh, transaction_qty),
                    },
                    typ: OBRespType::EXECUTE,
                });
                actions.push(OBResponseWrapper {
                    resp: OBResponse { price: price_delta },
                    typ: OBRespType::PRICE,
                });
            } else {
                break;
            }

            if qty == 0 {
                break;
            }
        }

        if qty > 0 {
            let oid = self.add(qty, price);
            actions.push(OBResponseWrapper {
                resp: OBResponse {
                    add: AddResponse::new(oid, qty),
                },
                typ: OBRespType::ADD,
            });
            actions.push(OBResponseWrapper {
                resp: OBResponse {
                    price: PriceLevelResponse::new(price, qty as i64),
                },
                typ: OBRespType::PRICE,
            });
        }

        actions
    }

    pub fn get_level_view(self: &Self) -> [u64; 200] {
        let mut ret: [u64; 200] = [0; 200];
        for pl in &self.sorted_yes {
            let level_id = pl.level_id;
            let (qty, _) = {
                let level = &self.level_arena[level_id];
                (level.qty, level.head)
            };

            ret[level_id] = qty;
        }

        for pl in &self.sorted_no {
            let level_id = pl.level_id;
            let (qty, _) = {
                let level = &self.level_arena[level_id];
                (level.qty, level.head)
            };

            ret[level_id + 100] = qty;
        }

        return ret;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn arena() -> Rc<RefCell<BumpAllocator<OrderChain>>> {
        Rc::new(RefCell::new(BumpAllocator::<OrderChain>::with_capacity(100)))
    }

    fn book() -> Orderbook {
        let arena = arena();
        Orderbook::with_capacities(arena, 200)
    }

    fn test_n_remove(n: usize, idx: usize) {
        let mut book = book();
        let mut order_vec: Vec<usize> = vec![];
        for i in 0..n {
            book.add((i + 1) as u64, 50);
            order_vec.push(i);
        }

        // delete the targeted order
        book.reduce_order(idx, (idx + 1) as u64);
        order_vec.remove(idx);
        book.print();

        // get the best order
        let best_order = book.best_order(-50);
        match order_vec.len() {
            0 => {
                assert!(best_order.is_none());
                assert!(book.level_arena.len() == 0);
                return;
            },
            _ => assert!(best_order.is_some())
        }

        // get the best order id
        let (best_oid, _) = best_order.unwrap();

        // asser that the best order id is the one we put in first (after removal)
        assert!(best_oid == *order_vec.first().unwrap());

        let mut arena = book.order_arena.borrow_mut();
        let mut oid_iter = best_oid;
        let mut oid_prev = usize::MAX;
        for e in order_vec.iter() {
            let order = arena.get(oid_iter);
            assert!(oid_iter == *e);
            assert!(order.qty == (*e + 1) as u64);
            assert!(order.prev == oid_prev, "{}.prev is incorrectly {} not {}", oid_iter, order.prev, oid_prev);

            oid_prev = oid_iter;
            oid_iter = order.next;
        }

        assert!(oid_iter == usize::MAX, "last order in chain should be nil");

        let best_order = arena.get(best_oid);

        let best_level = &book.level_arena[best_order.level_id];

        assert!(best_level.head == *order_vec.first().unwrap());
    }

    #[test]
    fn test_chaining() {
        let mut book = book();
        for i in 1..101 {
            book.add(i, 99);
        }

        let mut arena = book.order_arena.borrow_mut();
        let mut oid = 0;
        // oid's should be 0 -> 99
        for i in 0..100 {
            let order = arena.get(oid);
            assert!(order.qty == i + 1);
            assert!(oid == i as usize);
            oid = order.next;
        }
    }

    #[test]
    fn test_5_remove_start() {
        test_n_remove(5, 0);
    }

    #[test]
    fn test_5_remove_end() {
        test_n_remove(5, 4);
    }

    #[test]
    fn test_5_remove_middle() {
        test_n_remove(5, 2);
    }

    #[test]
    fn test_1_remove() {
        test_n_remove(1, 0);
    }

    #[test]
    fn test_2_remove_first() {
        test_n_remove(2, 0);
    }

    #[test]
    fn test_2_remove_last() {
        test_n_remove(2, 1);
    }
}
