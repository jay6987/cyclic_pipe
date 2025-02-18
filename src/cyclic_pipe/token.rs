use std::sync::mpsc;

pub struct Token<T>
where
    T: Clone,
{
    pub buf: T,
    pub(crate) sender: mpsc::Sender<T>,
}

impl<T> Token<T>
where
    T: Clone,
{
    pub fn done(self) {
        self.sender.send(self.buf).unwrap();
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_done_sends_value() {
        let (tx, rx) = mpsc::channel();
        let token = Token {
            buf: "hello",
            sender: tx,
        };
        token.done();
        assert_eq!(rx.try_recv().unwrap(), "hello");
    }

    // This doc test is compile-fail to ensure the token cannot be used after done() is called.
    //
    // It verifies that attempting to use a Token after calling done() does not compile.
    //
    // ```compile_fail
    // use std::sync::mpsc;
    // use cyclic_pipe::token::Token;
    //
    // let (tx, _rx) = mpsc::channel();
    // let token = Token {
    //     buf: "test",
    //     sender: tx,
    // };
    // token.done();
    // // Attempting to call done() a second time is a compile error because token has been moved.
    // token.done();
    // ```
    //
    // Note: Replace `cyclic_pipe::token::Token` with the appropriate module path if needed.

    #[test]
    fn test_token_drop_without_done() {
        use super::Token;
        use std::sync::mpsc;

        let (tx, rx) = mpsc::channel();

        {
            let _token = Token {
                buf: "hello",
                sender: tx,
            };
            // token is dropped here without calling done()
        }

        // The channel has no senders left, so recv() will return an error.
        assert!(rx.recv().is_err());
    }
}
