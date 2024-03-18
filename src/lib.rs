mod detail;
mod test;

/// A `PipelineResult` is the type returned when a pipeline is ran,
/// i.e., by calling the [`run`][Pipeline::run] method on the [`Pipeline`] trait
#[derive(Debug)]
pub enum PipelineResult<T, E> {
    /// The pipeline ran successfully to the end, returning the [`Output`][Stage::Output]
    /// of its final [`Stage`]
    Ok(T),

    /// One of the stages of the pipeline returned an error
    Err(E),

    /// One of the stages of the pipeline was [cancelled][Continue::Cancel] by
    /// its inspection callback
    Cancelled,
}

/// `Continue` describes whether a pipeline should continue running after
/// its inspection callback returns.
///
/// The intent of this is to make it easy to allow for convenient debuggability
/// by allowing partial running of a pipeline up to the point that needs
/// debugging/inspecting
///
/// In the minimal case of an empty callback or a callback that just
/// logs the value, an implementation of [`From<()>`][Continue::from] exists for convenience
#[derive(Debug)]
pub enum Continue {
    /// Continue running this pipleine as normal, passing this stage's [`Output`][Stage::Output]
    /// to the next stage
    Continue,

    /// Stops running the pipeline, returning [`Cancelled`][PipelineResult::Cancelled] to the
    /// pipeline's [`run`][Pipeline::run] call
    Cancel,
}

/// A convenience implementation allowing for minimal implementations of a callback
/// without needing to explicitly return [`Continue`][Continue::Continue]
impl From<()> for Continue {
    /// Returns [`Continue::Continue`]
    fn from(_: ()) -> Continue {
        Continue::Continue
    }
}

/// A `Stage` describes a single, potentially fallible, step of a pipeline.
/// It takes (via the [`run`][Stage::run] method), a specific [`Input`][Stage::Input]
/// and returns either its defined [`Output`][Stage::Output] or the pipeline's
/// defined `Error` type
pub trait Stage<Error> {
    /// The type passed to this stage by either the preceding stage in the pipeline
    /// or at the start of the stage via the pipeline itself's [`run`][Pipeline::run] method
    type Input;

    /// The type produced by this stage on successful execution to pass either to the next
    /// stage of the pipeline or to the result of the pipeline's [`run`][Pipeline::run] call
    type Output;

    /// Runs this stage with the given input parameter, returning either an instance of
    /// the stage's [`Output`][Stage::Output] to pass to the next stage or an instance of the
    /// pipeline's `Error` type to return to the pipeline's [`run`][Pipeline::run] call as
    /// a [`PipelineResult::Err`]
    fn run(self, input: Self::Input) -> Result<Self::Output, Error>;
}

/// A `Pipeline` describes a full sequence of [`Stage`s][Stage], each executing in
/// sequence. The pipeline as a whole takes in a defined [`Start`][Pipeline::Start]
/// (the [`Input`][Stage::Input] of the first `Stage`) and returns a [`PipelineResult`]
/// of either [`End`][Pipeline::End] (the [`Output`][Stage::Output])of the final `Stage`),
/// `Error` if either of the `Stage`s returns an error, or [`PipelineResult::Cancelled`] if
/// any of the stagee's inspection callbacks cancels the pipeline execution (see [`Continue`])
pub trait Pipeline<Error> {
    /// The [`Input`][Stage::Input] type of the first [`Stage`] in this pipeline
    type Start;

    /// The [`Output`][Stage::Output] type of the last [`Stage`] in this pipeline
    type End;

    /// Run the entire pipeline, [`Stage`] by `Stage`, until either the end of the pipeline
    /// or one of the `Stage`s returns an error or is [cancelled][Continue::Cancel]
    fn run(self, input: Self::Start) -> PipelineResult<Self::End, Error>;
}

/// The `Extend` trait allows extending a pipeline by attaching a new [`Stage`] to it
/// that takes the current pipeline's [`End`][Pipeline::End] type as [`Input`][Stage::Input]
pub trait Extend<Error>: Pipeline<Error> {
    /// Extends the current pipeline, returning a new pipeline with the same [`Start`][Pipeline::Start]
    /// but with the [`End`][Pipeline::End] of the newly extending [`Stage`].
    ///
    /// The `callback` passed-in is used to conveniently inspect the result of the new `Stage`,
    /// and can also be utilised to prematurely terminate a running pipeline. See [`Continue`]
    /// for more information on the return type of this callback
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

/// The `pipeline` method constructs the beginning of a pipeline, running a single [`Stage`]
/// with its associated inspection `callback`. See [`Extend::and_then`] for more information
/// about the inspection callback
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
