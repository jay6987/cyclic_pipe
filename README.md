# cyclic_pipe

## Cyclic Pipe

`cyclic_pipe` is a Rust library that provides fixed-size, buffer pre-allocated cyclic pipes
which supports multi-producer and multi-consumer concurrent access.

This crate is designed to facilitate efficient and synchronized data transfer between
different parts of a program, using a cyclic buffer to manage the data flow.

The cyclic pipe works like a cyclic conveyor belt with fixed number of buckets on it. Producers wait for
an empty bucket to put data in, while consumers wait for a bucket with data to take out. Producers put data
into the empty bucket and then the bucket is moved to the consumer side. The consumer takes the data out of
the bucket and then the empty bucket is moved back to the producer side. The cyclic pipe supports multi-producers
and multi-consumers, and ensures that buffer orders are not violated.

### Features

- Fixed-size: The size of the pipe is determined at creation time and does not change.
- Pre-allocated buffers: The buffers are allocated once and reused, avoiding the overhead of repeated allocations.
- Synchronized access: Ensures safe concurrent access to the buffer for multi-writter and multi-reader.
- Sequential access: First avaliable buffer will be first written then first transfered to consumer to read.

### Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
cyclic_pipe = "0.1.0"
```

Import the crate in your code:

```rust
extern crate cyclic_pipe;
```

### Example

```rust
use std::thread;

fn main() {
    let frame_size = 1000;
    let num_frames = 1000;

    let (p, c) = cyclic_pipe::Builder::<Vec<f32>>::new()
        .with_size(2)
        .with_init_template(vec![std::f32::NAN; frame_size])
        .build();

    let handle_p = thread::spawn(move || {
        let mut writing_handles = Vec::new();
        for i in 0..num_frames {
            let mut token = p.get_write_token().unwrap();
            assert!(token.buf.len() == frame_size,
"buffer is pre-allocated and not changed");

            // since writting and pushing are done in spawn threads,
            // the finishing order is not guaranteed.
            writing_handles.push(thread::spawn(move || {
                token.buf[0] = i as f32;
                token.done();
            }));
        }
        for h in writing_handles {
            let _ = h.join();
        }
    });

    let handle_c = thread::spawn(move || {
        for i in 0..num_frames {
            let token = c.get_read_token().unwrap();

            assert_eq!(token.buf[0], i as f32,
"reading tokens are always in order with the writting tokens were got instead of done");
            token.done();
        }
    });

    let _ = handle_p.join();
    let _ = handle_c.join();
}
```

### Modules

- `Builder`: To create a cyclic pipe and return producer and consumer ends.
- `Producer`: To get empty buffer from pipe and push written data to pipe.
- `Consumer`: To get written data and return used buffer to pipe.
- `Token`: To access avaliable(writable/readable) buffer.

### License

This project is licensed under

- [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)
- [MIT license](http://opensource.org/licenses/MIT)

at your option.

### Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
