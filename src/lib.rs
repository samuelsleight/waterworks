pub trait Stage<Error> {
    type Input;
    type Output;

    fn run(self, input: Self::Input) -> Result<Self::Output, Error>;
}

pub trait RunPipeline<Error> {
    type Start;
    type End;

    fn run(self, input: Self::Start) -> Result<Self::End, Error>;
}

pub trait Pipeline<Error, S, F>: RunPipeline<Error>
    where
        S: Stage<Error, Input=Self::End>,
        F: FnOnce(&S::Output) {

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

impl<S, F, Error, T, G> Pipeline<Error, T, G> for PipelineEnd<S, F> 
    where 
        S: Stage<Error>, 
        F: FnOnce(&S::Output),
        T: Stage<Error, Input=S::Output>,
        G: FnOnce(&T::Output) {

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

impl<S, F, P, Error, T, G> Pipeline<Error, T, G> for PipelineStage<S, F, P> 
    where 
        S: Stage<Error, Output=P::Start>, 
        F: FnOnce(&S::Output), 
        P: Pipeline<Error, T, G>,
        T: Stage<Error, Input=P::End>,
        G: FnOnce(&T::Output) {

    type AndThen = PipelineStage<S, F, P::AndThen>;

    fn and_then(self, stage: T, callback: G) -> Self::AndThen {
        PipelineStage {
            stage: self.stage,
            callback: self.callback,
            next: self.next.and_then(stage, callback)
        }
    }
}

impl<S, F, Error> RunPipeline<Error> for PipelineEnd<S, F>
    where
        S: Stage<Error>,
        F: FnOnce(&S::Output) {

    type Start = S::Input;
    type End = S::Output;

    fn run(self, input: Self::Start) -> Result<Self::End, Error> {
        match self.stage.run(input) {
            Ok(output) => {
                (self.callback)(&output);
                Ok(output)
            },

            err => err
        }
    }
}

impl<S, F, Error, P> RunPipeline<Error> for PipelineStage<S, F, P>
    where
        S: Stage<Error>,
        F: FnOnce(&S::Output),
        P: RunPipeline<Error, Start=S::Output> {

    type Start = S::Input;
    type End = P::End;

    fn run(self, input: Self::Start) -> Result<Self::End, Error> {
        match self.stage.run(input) {
            Ok(output) => {
                (self.callback)(&output);
                self.next.run(output)
            },

            Err(err) => Err(err),
        }
    }
}

pub fn pipeline<S, F, Error>(stage: S, callback: F) -> PipelineEnd<S, F>
    where
        S: Stage<Error>,
        F: FnOnce(&S::Output) {

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
    fn test() {
        println!("{:?}", pipeline(StageA, |v| for i in v { println!("{}", i)})
            .and_then(StageB, |_| {})
            .run(()))
    }
}
