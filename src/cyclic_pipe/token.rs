use std::sync::mpsc;

/// A token to access the buffer.
pub struct Token<T>
where
    T: Clone,
{
    /// The buffer from the cyclic pipe to be accessed.
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

    /// Return the buffer to the cyclic pipe.
    /// This method must be called after the buffer is finished being accessed.
    /// If this method is not called before the token is dropped, the other end
    /// of the cyclic pipe will get an error when trying to get a token.
    ///
    /// The reason why this method is designed to be called explicitly in stead of
    /// automatically when the token is dropped is to let the consumer know whether
    /// the buffer is really done or not (i.e. the producer just crushes and the buffer
    /// is not really done yet).
    /// 
    /// # Compile Fail
    ///
    /// a Token cannot be used after it is done.
    /// 
    /// ```compile_fail
    /// use cyclic_pipe::Builder;
    /// use std::sync::mpsc;
    /// 
    /// let (p, _c) = Builder::new()
    ///     .with_init_template("hello".to_string())
    ///     .with_size(1)
    ///     .build();
    /// let token = p.get_write_token().unwrap();
    /// token.buf = "world".to_string();
    /// token.done();
    /// println!("{}", token.buf); // This line should cause a compile-time error
    /// ```
    pub fn done(self) {
        let _ = self.inner.sender.send(self.buf);
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
}
