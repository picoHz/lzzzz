mod lz4f;
mod lz4f_stream;

use pbr::ProgressBar;
use rayon::prelude::*;
use std::{cmp, env, fmt, sync::mpsc::channel, thread};

pub fn run<R>(func: fn(u64) -> Result<(), R>)
where
    R: Send + fmt::Debug,
{
    let count = env::var_os("LZZZZ_FUZZER_ITERATION")
        .and_then(|s| s.into_string().ok())
        .and_then(|n| n.parse().ok())
        .unwrap_or(10000u64);

    let step = 100;

    let (tx, rx) = channel();

    thread::spawn(move || {
        let mut pb = ProgressBar::new(count);
        while !pb.is_finish {
            match rx.recv() {
                Ok(n) => {
                    pb.add(n);
                }
                Err(_) => break,
            };
            pb.tick();
        }
        pb.set(count);
        pb.finish_print("done");
    });

    let err = (0..step)
        .into_par_iter()
        .map(|s| ((count / step) * s)..cmp::min((count / step) * (s + 1), count))
        .map_with(tx, |tx, range| {
            let len = range.end - range.start;
            let err = range
                .into_par_iter()
                .map(func)
                .find_map_any(|result| result.err());
            let _ = tx.send(len);
            err
        })
        .find_map_any(|err| err);
    if let Some(err) = err {
        panic!("{:?}", err);
    }
}
