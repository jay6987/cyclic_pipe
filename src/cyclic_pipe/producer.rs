use super::token::Token;
use std::sync::mpsc;

pub struct Producer<T>
where
    T: Clone,
{
    pub(crate) rx_empty: mpsc::Receiver<mpsc::Receiver<T>>,
    pub(crate) tx_full: mpsc::Sender<mpsc::Receiver<T>>,
}

impl<T> Producer<T>
where
    T: Clone,
{
    pub fn get_write_token(&self) -> Result<Token<T>, mpsc::RecvError> {
        match self.rx_empty.recv() {
            Ok(buf_recv) => Ok(match buf_recv.recv() {
                Ok(buf) => {
                    let (tx, rx) = mpsc::channel::<T>();
                    self.tx_full.send(rx).unwrap();
                    Token { buf, sender: tx }
                }
                Err(e) => return Err(e),
            }),
            Err(e) => Err(e),
        }
    }
}
