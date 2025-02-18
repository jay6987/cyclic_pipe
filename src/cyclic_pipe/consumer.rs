use super::token::Token;
use std::sync::mpsc;

pub struct Consumer<T>
where
    T: Clone,
{
    pub(crate) rx_full: mpsc::Receiver<mpsc::Receiver<T>>,
    pub(crate) tx_empty: mpsc::Sender<mpsc::Receiver<T>>,
}

impl<T> Consumer<T>
where
    T: Clone,
{
    pub fn get_read_token(&self) -> Result<Token<T>, mpsc::RecvError> {
        match self.rx_full.recv() {
            Ok(buf_recv) => Ok(match buf_recv.recv() {
                Ok(buf) => {
                    let (tx, rx) = std::sync::mpsc::channel::<T>();
                    self.tx_empty.send(rx).unwrap();
                    Token { buf, sender: tx }
                }
                Err(e) => return Err(e),
            }),
            Err(e) => Err(e),
        }
    }
}
