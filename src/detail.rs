use super::*;

struct PipelineStage<S, F, Next> {
    stage: S,
    callback: F,
    next: Next,
}

pub(crate) struct PipelineEnd<S, F> {
    stage: S,
    callback: F,
}

impl<S, F> PipelineEnd<S, F> {
    pub(crate) fn new(stage: S, callback: F) -> Self {
        Self { stage, callback }
    }
}

impl<PS, PF, PR, Error> Extend<Error> for PipelineEnd<PS, PF>
where
    PS: Stage<Error>,
    PF: FnOnce(&PS::Output) -> PR,
    PR: Into<Continue>,
{
    fn and_then<S, F, R>(
        self,
        stage: S,
        callback: F,
    ) -> impl Extend<Error, Start = Self::Start, End = S::Output>
    where
        S: Stage<Error, Input = Self::End>,
        F: FnOnce(&S::Output) -> R,
        R: Into<Continue>,
    {
        PipelineStage {
            stage: self.stage,
            callback: self.callback,
            next: PipelineEnd { stage, callback },
        }
    }
}

impl<PS, PF, PR, Next, Error> Extend<Error> for PipelineStage<PS, PF, Next>
where
    PS: Stage<Error>,
    PF: FnOnce(&PS::Output) -> PR,
    PR: Into<Continue>,
    Next: Extend<Error, Start = PS::Output>,
{
    fn and_then<S, F, R>(
        self,
        stage: S,
        callback: F,
    ) -> impl Extend<Error, Start = Self::Start, End = S::Output>
    where
        S: Stage<Error, Input = Self::End>,
        F: FnOnce(&S::Output) -> R,
        R: Into<Continue>,
    {
        PipelineStage {
            stage: self.stage,
            callback: self.callback,
            next: self.next.and_then(stage, callback),
        }
    }
}

impl<PS, PF, PR, Error> Pipeline<Error> for PipelineEnd<PS, PF>
where
    PS: Stage<Error>,
    PF: FnOnce(&PS::Output) -> PR,
    PR: Into<Continue>,
{
    type Start = PS::Input;
    type End = PS::Output;

    fn run(self, input: Self::Start) -> PipelineResult<Self::End, Error> {
        match self.stage.run(input) {
            Ok(output) => match (self.callback)(&output).into() {
                Continue::Continue => PipelineResult::Ok(output),
                Continue::Cancel => PipelineResult::Cancelled,
            },

            Err(err) => PipelineResult::Err(err),
        }
    }
}

impl<PS, PF, PR, Error, Next> Pipeline<Error> for PipelineStage<PS, PF, Next>
where
    PS: Stage<Error>,
    PF: FnOnce(&PS::Output) -> PR,
    PR: Into<Continue>,
    Next: Pipeline<Error, Start = PS::Output>,
{
    type Start = PS::Input;
    type End = Next::End;

    fn run(self, input: Self::Start) -> PipelineResult<Self::End, Error> {
        match self.stage.run(input) {
            Ok(output) => match (self.callback)(&output).into() {
                Continue::Continue => self.next.run(output),
                Continue::Cancel => PipelineResult::Cancelled,
            },

            Err(err) => PipelineResult::Err(err),
        }
    }
}
