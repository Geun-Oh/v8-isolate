use std::sync::Mutex;

use v8::{Isolate, OwnedIsolate};

pub struct IsolateWithIdx {
    pub isolate: OwnedIsolate,
    pub idx: usize,
}

pub struct IsolatePool {
    pool: Mutex<Vec<IsolateWithIdx>>,
    size: usize,
}

impl IsolatePool {
    pub fn new(size: usize) -> Self {
        let mut isolates = Vec::with_capacity(size);
        for idx in 0..size {
            let create_params = v8::Isolate::create_params().heap_limits(0, 10 * 1024 * 1024);
            let isolate = Isolate::new(create_params);
            isolates.push(IsolateWithIdx { isolate, idx });
        }
        Self {
            pool: Mutex::new(isolates),
            size,
        }
    }

    pub fn get_isolate(&self) -> Option<IsolateWithIdx> {
        let mut pool = self.pool.lock().unwrap();
        pool.pop()
    }

    pub fn return_isolate(&self, isolate: IsolateWithIdx) {
        let mut pool = self.pool.lock().unwrap();
        pool.push(isolate);
    }
}

impl Drop for IsolatePool {
    fn drop(&mut self) {
        let mut pool = self.pool.lock().unwrap();
        let mut idx = self.size - 1;

        while idx > 0 {
            if let Some(pos) = pool.iter().position(|isolate_with_idx| isolate_with_idx.idx == idx) {
                pool.remove(pos);
                idx -= 1;
            }
        }
    }
}