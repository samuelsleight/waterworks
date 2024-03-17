mod detail;
mod test;

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
