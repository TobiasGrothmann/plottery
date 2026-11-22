use std::sync::{LazyLock, Mutex, PoisonError};

use rand::{rngs::StdRng, RngExt, SeedableRng};

static RNG: LazyLock<Mutex<StdRng>> = LazyLock::new(|| Mutex::new(rand::make_rng::<StdRng>()));

/// Runs `f` with the global generator. A panic while it was held leaves no broken state, so poisoning is ignored.
pub(crate) fn with_rng<T>(f: impl FnOnce(&mut StdRng) -> T) -> T {
    f(&mut RNG.lock().unwrap_or_else(PoisonError::into_inner))
}

/// Seeds the global random number generator behind the `rand_*` functions and new [`crate::Noise`]s.
pub fn seed(seed: i32) {
    with_rng(|rng| *rng = StdRng::seed_from_u64(seed as u64));
}

pub fn seed_random() {
    seed(rand::rng().random::<i32>());
}

/// Seeds the generator for a test that uses it; the guard keeps parallel tests out until dropped.
#[cfg(test)]
pub(crate) fn seed_for_test(seed_value: i32) -> std::sync::MutexGuard<'static, ()> {
    static TEST_LOCK: Mutex<()> = Mutex::new(());
    let guard = TEST_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
    seed(seed_value);
    guard
}
