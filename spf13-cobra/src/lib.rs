// Port of github.com/spf13/cobra@v1.10.2
//
// go: package cobra — files ported:
//   cobra.go   → src/cobra.rs   (template machinery not ported, see file header)
//   args.go    → src/args.rs    (full)
//   command.go → src/command.rs (core; per-method divergences documented inline)
//
// NOT ported (KNOWN DIVERGENCE, out of scope for the core port):
//   completions.go, shell_completions.go (except the annotation constants and
//   MarkFlagRequired family, which command.go's required-flag validation needs),
//   active_help.go, flag_groups.go, bash/zsh/fish/powershell generators,
//   command_win.go (windows mousetrap), text/template support (cobra v1.10
//   renders default usage/help/version through native functions, which ARE
//   ported; only user-supplied custom templates are unsupported).
#![no_std]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(unused_imports)]

extern crate alloc;

mod args;
mod cobra;
mod command;
mod shell_completions;

pub use args::*;
pub use cobra::*;
pub use command::*;
pub use shell_completions::*;
