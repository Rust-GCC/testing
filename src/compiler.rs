//! Builder pattern around compiler invocations. This is a wrapper around [`Command`],
//! with adequate defaults and added functions or types to help make compiler invocations
//! in the testing project safer, easier and less verbose.

use std::path::Path;
use std::process::{Command, Stdio};

use crate::args::Args;

/// All *used* Rust editions
#[derive(Clone, Copy)]
pub enum Edition {
    E2021,
}

impl Edition {
    fn to_str(self) -> &'static str {
        match self {
            Edition::E2021 => "2021",
        }
    }
}

/// All compiler kinds used in the testsuite
pub enum Kind {
    Gccrs,
    RustcBootstrap,
}

impl Kind {
    /// Get the path associated with a specific compiler kind
    fn as_path_from_args<'args>(&self, args: &'args Args) -> &'args Path {
        match self {
            Kind::Gccrs => &args.gccrs,
            Kind::RustcBootstrap => &args.rustc,
        }
    }
}

/// Extend the [`Command`] type with functions associated with the compiler we're going to run
trait CommandExt {
    /// Set the default arguments for a specific compiler
    fn default_args(&mut self, kind: &Kind) -> &mut Command;

    /// Set the default environment variables for a specific compiler
    fn default_env(&mut self, kind: &Kind) -> &mut Command;
}

impl CommandExt for Command {
    fn default_args<'cmd>(&'cmd mut self, kind: &Kind) -> &'cmd mut Command {
        match kind {
            // specify Rust language by default, which allows us to compile Rust files with funny extensions
            Kind::Gccrs => self.arg("-x").arg("rust"),
            Kind::RustcBootstrap => self,
        }
    }

    fn default_env(&mut self, kind: &Kind) -> &mut Command {
        match kind {
            Kind::Gccrs => self,
            Kind::RustcBootstrap => self.env("RUSTC_BOOTSTRAP", "1"),
        }
    }
}

/// Represents a compiler invocation
pub struct Compiler {
    cmd: Command,
    kind: Kind,
    stdout: Option<Stdio>,
    stderr: Option<Stdio>,
}

impl Compiler {
    /// Create a new compiler invocation
    pub fn new(kind: Kind, args: &Args) -> Compiler {
        Compiler {
            cmd: Command::new(kind.as_path_from_args(args)),
            kind,
            stdout: None,
            stderr: None,
        }
    }

    fn kind(&self) -> &Kind {
        &self.kind
    }

    /// Set the crate name to use for a compiler invocation. This is equivalent
    /// to `--crate-name` for `rustc` and `-frust-crate-name` for `gccrs`
    pub fn crate_name(&mut self, crate_name: &str) -> &mut Compiler {
        match self.kind() {
            Kind::Gccrs => self.cmd.arg("-frust-crate-name"),
            Kind::RustcBootstrap => self.cmd.arg("--crate-name"),
        };

        self.cmd.arg(crate_name);
        self
    }

    /// Set the edition to use for a compiler invocation. This is equivalent to
    /// `--edition` for `rustc` and `-frust-edition` for `gccrs`
    pub fn edition(&mut self, edition: Edition) -> &mut Compiler {
        match self.kind() {
            Kind::Gccrs => self.cmd.arg("-frust-edition"),
            Kind::RustcBootstrap => self.cmd.arg("--edition"),
        };

        self.cmd.arg(edition.to_str());
        self
    }

    /// Access the underlaying [`Command`] of a compiler invocation. This is a destructive operation
    /// and should only be done as the last step of the building process. You can then choose to pass
    /// additional arguments, spawn the command, etc... as you would with a regularly built [`Command`]
    pub fn command(&mut self) -> &mut Command {
        let kind = &self.kind;

        self.cmd
            .default_args(kind)
            .default_env(kind)
            // We want errors and output to be silent by default in the testing project
            .stderr(self.stdout.take().unwrap_or_else(Stdio::null))
            .stdout(self.stderr.take().unwrap_or_else(Stdio::null))
    }
}