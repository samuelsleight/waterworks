#[derive(Debug)]
pub enum Err<E> {
    Cancelled,
    Err(E)
}

#[derive(Debug)]
pub enum Continue {
    Continue,
    Cancel
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

pub trait RunPipeline<Error> {
    type Start;
    type End;

    fn run(self, input: Self::Start) -> Result<Self::End, Err<Error>>;
}

pub trait Pipeline<Error, S, FR, F>: RunPipeline<Error>
    where
        S: Stage<Error, Input=Self::End>,
        FR: Into<Continue>,
        F: FnOnce(&S::Output) -> FR {

    type AndThen;

    fn and_then(self, stage: S, callback: F) -> Self::AndThen;
}

pub struct PipelineStage<S, F, Next> {
    stage: S,
    callback: F,
    next: Next
}

pub struct PipelineEnd<S, F> {
    stage: S,
    callback: F
}

impl<S, FR, F, Error, T, GR, G> Pipeline<Error, T, GR, G> for PipelineEnd<S, F> 
    where 
        S: Stage<Error>,
        FR: Into<Continue>,
        F: FnOnce(&S::Output) -> FR,
        T: Stage<Error, Input=S::Output>,
        GR: Into<Continue>,
        G: FnOnce(&T::Output) -> GR {

    type AndThen = PipelineStage<S, F, PipelineEnd<T, G>>;

    fn and_then(self, stage: T, callback: G) -> Self::AndThen {
        PipelineStage {
            stage: self.stage,
            callback: self.callback,
            next: PipelineEnd {
                stage: stage,
                callback: callback,
            }
        }
    }
}

impl<S, FR, F, P, Error, T, GR, G> Pipeline<Error, T, GR, G> for PipelineStage<S, F, P> 
    where 
        S: Stage<Error, Output=P::Start>, 
        FR: Into<Continue>,
        F: FnOnce(&S::Output) -> FR, 
        P: Pipeline<Error, T, GR, G>,
        T: Stage<Error, Input=P::End>,
        GR: Into<Continue>,
        G: FnOnce(&T::Output) -> GR {

    type AndThen = PipelineStage<S, F, P::AndThen>;

    fn and_then(self, stage: T, callback: G) -> Self::AndThen {
        PipelineStage {
            stage: self.stage,
            callback: self.callback,
            next: self.next.and_then(stage, callback)
        }
    }
}

impl<S, FR, F, Error> RunPipeline<Error> for PipelineEnd<S, F>
    where
        S: Stage<Error>,
        FR: Into<Continue>,
        F: FnOnce(&S::Output) -> FR {

    type Start = S::Input;
    type End = S::Output;

    fn run(self, input: Self::Start) -> Result<Self::End, Err<Error>> {
        match self.stage.run(input) {
            Ok(output) => match (self.callback)(&output).into() {
                Continue::Continue => Ok(output),
                Continue::Cancel => Err(Err::Cancelled)
            },

            Err(err) => Err(Err::Err(err))
        }
    }
}

impl<S, FR, F, Error, P> RunPipeline<Error> for PipelineStage<S, F, P>
    where
        S: Stage<Error>,
        FR: Into<Continue>,
        F: FnOnce(&S::Output) -> FR,
        P: RunPipeline<Error, Start=S::Output> {

    type Start = S::Input;
    type End = P::End;

    fn run(self, input: Self::Start) -> Result<Self::End, Err<Error>> {
        match self.stage.run(input) {
            Ok(output) => match (self.callback)(&output).into() {
                Continue::Continue => self.next.run(output),
                Continue::Cancel => Err(Err::Cancelled)
            },

            Err(err) => Err(Err::Err(err)),
        }
    }
}

pub fn pipeline<S, FR, F, Error>(stage: S, callback: F) -> PipelineEnd<S, F>
    where
        S: Stage<Error>,
        FR: Into<Continue>,
        F: FnOnce(&S::Output) -> FR {

    PipelineEnd {
        stage: stage,
        callback: callback
    }
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
            input.into_iter().map(|i| write!(s, "{}", i)).collect::<Result<Vec<_>, _>>().map(|_| s)
        }
    }

    #[test]
    fn run_test() {
        println!("{:?}", pipeline(StageA, |v| for i in v { println!("{}", i)})
            .and_then(StageB, |_| {})
            .run(()))
    }

    #[test]
    fn cancel_test() {
        println!("{:?}", pipeline(StageA, |_| Continue::Cancel)
            .and_then(StageB, |_| {})
            .run(()))
    }
}
