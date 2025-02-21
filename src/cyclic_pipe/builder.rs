use crate::cyclic_pipe::{consumer::Consumer, producer::Producer};
use std::sync::mpsc;

/// A builder for creating a cyclic pipe with a specified size and initial template.
///
/// The `Builder` struct allows you to configure a cyclic pipe then create the pipe and return
/// producer and consumer ends. The cyclic pipe can be used to send and receive data in a circular
/// buffer-like fashion.
///
/// # Type Parameters
///
/// * `T` - The type of the elements in the cyclic pipe. It must implement the `Clone` trait.
///
/// # Fields
///
/// * `size` - The size of the cyclic pipe. This determines the number of elements that can be
///   stored in the pipe at any given time.
/// * `init_template` - An optional initial template value that will be used to initialize the
///   elements in the cyclic pipe.
///
/// # Examples
///
/// ```
/// use cyclic_pipe::Builder;
///
/// let builder = Builder::new()
///     .with_size(2)
///     .with_init_template(vec![0; 1000]);
/// let (producer, consumer) = builder.build();
/// ```
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
    /// Creates a new `Builder` with default values.
    /// The default size is 1 and the initial template is `None`.
    pub fn new() -> Self {
        Builder {
            size: 1,
            init_template: None,
        }
    }

    /// Sets the size of the cyclic pipe.
    /// The minimum size is 1.
    /// When the size is set to 1, the cyclic pipe will behave like a mutexed buffer,
    /// where the producer and consumer must take turns to access the buffer.
    pub fn with_size(mut self, size: usize) -> Self {
        self.size = size;
        self
    }

    /// Sets the initial template value that will be clone for elements in the cyclic pipe.
    pub fn with_init_template(mut self, init_template: T) -> Self {
        self.init_template = Some(init_template);
        self
    }

    /// Create the cyclic pipe and return producer and consumer ends.
    pub fn build(self) -> (Producer<T>, Consumer<T>) {
        let (tx_empty, rx_empty) = mpsc::channel::<std::sync::mpsc::Receiver<T>>();
        let (tx_full, rx_full) = mpsc::channel::<std::sync::mpsc::Receiver<T>>();
        match self.init_template{
            None => panic!("init_template is not set"),
            Some(template) => {
          for _i in 0..(self.size - 1) {
            let (tx, rx) = mpsc::channel::<T>();
            tx_empty.send(rx).unwrap();
            tx.send(template.clone()).unwrap();
        }
        {
            let (tx, rx) = mpsc::channel::<T>();
            tx_empty.send(rx).unwrap();
            tx.send(template).unwrap();
        }
        let producer = Producer::new(rx_empty, tx_full);
        let consumer = Consumer::new(rx_full, tx_empty);
        (producer, consumer)              
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_build_with_init_template() {
        let builder = Builder::new()
            .with_size(2)
            .with_init_template(vec![0; 1000]);
        let (_p, _c) = builder.build();
    }

    #[test]
    #[should_panic(expected = "init_template is not set")]
    fn builder_build_without_init_template() {
        let builder = Builder::<Vec<i32>>::new().with_size(2);
        let _ = builder.build();
    }

    #[test]
    fn builder_build_size() {
        let pipe_size = 10;
        let builder = Builder::new()
            .with_size(10)
            .with_init_template(vec![0; 1000]);
        let (p, c) = builder.build();
        let mut concurent_write_tokens = Vec::with_capacity(pipe_size);
        let mut concurent_read_tokens = Vec::with_capacity(pipe_size);
        for _i in 0..pipe_size{
            concurent_write_tokens.push(p.get_write_token().unwrap());
        }
        for wt in concurent_write_tokens{
            wt.done();
            concurent_read_tokens.push(c.get_read_token().unwrap());
        }
        for rt in concurent_read_tokens{
            rt.done();
        }
    }

}