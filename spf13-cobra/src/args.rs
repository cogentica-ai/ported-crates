// Port of github.com/spf13/cobra@v1.10.2 args.go

use goish::fmt;
use goish::strings;
use goish::errors::error;
use goish::string;
use goish::goslice::slice;
use goish::{append, make, nil, int};

use crate::command::Command;
use crate::cobra::stringInSlice;

// go: github.com/spf13/cobra@v1.10.2 args.go:22-22 PositionalArgs
pub type PositionalArgs =
    alloc::sync::Arc<dyn Fn(&mut Command, slice<string>) -> error + Send + Sync>;

// go: github.com/spf13/cobra@v1.10.2 args.go:28-39 legacyArgs
pub(crate) fn legacyArgs(cmd: &mut Command, args: slice<string>) -> error {
    // no subcommand, always take args
    if !cmd.HasSubCommands() {
        return nil.into();
    }

    // root command with subcommands, do subcommand checking.
    if !cmd.HasParent() && args.Len() > 0 {
        return fmt::Errorf!(
            "unknown command %q for %q%s",
            args[0usize].clone(),
            cmd.CommandPath(),
            cmd.findSuggestions(args[0usize].clone())
        );
    }
    nil.into()
}

// go: github.com/spf13/cobra@v1.10.2 args.go:42-47 NoArgs
pub fn NoArgs(cmd: &mut Command, args: slice<string>) -> error {
    if args.Len() > 0 {
        return fmt::Errorf!(
            "unknown command %q for %q",
            args[0usize].clone(),
            cmd.CommandPath()
        );
    }
    nil.into()
}

// go: github.com/spf13/cobra@v1.10.2 args.go:51-66 OnlyValidArgs
pub fn OnlyValidArgs(cmd: &mut Command, args: slice<string>) -> error {
    if cmd.ValidArgs.Len() > 0 {
        // Remove any description that may be included in ValidArgs.
        // A description is following a tab character.
        let mut validArgs: slice<string> = make!([]string, 0, cmd.ValidArgs.Len());
        for (_, v) in goish::range!(cmd.ValidArgs.clone()) {
            validArgs = append!(validArgs, strings::SplitN(v.clone(), "\t", 2)[0usize].clone());
        }
        for (_, v) in goish::range!(args.clone()) {
            if !stringInSlice(v.clone(), validArgs.clone()) {
                return fmt::Errorf!(
                    "invalid argument %q for %q%s",
                    v.clone(),
                    cmd.CommandPath(),
                    cmd.findSuggestions(args[0usize].clone())
                );
            }
        }
    }
    nil.into()
}

// go: github.com/spf13/cobra@v1.10.2 args.go:69-71 ArbitraryArgs
pub fn ArbitraryArgs(cmd: &mut Command, args: slice<string>) -> error {
    nil.into()
}

// go: github.com/spf13/cobra@v1.10.2 args.go:74-81 MinimumNArgs
pub fn MinimumNArgs(n: int) -> PositionalArgs {
    alloc::sync::Arc::new(move |cmd: &mut Command, args: slice<string>| -> error {
        if args.Len() < n {
            return fmt::Errorf!("requires at least %d arg(s), only received %d", n, args.Len());
        }
        nil.into()
    })
}

// go: github.com/spf13/cobra@v1.10.2 args.go:84-91 MaximumNArgs
pub fn MaximumNArgs(n: int) -> PositionalArgs {
    alloc::sync::Arc::new(move |cmd: &mut Command, args: slice<string>| -> error {
        if args.Len() > n {
            return fmt::Errorf!("accepts at most %d arg(s), received %d", n, args.Len());
        }
        nil.into()
    })
}

// go: github.com/spf13/cobra@v1.10.2 args.go:94-101 ExactArgs
pub fn ExactArgs(n: int) -> PositionalArgs {
    alloc::sync::Arc::new(move |cmd: &mut Command, args: slice<string>| -> error {
        if args.Len() != n {
            return fmt::Errorf!("accepts %d arg(s), received %d", n, args.Len());
        }
        nil.into()
    })
}

// go: github.com/spf13/cobra@v1.10.2 args.go:104-111 RangeArgs
pub fn RangeArgs(min: int, max: int) -> PositionalArgs {
    alloc::sync::Arc::new(move |cmd: &mut Command, args: slice<string>| -> error {
        if args.Len() < min || args.Len() > max {
            return fmt::Errorf!(
                "accepts between %d and %d arg(s), received %d",
                min,
                max,
                args.Len()
            );
        }
        nil.into()
    })
}

// go: github.com/spf13/cobra@v1.10.2 args.go:114-123 MatchAll
// KNOWN DIVERGENCE: Go is variadic (pargs ...PositionalArgs); the port takes
// a Vec of validators.
pub fn MatchAll(pargs: alloc::vec::Vec<PositionalArgs>) -> PositionalArgs {
    alloc::sync::Arc::new(move |cmd: &mut Command, args: slice<string>| -> error {
        for parg in pargs.iter() {
            let err = parg(cmd, args.clone());
            if err != nil {
                return err;
            }
        }
        nil.into()
    })
}

// go: github.com/spf13/cobra@v1.10.2 args.go:129-131 ExactValidArgs
// Deprecated: use MatchAll(ExactArgs(n), OnlyValidArgs) instead
pub fn ExactValidArgs(n: int) -> PositionalArgs {
    MatchAll(alloc::vec![
        ExactArgs(n),
        alloc::sync::Arc::new(OnlyValidArgs) as PositionalArgs
    ])
}
