// go: file golangflag.go decls: isGotestFlag, isGotestShorthandFlag
//
// PARTIAL file — and the reason is a dependency, not a decision.
//
// golangflag.go's other five declarations (wrapFlagValue,
// PFlagFromGoFlag, FlagSet.AddGoFlag, FlagSet.AddGoFlagSet,
// FlagSet.CopyToGoFlagSet, ParseSkippedFlags) bridge pflag to the Go
// standard library's `flag` package. Each one needs, from goish::flag:
//
//   * a `Value` interface (String/Set/Type) — goish::flag has none;
//   * a `Flag` STRUCT with exported Name/Usage/Value/DefValue —
//     goish::flag::Flag<T> is a generic value handle holding an
//     Arc<SpinLock<T>> and exposing only Get();
//   * FlagSet::Lookup and VisitAll returning those flags.
//
// Checked against goish-v1 src/flag/{mod,flag}.rs, whose own header
// records that its FlagSet and parser are hand-written rather than
// ported. Until goish::flag grows Go's shape these five cannot be
// written without inventing the type they are supposed to wrap, so
// they are left undone and named here rather than faked.

use crate::*;

// go: github.com/spf13/pflag@v1.0.10 golangflag.go:15-17 isGotestFlag
/// Go: "go test flags prefixes".
pub fn isGotestFlag(flag: string) -> bool {
    return strings::HasPrefix(flag, string("-test."));
}

// go: github.com/spf13/pflag@v1.0.10 golangflag.go:19-21 isGotestShorthandFlag
pub fn isGotestShorthandFlag(flag: string) -> bool {
    return strings::HasPrefix(flag, string("test."));
}
