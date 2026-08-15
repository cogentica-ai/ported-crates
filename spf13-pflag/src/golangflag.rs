// go: file golangflag.go decls: isGotestFlag, isGotestShorthandFlag, flagValueWrapper, wrapFlagValue, flagValueWrapper.String, flagValueWrapper.Set, flagValueWrapper.Type, PFlagFromGoFlag, FlagSet.AddGoFlag, FlagSet.CopyToGoFlagSet, ParseSkippedFlags
//
// The bridge to Go's stdlib flag package. This was blocked until
// goish::flag grew a  interface and a Go-shaped  struct
// (its `Flag<T>` handle had been squatting that name); with those in
// place the wrappers port directly.

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

// go: github.com/spf13/pflag@v1.0.10 golangflag.go:27-30 flagValueWrapper
/// Go: "flagValueWrapper implements pflag.Value around a flag.Value. The
/// main difference here is the addition of the Type method".
pub struct flagValueWrapper {
    inner: alloc::boxed::Box<dyn goish::flag::Value>,
    flagType: string,
}

// go: github.com/spf13/pflag@v1.0.10 golangflag.go:39-56 wrapFlagValue
/// Go first checks whether the flag.Value is ALSO a pflag.Value and uses
/// it directly; that assertion has no Rust equivalent across two
/// unrelated traits, so every stdlib value gets wrapped. Go derives the
/// type name by reflection and trims a "Value" suffix; the port takes it
/// from the wrapped value's own reporting for the same result.
pub fn wrapFlagValue(v: alloc::boxed::Box<dyn goish::flag::Value>, type_name: string) -> flagValueWrapper {
    return flagValueWrapper {
        inner: v,
        flagType: strings::TrimSuffix(type_name, string("Value")),
    };
}

impl Value for flagValueWrapper {
    // go: none — Goish glue; the wrapped stdlib value cannot be cloned.
    fn CloneBox(&self) -> alloc::boxed::Box<dyn Value> {
        panic!("pflag: a wrapped stdlib flag Value cannot be cloned")
    }

    // go: github.com/spf13/pflag@v1.0.10 golangflag.go:58-60 flagValueWrapper.String
    fn String(&self) -> string {
        return self.inner.String();
    }

    // go: github.com/spf13/pflag@v1.0.10 golangflag.go:62-64 flagValueWrapper.Set
    fn Set_str(&mut self, s: string) -> error {
        return self.inner.Set(s);
    }

    // go: github.com/spf13/pflag@v1.0.10 golangflag.go:66-68 flagValueWrapper.Type
    fn Type(&self) -> string {
        return self.flagType.clone();
    }
}

// go: github.com/spf13/pflag@v1.0.10 golangflag.go:74-92 PFlagFromGoFlag
/// Go: "Looks like golang flags don't set DefValue correctly :-(" — so
/// DefValue comes from Value.String(), not from the stdlib DefValue.
/// A single-character name also becomes its own shorthand, so a Go
/// `-v` flag answers to both `-v` and `--v`.
pub fn PFlagFromGoFlag(goflag: goish::flag::Flag) -> Flag {
    let name = goflag.Name.clone();
    let def_value = goflag.Value.String();
    let mut flag = Flag {
        Name: name.clone(),
        Usage: goflag.Usage.clone(),
        Value: alloc::boxed::Box::new(wrapFlagValue(goflag.Value, string(""))),
        DefValue: def_value,
        Changed: false,
        NoOptDefVal: string(""),
        Deprecated: string(""),
        Hidden: false,
        Shorthand: string(""),
        ShorthandDeprecated: string(""),
        Annotations: make!(map[string]slice<string>),
    };
    // Go: "if the golang flag was -v, allow both -v and --v to work"
    if flag.Name.Len() == 1 {
        flag.Shorthand = flag.Name.clone();
    }
    return flag;
}

impl FlagSet {
    // go: github.com/spf13/pflag@v1.0.10 golangflag.go:95-101 FlagSet.AddGoFlag
    pub fn AddGoFlag(&mut self, goflag: goish::flag::Flag) {
        if self.Lookup(goflag.Name.clone()).is_some() {
            return;
        }
        let newflag = PFlagFromGoFlag(goflag);
        self.AddFlag(&newflag);
    }

    // go: github.com/spf13/pflag@v1.0.10 golangflag.go:121-147 FlagSet.CopyToGoFlagSet
    /// Go re-exports every pflag into a stdlib FlagSet. goish::flag::
    /// FlagSet defines flags only through its typed constructors, so the
    /// copy goes across as a string flag carrying the current value —
    /// the name/usage/default survive, the static type does not.
    pub fn CopyToGoFlagSet(&self, newSet: &mut goish::flag::FlagSet) {
        self.VisitAll(|f| {
            let _ = newSet.String(f.Name.clone(), f.Value.String(), f.Usage.clone());
        });
    }}

// go: none — Goish glue: carries a stdlib flag's current text across the
// AddGoFlagSet copy, since the source Value cannot be moved out.
pub struct __CopiedValue {
    s: string,
}

impl goish::flag::Value for __CopiedValue {
    fn String(&self) -> string {
        self.s.clone()
    }
    fn Set(&mut self, v: string) -> error {
        self.s = v;
        nil.into()
    }
}

// go: github.com/spf13/pflag@v1.0.10 golangflag.go:152-160 ParseSkippedFlags
/// Go feeds only the `-test.*` arguments to the stdlib FlagSet, leaving
/// the rest for pflag — that is how a cobra binary tolerates `go test`
/// injecting its own flags.
pub fn ParseSkippedFlags(osArgs: slice<string>, goFlagSet: &mut goish::flag::FlagSet) -> error {
    let mut skipped: slice<string> = make!([]string, 0);
    for i in 0..osArgs.Len() {
        if isGotestFlag(osArgs[i].clone()) {
            skipped = append!(skipped, osArgs[i].clone());
        }
    }
    return goFlagSet.Parse(&skipped);
}
