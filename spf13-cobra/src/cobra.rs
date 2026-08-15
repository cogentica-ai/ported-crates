// Port of github.com/spf13/cobra@v1.10.2 cobra.go
//
// KNOWN DIVERGENCE: the text/template machinery (templateFuncs,
// AddTemplateFunc, AddTemplateFuncs, tmpl, tmplFunc) is not ported — goish
// has no text/template. Cobra v1.10 renders default usage/help/version via
// native functions (command.go), so only user-set custom templates are lost.
// Gt and Eq (both marked "FIXME … unused by cobra" upstream) are not ported —
// they exist only for template compatibility.

use goish::fmt;
use goish::os;
use goish::strings;
use goish::sync;
use goish::unicode;
use goish::io;
use goish::errors::error;
use goish::lazy::Lazy;
use goish::string;
use goish::goslice::slice;
use goish::{append, bytes, make, nil, int, byte};

use core::sync::atomic::{AtomicBool, Ordering};

// go: cobra.go:42-43
pub(crate) static initializers: Lazy<
    sync::Mutex<alloc::vec::Vec<alloc::sync::Arc<dyn Fn() + Send + Sync>>>,
> = Lazy::new(|| sync::Mutex::new(alloc::vec::Vec::new()));
pub(crate) static finalizers: Lazy<
    sync::Mutex<alloc::vec::Vec<alloc::sync::Arc<dyn Fn() + Send + Sync>>>,
> = Lazy::new(|| sync::Mutex::new(alloc::vec::Vec::new()));

// go: cobra.go:45-50
pub const defaultPrefixMatching: bool = false;
pub const defaultCommandSorting: bool = true;
pub const defaultCaseInsensitive: bool = false;
pub const defaultTraverseRunHooks: bool = false;

// go: cobra.go:55 — mutable package var lowered to an atomic (goish pattern
// for settable bool globals).
pub static EnablePrefixMatching: AtomicBool = AtomicBool::new(defaultPrefixMatching);

// go: github.com/spf13/cobra@v1.10.2 cobra.go:59-59 EnableCommandSorting
pub static EnableCommandSorting: AtomicBool = AtomicBool::new(defaultCommandSorting);

// go: github.com/spf13/cobra@v1.10.2 cobra.go:62-62 EnableCaseInsensitive
pub static EnableCaseInsensitive: AtomicBool = AtomicBool::new(defaultCaseInsensitive);

// go: github.com/spf13/cobra@v1.10.2 cobra.go:66-66 EnableTraverseRunHooks
pub static EnableTraverseRunHooks: AtomicBool = AtomicBool::new(defaultTraverseRunHooks);

// go: github.com/spf13/cobra@v1.10.2 cobra.go:99-101 OnInitialize
// KNOWN DIVERGENCE: Go is variadic (y ...func()); call once per function.
pub fn OnInitialize(y: alloc::sync::Arc<dyn Fn() + Send + Sync>) {
    initializers.Lock().push(y);
}

// go: github.com/spf13/cobra@v1.10.2 cobra.go:105-107 OnFinalize
// KNOWN DIVERGENCE: Go is variadic (y ...func()); call once per function.
pub fn OnFinalize(y: alloc::sync::Arc<dyn Fn() + Send + Sync>) {
    finalizers.Lock().push(y);
}

// go: github.com/spf13/cobra@v1.10.2 cobra.go:159-161 trimRightSpace
pub(crate) fn trimRightSpace<S: Into<string>>(s: S) -> string {
    let s = s.into();
    strings::TrimRightFunc(s, unicode::IsSpace)
}

// go: github.com/spf13/cobra@v1.10.2 cobra.go:166-171 appendIfNotPresent
pub(crate) fn appendIfNotPresent<S1: Into<string>, S2: Into<string>>(
    s: S1,
    stringToAppend: S2,
) -> string {
    let s = s.into();
    let stringToAppend = stringToAppend.into();
    if strings::Contains(s.clone(), stringToAppend.clone()) {
        return s;
    }
    (s) + (" ") + (stringToAppend)
}

// go: github.com/spf13/cobra@v1.10.2 cobra.go:174-177 rpad
pub(crate) fn rpad<S: Into<string>>(s: S, padding: int) -> string {
    let s = s.into();
    let formattedString = fmt::Sprintf!("%%-%ds", padding);
    fmt::Sprintf!(formattedString, s)
}

// go: cobra.go:192 (ld) — levenshtein distance
pub(crate) fn ld<S1: Into<string>, S2: Into<string>>(s: S1, t: S2, ignoreCase: bool) -> int {
    let mut s = s.into();
    let mut t = t.into();
    if ignoreCase {
        s = strings::ToLower(s);
        t = strings::ToLower(t);
    }
    let mut d: slice<slice<int>> = make!([] slice<int>, s.Len() + 1);
    for i in 0..d.Len() {
        d[i] = make!([]int, t.Len() + 1);
        d[i][0] = i;
    }
    {
        let cols = d[0].Len();
        for j in 0..cols {
            d[0][j] = j;
        }
    }
    let mut j: int = 1;
    while j <= t.Len() {
        let mut i: int = 1;
        while i <= s.Len() {
            if s[i - 1] == t[j - 1] {
                let v = d[i - 1][j - 1];
                d[i][j] = v;
            } else {
                let mut min = d[i - 1][j];
                if d[i][j - 1] < min {
                    min = d[i][j - 1];
                }
                if d[i - 1][j - 1] < min {
                    min = d[i - 1][j - 1];
                }
                d[i][j] = min + 1;
            }
            i += 1;
        }
        j += 1;
    }
    d[s.Len()][t.Len()]
}

// go: github.com/spf13/cobra@v1.10.2 cobra.go:225-232 stringInSlice
pub(crate) fn stringInSlice<S: Into<string>>(a: S, list: slice<string>) -> bool {
    let a = a.into();
    for (_, b) in goish::range!(list) {
        if *b == a {
            return true;
        }
    }
    false
}

// go: github.com/spf13/cobra@v1.10.2 cobra.go:235-240 CheckErr
// KNOWN DIVERGENCE: Go takes interface{}; the port takes error (the only
// type cobra itself ever passes).
pub fn CheckErr(msg: error) {
    if msg != nil {
        let mut e = os::Stderr();
        fmt::Fprintln!(e, "Error:", msg);
        os::Exit(1);
    }
}

// go: github.com/spf13/cobra@v1.10.2 cobra.go:243-246 WriteStringAndCheck
// KNOWN DIVERGENCE: io.StringWriter narrowed to io::Writer (goish strings
// and buffers implement Writer; write errors still route through CheckErr).
pub fn WriteStringAndCheck<S: Into<string>>(b: &mut dyn io::Writer, s: S) {
    let s = s.into();
    let (_, err) = b.Write(bytes(s));
    CheckErr(err);
}
