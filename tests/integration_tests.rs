use ::cyclic_pipe;
use std::thread;

#[test]
fn example() {
    let frame_size = 1000;
    let num_frames = 1000;

    let (p, c) = cyclic_pipe::Builder::<Vec<f32>>::new()
        .with_size(2)
        .with_init_template(vec![std::f32::NAN; frame_size])
        .build();

    let handle_p = thread::spawn(move || {
        let mut handles_writing = Vec::new();
        for i in 0..num_frames {
            let mut token = p.get_write_token().unwrap();
            assert!(token.buf.len() == frame_size);

            // since writting and pushing are done in spawn threads,
            // the finishing order is not guaranteed.
            handles_writing.push(thread::spawn(move || {
                token.buf[0] = i as f32; 
                token.done(); 
            }));
        }
        for handle in handles_writing {
            let _ = handle.join();
        }
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
