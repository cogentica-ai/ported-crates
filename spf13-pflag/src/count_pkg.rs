// go: file count.go decls: newCountValue, countConv, CountVar, CountVarP, Count, CountP
//
// PARTIAL file: count.go's Value impl and FlagSet methods are in lib.rs.
// Note Count takes no default value — the flag counts occurrences.

use crate::*;

// go: github.com/spf13/pflag@v1.0.10 count.go:8-11 newCountValue
/// Stores `val`, as Go does. Deliberately NOT delegating to lib.rs's
/// `countValue::new`, which hardcodes 0 and drops its argument: that is
/// invisible to pflag's own callers (FlagSet::CountVar always passes 0)
/// but would silently zero a caller-supplied starting count here.
pub fn newCountValue(val: int, p: *mut int) -> countValue {
    unsafe {
        *p = val;
    }
    return countValue { ptr: p };
}

// go: github.com/spf13/pflag@v1.0.10 count.go:30-36 countConv
pub fn countConv(sval: string) -> (goish::goany::Any, error) {
    let (i, err) = strconv::Atoi(sval);
    if err != nil {
        return (goish::goany::Any::from(nil), err);
    }
    return (goish::goany::Any::new(i), nil.into());
}

// go: github.com/spf13/pflag@v1.0.10 count.go:61-63 CountVar
pub fn CountVar(p: *mut int, name: string, usage: string) {
    COMMAND_LINE.Lock().CountVarP(p, name, string(""), usage);
}

// go: github.com/spf13/pflag@v1.0.10 count.go:66-68 CountVarP
pub fn CountVarP(p: *mut int, name: string, shorthand: string, usage: string) {
    COMMAND_LINE.Lock().CountVarP(p, name, shorthand, usage);
}

// go: github.com/spf13/pflag@v1.0.10 count.go:89-91 Count
pub fn Count(name: string, usage: string) -> *mut int {
    return COMMAND_LINE.Lock().CountP(name, string(""), usage);
}

// go: github.com/spf13/pflag@v1.0.10 count.go:94-96 CountP
pub fn CountP(name: string, shorthand: string, usage: string) -> *mut int {
    return COMMAND_LINE.Lock().CountP(name, shorthand, usage);
}
