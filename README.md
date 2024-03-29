# 🚰 Waterworks 

`waterworks` is a library for building pipelines of chained computations,
with a focus on being explicit about opting in as a stage of a given pipeline
and for a quick and convenient way of inspecting the output of each stage and acting on that output to potentially terminate the pipeline early.

It was designed to be used for the flow of data in a compiler, and is put to
"production" use in my personal compiler project, [`catastrophic`](https://github.com/samuelsleight/catastrophic-lang).

## Usage

Usage of `waterworks` first requires defining the individual stages that the
overall computation pipeline will be put together with.

A stage is defined by the `Error` type global to the pipeline, the input and 
output of that specific stage, and the `run` method for performing the
actual computation.

For example, if we were constructing a compiler we might have an initial
file parsing stage like so:

```rust
struct FileParseStage;

impl Stage<CompileError> for FileParseStage {
    type Input = PathBuf;
    type Output = Ast;

    fn run(self, input: PathBuf) -> Result<Ast, CompileError> {
        parse_file(input)
    }
}
```

This can then be put together into a full pipleine, chained into adsitional
compilation stages:

```rust
let pipe = pipeline(FileParseStage, |_| ())
    .and_then(AstAnalysisStage, |_| ())
    .and_then(OptimisationStage, |_| ())
    .and_then(ComppilationStage, |_| ());
```

### Callbacks

The pipeline construction and chaining methods both take a callback in
addition to the stage being attached to the pipeline. This callback can be 
used to both inspect the result of the associated stage and to terminate the
execution of the pipeline early.

For example, say you wanted a compiler flag that simply logged the output of
the parser and then returned - you could implement that with the following callback when attaching the `FileParseStage` above:

```rust
|ast| if log_ast {
    println!("{:?}", ast);
    Continue::Cancel
} else {
    Continue::Continue
}
```

For convenience in trivial cases, a `()`-returning closure is treated as if
it was explicitly returning `Continue::Continue`

### Running

To then run a pipeline, it's a simple case of calling `run`, passing in the
first stage's defined input. This will then return a result of either the
output of the final stage in the pipeline, the first error that gets returned
by a stage, or whether the pipeline was cancelled early.

```rust
let result = pipe.run("waterworks.rs");
```
