// Port of github.com/spf13/cobra@v1.10.2 shell_completions.go (partial)
//
// Only the required-flag marking family is ported — command.go's
// ValidateRequiredFlags depends on the annotation. The completion
// generators (MarkFlagFilename, MarkFlagCustom, MarkFlagDirname, and the
// per-shell generators) are not ported (KNOWN DIVERGENCE).

use goish::errors::error;
use goish::string;
use goish::slice;

use spf13_pflag as flag;

use crate::command::Command;

// go: bash_completions.go:32 (BashCompOneRequiredFlag)
pub const BashCompOneRequiredFlag: &str = "cobra_annotation_bash_completion_one_required_flag";

impl Command {
    // go: shell_completions.go:24 (MarkFlagRequired)
    pub fn MarkFlagRequired<S: Into<string>>(&mut self, name: S) -> error {
        MarkFlagRequired(self.Flags(), name)
    }

    // go: shell_completions.go:31 (MarkPersistentFlagRequired)
    pub fn MarkPersistentFlagRequired<S: Into<string>>(&mut self, name: S) -> error {
        MarkFlagRequired(self.PersistentFlags(), name)
    }
}

// go: shell_completions.go:38 (MarkFlagRequired)
pub fn MarkFlagRequired<S: Into<string>>(flags: &mut flag::FlagSet, name: S) -> error {
    flags.SetAnnotation(
        name,
        BashCompOneRequiredFlag,
        slice!([]string{"true"}),
    )
}
