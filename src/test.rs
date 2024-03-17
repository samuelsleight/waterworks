#![cfg(test)]

use super::*;
use std::fmt::Write;

struct StageA;
struct StageB;

impl Stage<std::fmt::Error> for StageA {
    type Input = ();
    type Output = Vec<i64>;

    fn run(self, _: ()) -> Result<Vec<i64>, std::fmt::Error> {
        Ok(vec![12, 13, 14, 15, 1])
    }
}

impl Stage<std::fmt::Error> for StageB {
    type Input = Vec<i64>;
    type Output = String;

    fn run(self, input: Vec<i64>) -> Result<String, std::fmt::Error> {
        let mut s = String::new();
        input
            .into_iter()
            .map(|i| write!(s, "{}", i))
            .collect::<Result<Vec<_>, _>>()
            .map(|_| s)
    }
}

#[test]
fn run_test() {
    println!(
        "{:?}",
        pipeline(StageA, |v| for i in v {
            println!("{}", i)
        })
        .and_then(StageB, |_| {})
        .run(())
    )
}

#[test]
fn cancel_test() {
    println!(
        "{:?}",
        pipeline(StageA, |_| Continue::Cancel)
            .and_then(StageB, |_| {})
            .run(())
    )
}
