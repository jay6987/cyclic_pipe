use ::cyclic_pipe;
use std::thread::{self, sleep};
use threadpool::ThreadPool;

#[test]
fn example() {
    let frame_size = 1000;
    let num_frames = 1000;
    let num_workers = 2;

    let (p, c) = cyclic_pipe::Builder::<Vec<f32>>::new()
        .with_size(2)
        .with_init_template(vec![std::f32::NAN; frame_size])
        .build();

    let handle_p = thread::spawn(move || {
        let workers = ThreadPool::new(num_workers);
        for i in 0..num_frames {
            let mut token = p.get_write_token().unwrap();
            assert!(token.buf.len() == frame_size);

            // since writting and pushing are done in spawn threads,
            // the finishing order is not guaranteed.
            workers.execute(move || {
                token.buf[0] = i as f32;
                token.done();
            });
        }
        workers.join();
    });

    let handle_c = thread::spawn(move || {
        for i in 0..num_frames {
            let token = c.get_read_token().unwrap();
            assert!(token.buf.len() == frame_size);

            // reading tokens are always in order with the
            // writting tokens were got instead of done.
            assert_eq!(token.buf[0], i as f32);
            token.done();
        }
    });

    let _ = handle_p.join();
    let _ = handle_c.join();
}

#[test]
fn producer_disconnected() {
    let (p, c) = cyclic_pipe::Builder::<Vec<f32>>::new()
        .with_size(1)
        .with_init_template(vec![std::f32::NAN; 1000])
        .build();

    drop(p);

    assert!(c.get_read_token().is_err());
}

#[test]
fn consumer_disconnected() {
    let (p, c) = cyclic_pipe::Builder::<Vec<f32>>::new()
        .with_size(2)
        .with_init_template(vec![std::f32::NAN; 1000])
        .build();

    // the 1st write token should be ok to get
    let wt = p.get_write_token().unwrap();

    // consumer is disconnected here.
    drop(c);

    assert!(wt.buf.len() == 1000);

    wt.done();

    // the 2nd write token should NOT be ok to get
    assert!(p.get_write_token().is_err());
}

#[test]
fn procuder_disconnected() {
    let (p, c) = cyclic_pipe::Builder::new()
        .with_size(2)
        .with_init_template("hello".to_string())
        .build();

    // the 1st write token should be ok to get
    let wt = p.get_write_token().unwrap();
    wt.done();

    // producer is disconnected here.
    drop(p);

    // the 1st read token should be able to get since the 1st write token is done.
    let rt = c.get_read_token().unwrap();
    assert!(rt.buf == "hello");
    rt.done();

    // the 2nd read token should NOT be ok to get.
    assert!(c.get_read_token().is_err())
}

#[test]
fn buffer_orderring() {
    let (p, c) = cyclic_pipe::Builder::new()
        .with_size(2)
        .with_init_template("".to_string())
        .build();

    let mut wt1 = p.get_write_token().unwrap();
    let mut wt2 = p.get_write_token().unwrap();

    wt2.buf = "world".to_string();
    wt2.done();

    wt1.buf = "hello".to_string();
    wt1.done();

    let rt1 = c.get_read_token().unwrap();
    assert!(rt1.buf == "hello");
    rt1.done();

    let rt2 = c.get_read_token().unwrap();
    assert!(rt2.buf == "world");
    rt2.done();
}

#[test]
fn token_done_never_throw_even_channel_closed() {
    let (p, c) = cyclic_pipe::Builder::new()
        .with_size(2)
        .with_init_template("".to_string())
        .build();

    let wt = p.get_write_token().unwrap();
    wt.done();

    let wt = p.get_write_token().unwrap();

    let rt = c.get_read_token().unwrap();

    drop(p);
    drop(c);

    wt.done();
    rt.done();
}

#[test]
fn consumer_blocked_until_producer_write_done() {
    let (p, c) = cyclic_pipe::Builder::new()
        .with_size(1)
        .with_init_template("".to_string())
        .build();
    let handle = thread::spawn(move || {
        let rt = c.get_read_token().unwrap();
        assert!(rt.buf == "hello");
        rt.done();
    });

    let mut wt = p.get_write_token().unwrap();
    wt.buf = "hello".to_string();

    sleep(std::time::Duration::from_millis(10));

    assert!(!handle.is_finished());

    wt.done();
    let _ = handle.join();
}

#[test]
fn producer_blocked_until_consumer_read_done() {
    let (p, c) = cyclic_pipe::Builder::new()
        .with_size(1)
        .with_init_template("".to_string())
        .build();

    let mut wt = p.get_write_token().unwrap();
    wt.buf = "hello".to_string();
    wt.done();

    let handle = thread::spawn(move || {
        let mut wt = p.get_write_token().unwrap();
        wt.buf = "world".to_string();
        wt.done();
    });

    sleep(std::time::Duration::from_millis(10));

    let rt = c.get_read_token().unwrap();
    assert!(rt.buf == "hello");

    assert!(!handle.is_finished());

    rt.done();

    let _ = handle.join();
}

#[test]
fn token_drop_without_done_cause_error() {
    let (p, c) = cyclic_pipe::Builder::new()
        .with_size(2)
        .with_init_template("hello".to_string())
        .build();

    {
        let _wt = p.get_write_token().unwrap();
        // write token drop here without calling done().
    }

    assert!(c.get_read_token().is_err());
}
