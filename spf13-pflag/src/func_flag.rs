// go: file func.go decls: funcValue, funcValue.Set, funcValue.Type, funcValue.String, FlagSet.Func, FlagSet.FuncP, Func, FuncP
//
// func.go — a flag whose Set calls a callback instead of storing.

use crate::*;

// go: github.com/spf13/pflag@v1.0.10 func.go:4-4 funcValue
/// Go: `type funcValue func(string) error` — the callback IS the Value.
pub struct funcValue {
    f: alloc::boxed::Box<dyn Fn(string) -> error + Send + Sync>,
}

impl Value for funcValue {
    // go: none — Goish glue. Go copies the func value; a boxed closure
    // cannot be cloned, so this Value is not shareable across FlagSets.
    fn CloneBox(&self) -> alloc::boxed::Box<dyn Value> {
        panic!("pflag: a func flag's Value cannot be cloned into another FlagSet")
    }

    // go: github.com/spf13/pflag@v1.0.10 func.go:6-6 funcValue.Set
    fn Set_str(&mut self, s: string) -> error {
        return (self.f)(s);
    }

    // go: github.com/spf13/pflag@v1.0.10 func.go:8-8 funcValue.Type
    fn Type(&self) -> string {
        return string("func");
    }

    // go: github.com/spf13/pflag@v1.0.10 func.go:10-10 funcValue.String
    /// Go: "same behavior as stdlib 'flag' package" — always empty.
    fn String(&self) -> string {
        return string("");
    }
}

impl FlagSet {
    // go: github.com/spf13/pflag@v1.0.10 func.go:16-18 FlagSet.Func
    pub fn Func<F>(&mut self, name: string, usage: string, fn_: F)
    where F: Fn(string) -> error + Send + Sync + 'static {
        self.FuncP(name, string(""), usage, fn_);
    }

    // go: github.com/spf13/pflag@v1.0.10 func.go:21-24 FlagSet.FuncP
    pub fn FuncP<F>(&mut self, name: string, shorthand: string, usage: string, fn_: F)
    where F: Fn(string) -> error + Send + Sync + 'static {
        let val = alloc::boxed::Box::new(funcValue { f: alloc::boxed::Box::new(fn_) });
        self.VarP(val, name, shorthand, usage);
    }
}

// go: github.com/spf13/pflag@v1.0.10 func.go:30-32 Func
pub fn Func<F>(name: string, usage: string, fn_: F)
where F: Fn(string) -> error + Send + Sync + 'static {
    COMMAND_LINE.Lock().FuncP(name, string(""), usage, fn_);
}

// go: github.com/spf13/pflag@v1.0.10 func.go:35-37 FuncP
pub fn FuncP<F>(name: string, shorthand: string, usage: string, fn_: F)
where F: Fn(string) -> error + Send + Sync + 'static {
    COMMAND_LINE.Lock().FuncP(name, shorthand, usage, fn_);
}
