use std::sync::{LazyLock, Mutex};

use rand::{rngs::StdRng, Rng, SeedableRng};

pub static RNG: LazyLock<Mutex<StdRng>> = LazyLock::new(|| Mutex::new(StdRng::from_entropy()));

/// Seeds the global random number generator behind the `rand_*` functions and new [`crate::Noise`]s.
pub fn seed(seed: i32) {
    if let Ok(mut rng) = RNG.lock() {
        *rng = StdRng::seed_from_u64(seed as u64);
    }
}

pub fn seed_random() {
    seed(rand::thread_rng().gen::<i32>());
}

/// Seeds the generator for a test that uses it; the guard keeps parallel tests out until dropped.
#[cfg(test)]
pub(crate) fn seed_for_test(seed_value: i32) -> std::sync::MutexGuard<'static, ()> {
    static TEST_LOCK: Mutex<()> = Mutex::new(());
    let guard = TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    seed(seed_value);
    guard
}
