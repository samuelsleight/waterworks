mod detail;

#[derive(Debug)]
pub enum PipelineResult<T, E> {
    Ok(T),
    Err(E),
    Cancelled,
}

#[derive(Debug)]
pub enum Continue {
    Continue,
    Cancel,
}

impl From<()> for Continue {
    fn from(_: ()) -> Continue {
        Continue::Continue
    }
}

pub trait Stage<Error> {
    type Input;
    type Output;

    fn run(self, input: Self::Input) -> Result<Self::Output, Error>;
}

pub trait Pipeline<Error> {
    type Start;
    type End;

    fn run(self, input: Self::Start) -> PipelineResult<Self::End, Error>;
}

pub trait Extend<Error>: Pipeline<Error> {
    fn and_then<S, F, R>(
        self,
        stage: S,
        callback: F,
    ) -> impl Extend<Error, Start = Self::Start, End = S::Output>
    where
        S: Stage<Error, Input = Self::End>,
        F: FnOnce(&S::Output) -> R,
        R: Into<Continue>;
}

pub fn pipeline<S, FR, F, Error>(
    stage: S,
    callback: F,
) -> impl Extend<Error, Start = S::Input, End = S::Output>
where
    S: Stage<Error>,
    FR: Into<Continue>,
    F: FnOnce(&S::Output) -> FR,
{
    detail::PipelineEnd::new(stage, callback)
}

#[cfg(test)]
mod test {
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
}
