use crate::cyclic_pipe::{consumer::Consumer, producer::Producer};
use std::sync::mpsc;

pub struct Builder<T>
where
    T: Clone,
{
    size: usize,
    init_template: Option<T>,
}

impl<T> Builder<T>
where
    T: Clone,
{
    pub fn new() -> Self {
        Builder {
            size: 1,
            init_template: None,
        }
    }

    pub fn with_size(mut self, size: usize) -> Self {
        self.size = size;
        self
    }

    pub fn with_init_template(mut self, init_template: T) -> Self {
        self.init_template = Some(init_template);
        self
    }

    pub fn build(self) -> (Producer<T>, Consumer<T>) {
        let (tx_empty, rx_empty) = mpsc::channel::<std::sync::mpsc::Receiver<T>>();
        let (tx_full, rx_full) = mpsc::channel::<std::sync::mpsc::Receiver<T>>();
        for _i in 0..(self.size - 1) {
            let (tx, rx) = mpsc::channel::<T>();
            tx_empty.send(rx).unwrap();
            tx.send(self.init_template.clone().unwrap()).unwrap();
        }
        {
            let (tx, rx) = mpsc::channel::<T>();
            tx_empty.send(rx).unwrap();
            tx.send(self.init_template.unwrap()).unwrap();
        }
        let producer = Producer::new(rx_empty, tx_full);
        let consumer = Consumer::new(rx_full, tx_empty);
        (producer, consumer)
    }
}
