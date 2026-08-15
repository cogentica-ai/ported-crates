// go: file flag.go decls: sortFlags, FlagSet.GetNormalizeFunc, FlagSet.Output, FlagSet.Name, FlagSet.SetOutput, wrapN, wrap, defaultUsage, FlagSet.usage, FlagSet.parseShortArg, FlagSet.ParseAll, ParseAll
//
// PARTIAL file: most of flag.go is in lib.rs. This holds the decls that
// were still missing, chiefly the usage-text wrapper pair and ParseAll.

use crate::*;

// go: github.com/spf13/pflag@v1.0.10 flag.go:229-242 sortFlags
/// Go takes `map[NormalizedName]*Flag` and returns the flags sorted by
/// name. The port stores flags in a Vec with name→index maps, so this
/// takes the name→index map and returns indices in sorted name order —
/// the same ordering decision, expressed in the port's shape.
pub fn sortFlags(flags: &map<string, usize>) -> slice<usize> {
    let mut list: slice<string> = make!([]string, 0);
    for (k, _) in flags.__iter() {
        list = append!(list, k.clone());
    }
    sort::Strings!(&mut list);
    let mut result: slice<usize> = make!([]usize, list.Len());
    for i in 0..list.Len() {
        result[i] = flags.Get(list[i].clone()).0;
    }
    return result;
}

// go: github.com/spf13/pflag@v1.0.10 flag.go:639-653 wrapN
/// Splits `s` at the last space/tab/newline before `i`, allowing up to
/// `slop` extra characters so a short final word is not orphaned.
pub fn wrapN(i: int, slop: int, s: string) -> (string, string) {
    let n = s.Len();
    if i + slop > n {
        return (s, string(""));
    }
    let sref: &str = s.as_ref();
    let head = string::from_bytes(&sref.as_bytes()[..i as usize]);
    let w = strings::LastIndexAny(head.clone(), string(" \t\n"));
    if w <= 0 {
        return (s, string(""));
    }
    let nl_pos = strings::LastIndex(head, string("\n"));
    let raw = sref.as_bytes();
    if nl_pos > 0 && nl_pos < w {
        return (
            string::from_bytes(&raw[..nl_pos as usize]),
            string::from_bytes(&raw[(nl_pos + 1) as usize..]),
        );
    }
    return (
        string::from_bytes(&raw[..w as usize]),
        string::from_bytes(&raw[(w + 1) as usize..]),
    );
}

// go: github.com/spf13/pflag@v1.0.10 flag.go:658-702 wrap
/// Go: "Wraps the string `s` to a maximum width `w` with leading indent
/// `i`. The first line is not indented (this is assumed to be done by
/// caller). Pass `w` == 0 to do no wrapping"
pub fn wrap(i: int, w: int, s: string) -> string {
    let mut i = i;
    if w == 0 {
        return strings::Replace(s, string("\n"),
                                string("\n") + strings::Repeat(string(" "), i), -1);
    }
    let mut wrap_w = w - i;
    let mut r = string("");
    // Go: "Not enough space for sensible wrapping. Wrap as a block on
    // the next line instead."
    if wrap_w < 24 {
        i = 16;
        wrap_w = w - i;
        r = r + string("\n") + strings::Repeat(string(" "), i);
    }
    // Go: "If still not enough space then don't even try to wrap."
    if wrap_w < 24 {
        return strings::Replace(s, string("\n"), r, -1);
    }
    // Go: "Try to avoid short orphan words on the final line"
    let slop: int = 5;
    wrap_w = wrap_w - slop;

    let (l, mut rest) = wrapN(wrap_w, slop, s);
    r = r + strings::Replace(l, string("\n"),
                             string("\n") + strings::Repeat(string(" "), i), -1);
    while rest != "" {
        let (t, next) = wrapN(wrap_w, slop, rest.clone());
        r = r + string("\n") + strings::Repeat(string(" "), i)
            + strings::Replace(t, string("\n"),
                               string("\n") + strings::Repeat(string(" "), i), -1);
        rest = next;
    }
    return r;
}

