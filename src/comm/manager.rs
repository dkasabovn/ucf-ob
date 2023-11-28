use crate::book::book::{Orderbook};
use crate::book::bump::BumpAllocator;
use std::cell::RefCell;
use std::rc::Rc;
use std::ops;

pub struct Manager {
    books: Vec<Orderbook>
}

impl Manager {
    pub fn new(order_capacity: usize, level_capacity: usize, book_size: u16) -> Self {
        let arena = Rc::new(RefCell::new(BumpAllocator::with_capacity(order_capacity)));
        let mut books: Vec<Orderbook> = Vec::with_capacity(book_size.into()); 

        for _ in 0..book_size {
            books.push(Orderbook::with_capacities(Rc::clone(&arena), level_capacity));
        }

        Manager {
            books: books, 
        }
    }

}

impl ops::Index<usize> for Manager {
    type Output = Orderbook;
    fn index<'a>(&'a self, i: usize) -> &'a Orderbook {
        &self.books[i]
    }
}

impl ops::IndexMut<usize> for Manager {
    fn index_mut<'a>(&'a mut self, i: usize) -> &'a mut Orderbook {
        &mut self.books[i]
    }
}
