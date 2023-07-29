use gloo_timers::future::sleep;
use std::fmt::Debug;
use std::time::Duration;

pub async fn delayed_assert_eq<FL, L, FR, R>(left: FL, right: FR)
where
    FL: Fn() -> L,
    FR: Fn() -> R,
    L: PartialEq<R>,
    L: Debug,
    R: Debug,
{
    'outer: for i in 0..2 {
        if i > 0 {
            assert_eq!(left(), right());
        }

        for _ in 0..100 {
            sleep(Duration::from_millis(10)).await;
            if left() == right() {
                break 'outer;
            }
        }
    }
}