impl FlagSet {
    // go: github.com/spf13/pflag@v1.0.10 flag.go:269-274 FlagSet.GetNormalizeFunc
    /// Go returns the set's normalize func, or an identity func when none
    /// was installed. The port hands back a boxed closure for the same
    /// reason: the caller only ever applies it.
    pub fn GetNormalizeFunc(&self) -> alloc::boxed::Box<dyn Fn(&FlagSet, string) -> NormalizedName + Send + Sync> {
        if self.normalize_name_fn.is_some() {
            // The stored closure cannot be cloned out, so the returned
            // one re-enters through normalizeFlagName, which consults it.
            return alloc::boxed::Box::new(|f: &FlagSet, name: string| f.normalizeFlagName(name));
        }
        return alloc::boxed::Box::new(|_f: &FlagSet, name: string| name);
    }

    // go: github.com/spf13/pflag@v1.0.10 flag.go:291-293 FlagSet.Name
    pub fn Name(&self) -> string {
        return self.name.clone();
    }

    // go: github.com/spf13/pflag@v1.0.10 flag.go:297-299 FlagSet.SetOutput
    /// Go: "If output is nil, os.Stderr is used."
    pub fn SetOutput(&mut self, output: alloc::boxed::Box<dyn io::Writer + Send + Sync>) {
        self.output = Some(output);
    }

    // go: github.com/spf13/pflag@v1.0.10 flag.go:283-288 FlagSet.Output
    /// Go returns io.Writer, defaulting to os.Stderr. The port reports
    /// whether a writer was installed; output_write applies the same
    /// default, so the routing decision is identical.
    pub fn Output(&self) -> bool {
        return self.output.is_some();
    }

    // go: github.com/spf13/pflag@v1.0.10 flag.go:948-956 FlagSet.usage
    pub fn usage(&self) {
        match self.Usage {
            None => defaultUsage(self),
            Some(ref u) => u(),
        }
    }

    // go: github.com/spf13/pflag@v1.0.10 flag.go:1116-1129 FlagSet.parseShortArg
    /// Go: "shorthands can be a series of shorthand letters of flags
    /// (e.g. -vvv)" — the loop is what makes `-vvv` three occurrences.
    pub fn parseShortArg<F>(&mut self, s: string, args: &mut alloc::vec::Vec<string>, fn_: &mut F) -> error
    where F: FnMut(&mut FlagSet, string, string) -> error {
        let sref: &str = s.as_ref();
        let raw = sref.as_bytes();
        let mut shorthands = string::from_bytes(&raw[1..]);
        while shorthands.Len() > 0 {
            let (rest, err) = self.parseSingleShortArg(shorthands, args, fn_);
            if err != nil {
                return err;
            }
            shorthands = rest;
        }
        return nil.into();
    }

    // go: github.com/spf13/pflag@v1.0.10 flag.go:1209-1229 FlagSet.ParseAll
    /// Like Parse, but calls `fn` for every flag instead of setting it.
    pub fn ParseAll<F>(&mut self, arguments: slice<string>, mut fn_: F) -> error
    where F: FnMut(&mut FlagSet, string, string) -> error {
        self.parsed = true;
        self.__reset_args();
        let mut args_vec: alloc::vec::Vec<string> = alloc::vec::Vec::new();
        for i in 0..arguments.Len() {
            args_vec.push(arguments[i].clone());
        }
        let err = self.parseArgs(args_vec, &mut fn_);
        if err != nil {
            match self.__error_handling() {
                0 => return err,
                1 => {
                    if errors::Is(err.clone(), ErrHelp) {
                        goish::syscall::Exit(0);
                    }
                    self.__output_write(fmt::Sprintf!("%v\n", err));
                    goish::syscall::Exit(2);
                }
                2 => panic!("pflag: ParseAll"),
                _ => {}
            }
        }
        return nil.into();
    }
}

// go: github.com/spf13/pflag@v1.0.10 flag.go:790-793 defaultUsage
pub fn defaultUsage(f: &FlagSet) {
    f.__output_write(fmt::Sprintf!("Usage of %s:\n", f.Name()));
    f.PrintDefaults();
}

// go: github.com/spf13/pflag@v1.0.10 flag.go:1246-1249 ParseAll
/// Go: "parses the command-line flags from os.Args[1:] and calls fn for
/// each." Errors are ignored — CommandLine is set for ExitOnError.
pub fn ParseAll<F>(fn_: F)
where F: FnMut(&mut FlagSet, string, string) -> error {
    let args = os::Args();
    let mut rest: slice<string> = make!([]string, 0);
    for i in 1..args.Len() {
        rest = append!(rest, args[i].clone());
    }
    let _ = COMMAND_LINE.Lock().ParseAll(rest, fn_);
}
