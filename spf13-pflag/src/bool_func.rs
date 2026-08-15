// go: file bool_func.go decls: boolfuncValue, boolfuncValue.Set, boolfuncValue.Type, boolfuncValue.String, boolfuncValue.IsBoolFlag, FlagSet.BoolFunc, FlagSet.BoolFuncP, BoolFunc, BoolFuncP
//
// bool_func.go — like func.go, but the flag takes no value: BoolFuncP
// sets NoOptDefVal so `--name` alone fires the callback.

use crate::*;

// go: github.com/spf13/pflag@v1.0.10 bool_func.go:4-4 boolfuncValue
pub struct boolfuncValue {
    f: alloc::boxed::Box<dyn Fn(string) -> error + Send + Sync>,
}

impl boolfuncValue {
    // go: github.com/spf13/pflag@v1.0.10 bool_func.go:12-12 boolfuncValue.IsBoolFlag
    pub fn IsBoolFlag(&self) -> bool {
        return true;
    }
}

impl Value for boolfuncValue {
    // go: none — Goish glue; see funcValue::CloneBox.
    fn CloneBox(&self) -> alloc::boxed::Box<dyn Value> {
        panic!("pflag: a boolfunc flag's Value cannot be cloned into another FlagSet")
    }

    // go: github.com/spf13/pflag@v1.0.10 bool_func.go:6-6 boolfuncValue.Set
    fn Set_str(&mut self, s: string) -> error {
        return (self.f)(s);
    }

    // go: github.com/spf13/pflag@v1.0.10 bool_func.go:8-8 boolfuncValue.Type
    fn Type(&self) -> string {
        return string("boolfunc");
    }

    // go: github.com/spf13/pflag@v1.0.10 bool_func.go:10-10 boolfuncValue.String
    fn String(&self) -> string {
        return string("");
    }
}

impl FlagSet {
    // go: github.com/spf13/pflag@v1.0.10 bool_func.go:18-20 FlagSet.BoolFunc
    pub fn BoolFunc<F>(&mut self, name: string, usage: string, fn_: F)
    where F: Fn(string) -> error + Send + Sync + 'static {
        self.BoolFuncP(name, string(""), usage, fn_);
    }

    // go: github.com/spf13/pflag@v1.0.10 bool_func.go:23-27 FlagSet.BoolFuncP
    /// Go takes VarPF's *Flag and sets NoOptDefVal = "true"; the port's
    /// VarPF hands back the flag's index, so the write goes through that.
    /// Without it `--name` with no value would be a parse error.
    pub fn BoolFuncP<F>(&mut self, name: string, shorthand: string, usage: string, fn_: F)
    where F: Fn(string) -> error + Send + Sync + 'static {
        let val = alloc::boxed::Box::new(boolfuncValue { f: alloc::boxed::Box::new(fn_) });
        let idx = self.VarPF(val, name, shorthand, usage);
        self.flags[idx].NoOptDefVal = string("true");
    }
}

// go: github.com/spf13/pflag@v1.0.10 bool_func.go:33-35 BoolFunc
pub fn BoolFunc<F>(name: string, usage: string, fn_: F)
where F: Fn(string) -> error + Send + Sync + 'static {
    COMMAND_LINE.Lock().BoolFuncP(name, string(""), usage, fn_);
}

// go: github.com/spf13/pflag@v1.0.10 bool_func.go:38-40 BoolFuncP
pub fn BoolFuncP<F>(name: string, shorthand: string, usage: string, fn_: F)
where F: Fn(string) -> error + Send + Sync + 'static {
    COMMAND_LINE.Lock().BoolFuncP(name, shorthand, usage, fn_);
}
