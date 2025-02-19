use super::token::Token;
use std::sync::mpsc;

pub struct Producer<T>
where
    T: Clone,
{
    inner: ProducerInner<T>,
}

struct ProducerInner<T>
where
    T: Clone,
{
    rx_empty: mpsc::Receiver<mpsc::Receiver<T>>,
    tx_full: mpsc::Sender<mpsc::Receiver<T>>,
}

impl<T> Producer<T>
where
    T: Clone,
{
    // A constructor for internal use, for example from Builder.
    pub(crate) fn new(
        rx_empty: mpsc::Receiver<mpsc::Receiver<T>>,
        tx_full: mpsc::Sender<mpsc::Receiver<T>>,
    ) -> Self {
        Producer {
            inner: ProducerInner { rx_empty, tx_full },
        }
    }

    pub fn get_write_token(&self) -> Result<Token<T>, mpsc::RecvError> {
        match self.inner.rx_empty.recv() {
            Ok(buf_recv) => Ok(match buf_recv.recv() {
                Ok(buf) => {
                    let (tx, rx) = mpsc::channel::<T>();
                    match self.inner.tx_full.send(rx) {
                        Ok(_) => Token::new(buf, tx),
                        Err(_) => return Err(mpsc::RecvError),
                    }
                }
                Err(e) => return Err(e),
            }),
            Err(e) => Err(e),
        }
    }
}
