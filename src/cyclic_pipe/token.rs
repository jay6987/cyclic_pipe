use std::sync::mpsc::{self, SendError};

pub struct Token<T>
where
    T: Clone,
{
    pub buf: T,
    inner: TokenInner<T>,
}

struct TokenInner<T>
where
    T: Clone,
{
    sender: mpsc::Sender<T>,
}

impl<T> Token<T>
where
    T: Clone,
{
    pub(crate) fn new(buf: T, sender: mpsc::Sender<T>) -> Self {
        Token {
            buf,
            inner: TokenInner { sender },
        }
    }

    pub fn done(self) {
        match self.inner.sender.send(self.buf) {
            Ok(_) => (),
            Err(SendError(_)) => (),
            // FIXME: should retrun error for write token
            //        while ignore error for read token
        };
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn done_sends_value() {
        let (tx, rx) = mpsc::channel();
        let token = Token::new("hello", tx);
        assert!(rx.try_recv().is_err(), "token is not done yet");
        token.done();
        assert_eq!(rx.recv().unwrap(), "hello");
    }

    #[test]
    fn token_drop_without_done() {
        use super::Token;
        use std::sync::mpsc;

        let (tx, rx) = mpsc::channel();

        {
            let _token = Token::new("hello", tx);
            // token is dropped here without calling done()
        }

        assert!(
            rx.recv().is_err(),
            "The channel has no senders left, so recv() will return an error."
        );
    }

    #[test]
    fn token_cannot_be_used_after_done() {
        let (tx, rx) = mpsc::channel();
        let token = Token::new("hello", tx);
        token.done();
        // Attempting to use token after done() should result in a compile-time error
        // println!("{}", token.buf()); // Uncommenting this line should cause a compile-time error
        assert_eq!(rx.recv().unwrap(), "hello");

        assert!(
            rx.recv().is_err(),
            "The channel has no senders left, so recv() directly return an error."
        );
    }
}
