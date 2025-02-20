use super::token::Token;
use std::sync::mpsc;

/// A consumer end of a cyclic pipe.
pub struct Consumer<T>
where
    T: Clone,
{
    inner: ConsumerInner<T>,
}

struct ConsumerInner<T>
where
    T: Clone,
{
    rx_full: mpsc::Receiver<mpsc::Receiver<T>>,
    tx_empty: mpsc::Sender<mpsc::Receiver<T>>,
}

impl<T> Consumer<T>
where
    T: Clone,
{
    // Constructor for internal use (e.g. from Builder)
    pub(crate) fn new(
        rx_full: mpsc::Receiver<mpsc::Receiver<T>>,
        tx_empty: mpsc::Sender<mpsc::Receiver<T>>,
    ) -> Self {
        Consumer {
            inner: ConsumerInner { rx_full, tx_empty },
        }
    }

    /// Wait for a read token to access the buffer.
    /// Will be blocked until a buffer is written.
    pub fn get_read_token(&self) -> Result<Token<T>, mpsc::RecvError> {
        match self.inner.rx_full.recv() {
            Ok(buf_recv) => Ok(match buf_recv.recv() {
                Ok(buf) => {
                    let (tx, rx) = mpsc::channel::<T>();
                    // consumer does not necessarily need to send back the buffer
                    // so ignore the error
                    let _ = self.inner.tx_empty.send(rx);
                    Token::new(buf, tx)
                }
                Err(e) => return Err(e),
            }),
            Err(e) => Err(e),
        }
    }
}
