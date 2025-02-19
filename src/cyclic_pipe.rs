/// To create a cyclic pipe and return producer and consumer ends.
pub mod builder;

/// To get empty buffer from pipe and push written data to pipe.
pub mod consumer;

/// To get written data and return used buffer to pipe.
pub mod producer;

/// To access avaliable(writable/readable) buffer.
pub mod token;
