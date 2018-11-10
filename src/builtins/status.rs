use std::path::PathBuf;
use crate::process_pool::ProcessPool;
pub fn status(cwd: &PathBuf, pool: &ProcessPool) {
    println!("CWD: {}", cwd.to_str().unwrap());
    println!("Pool has {} living processes.", pool.len());
}
