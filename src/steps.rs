/// Various steps in the `gccrs` compilation pipeline
pub enum CompileStep {
    Expansion,
    TypeCheck,
    End,
}

impl CompileStep {
    /// All the available compilation steps which `gccrs` can stop on
    pub fn variants() -> [CompileStep; 3] {
        [
            CompileStep::Expansion,
            CompileStep::TypeCheck,
            CompileStep::End,
        ]
    }

    /// Get the `gccrs` compile option associated with a compilation step
    pub fn compile_option(&self) -> &str {
        match self {
            CompileStep::Expansion => "-frust-compile-until=expansion",
            CompileStep::TypeCheck => "-frust-compile-until=typecheck",
            CompileStep::End => "-frust-compile-until=end",
        }
    }
}
