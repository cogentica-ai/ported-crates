// go: file errors.go decls: NotExistError.GetSpecifiedName, NotExistError.GetSpecifiedShortnames, ValueRequiredError.GetFlag, ValueRequiredError.GetSpecifiedName, ValueRequiredError.GetSpecifiedShortnames, InvalidValueError.GetFlag, InvalidValueError.GetValue, InvalidSyntaxError.GetSpecifiedFlag
//
// PARTIAL file: the four error types and their Error/Unwrap methods are
// in lib.rs; this adds the public accessors.
//
// DEVIATION on the two GetFlag methods. Go's errors hold `flag *Flag`,
// a pointer into the FlagSet, and hand it back whole. This port's
// errors carry the flag's NAME instead, because goish's FlagSet owns
// its flags per-set (the same reason AddFlag clones) and because a
// Flag holds a `Box<dyn Value>` that cannot always be cloned —
// funcValue's CloneBox deliberately panics. Returning the name keeps
// the accessor useful without inventing a lifetime the port cannot
// honour; callers needing the Flag can Lookup() it.

use crate::*;

impl NotExistError {
    // go: github.com/spf13/pflag@v1.0.10 errors.go:52-54 NotExistError.GetSpecifiedName
    pub fn GetSpecifiedName(&self) -> string {
        return self.name.clone();
    }

    // go: github.com/spf13/pflag@v1.0.10 errors.go:59-61 NotExistError.GetSpecifiedShortnames
    pub fn GetSpecifiedShortnames(&self) -> string {
        return self.specified_shorthands.clone();
    }
}

impl ValueRequiredError {
    // go: github.com/spf13/pflag@v1.0.10 errors.go:82-84 ValueRequiredError.GetFlag
    pub fn GetFlag(&self) -> string {
        return self.flag_name.clone();
    }

    // go: github.com/spf13/pflag@v1.0.10 errors.go:88-90 ValueRequiredError.GetSpecifiedName
    pub fn GetSpecifiedName(&self) -> string {
        return self.specified_name.clone();
    }

    // go: github.com/spf13/pflag@v1.0.10 errors.go:95-97 ValueRequiredError.GetSpecifiedShortnames
    pub fn GetSpecifiedShortnames(&self) -> string {
        return self.specified_shorthands.clone();
    }
}

impl InvalidValueError {
    // go: github.com/spf13/pflag@v1.0.10 errors.go:125-127 InvalidValueError.GetFlag
    pub fn GetFlag(&self) -> string {
        return self.flag_name.clone();
    }

    // go: github.com/spf13/pflag@v1.0.10 errors.go:130-132 InvalidValueError.GetValue
    pub fn GetValue(&self) -> string {
        return self.value.clone();
    }
}

impl InvalidSyntaxError {
    // go: github.com/spf13/pflag@v1.0.10 errors.go:147-149 InvalidSyntaxError.GetSpecifiedFlag
    pub fn GetSpecifiedFlag(&self) -> string {
        return self.specified_flag.clone();
    }
}
