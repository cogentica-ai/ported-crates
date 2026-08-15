// Port of github.com/spf13/cobra@v1.10.2 command.go
//
// Ownership model (// go: none — structural glue):
//   Go's Command tree is a graph of *Command pointers. The port stores
//   children as Box<Command> in the parent's `commands` Vec (stable heap
//   addresses, like the pflag port's Flag arena) and keeps Go's `parent`
//   back-pointer as a raw *mut Command. ExecuteC() re-anchors all parent
//   pointers before running (relinkParents), so the root Command may be
//   moved freely between AddCommand and Execute.
//
// KNOWN DIVERGENCE (whole-file level):
//   * context.Context plumbing (ctx, Context, SetContext, ExecuteContext,
//     ExecuteContextC) is not ported.
//   * Shell completion (ValidArgsFunction, initCompleteCmd,
//     InitDefaultCompletionCmd, CompletionOptions) is not ported.
//   * Custom usage/help/version templates (SetUsageTemplate etc.) are not
//     ported; the native default renderers below are.
//   * flag groups (ValidateFlagGroups) are not ported.
//   * globNormFunc / SetGlobalNormalizationFunc are not ported.
//   * flagErrorBuf is not ported — the ported pflag writes warnings to
//     stderr directly (it has no SetOutput).
//   * SetOut/SetErr take a shareable bytes.Buffer rather than an arbitrary
//     io.Writer (goish interface values cannot yet express a settable
//     mutable writer slot); the stdout/stderr default paths are complete.
//   * AddCommand/AddGroup take one element per call (Go is variadic).
//   * RemoveCommand, ResetCommands, ResetFlags, DebugFlags, SetIn,
//     InOrStdin, Flag, persistentFlag, SetHelpCommand,
//     SetCompletionCommandGroupID are not ported.

use goish::fmt;
use goish::os;
use goish::path::filepath;
use goish::strings;
use goish::sync;
use goish::bytes;
use goish::io;
use goish::errors;
use goish::errors::error;
use goish::string;
use goish::slice;
use goish::gomap::map;
use goish::{append, make, nil, int, byte};

use core::sync::atomic::Ordering;

use spf13_pflag as flag;

use crate::args::{legacyArgs, ArbitraryArgs, PositionalArgs};
use crate::cobra::{
    finalizers, initializers, ld, rpad, trimRightSpace, CheckErr, EnableCaseInsensitive,
    EnableCommandSorting, EnablePrefixMatching, EnableTraverseRunHooks,
};
use crate::shell_completions::BashCompOneRequiredFlag;

// go: command.go:34-38
pub const FlagSetByCobraAnnotation: &str = "cobra_annotation_flag_set_by_cobra";
pub const CommandDisplayNameAnnotation: &str = "cobra_annotation_command_display_name";

pub(crate) const helpFlagName: &str = "help";
pub(crate) const helpCommandName: &str = "help";

// go: completions.go:136 (type Completion = string) — only the alias is
// ported; completion machinery is not.
pub type Completion = string;

// go: command.go:42 (FParseErrWhitelist)
pub type FParseErrWhitelist = flag::ParseErrorsAllowlist;

// go: command.go:45 (Group)
#[derive(Clone, Default)]
pub struct Group {
    pub ID: string,
    pub Title: string,
}

// go: none — closure field types for the *Run family and the user-settable
// usage/help/flag-error functions (Go func fields lower to Arc<dyn Fn>).
pub type RunFn = alloc::sync::Arc<dyn Fn(&mut Command, slice<string>) + Send + Sync>;
pub type RunEFn = alloc::sync::Arc<dyn Fn(&mut Command, slice<string>) -> error + Send + Sync>;
pub type UsageFn = alloc::sync::Arc<dyn Fn(&mut Command) -> error + Send + Sync>;
pub type HelpFn = alloc::sync::Arc<dyn Fn(&mut Command, slice<string>) + Send + Sync>;
pub type FlagErrorFn = alloc::sync::Arc<dyn Fn(&mut Command, error) -> error + Send + Sync>;

// go: none — shareable writer slot for SetOut/SetErr/UsageString (mirrors
// Go's habit of passing *bytes.Buffer as io.Writer; see goish io::Writer's
// Arc<Mutex<W>> blanket impl).
pub type SharedBuf = alloc::sync::Arc<sync::Mutex<bytes::Buffer>>;

// go: none — the io.Writer value produced by OutOrStdout/OutOrStderr/
// ErrOrStderr: either a real fd or the shared override buffer.
pub enum cmdWriter {
    Std(os::File),
    Buf(SharedBuf),
}

impl io::Writer for cmdWriter {
    fn Write(&mut self, p: slice<byte>) -> (int, error) {
        match self {
            cmdWriter::Std(f) => f.Write(p),
            cmdWriter::Buf(b) => b.Lock().Write(p),
        }
    }
}

// go: command.go:213 (commandCalledAs)
#[derive(Clone, Default)]
pub struct commandCalledAs {
    pub name: string,
    pub called: bool,
}

// go: command.go:54 (Command) — field order preserved; unported fields are
// listed in the file-header divergence note.
pub struct Command {
    // Use is the one-line usage message.
    pub Use: string,
    // Aliases is an array of aliases that can be used instead of the first word in Use.
    pub Aliases: slice<string>,
    // SuggestFor is an array of command names for which this command will be suggested.
    pub SuggestFor: slice<string>,
    // Short is the short description shown in the 'help' output.
    pub Short: string,
    // The group id under which this subcommand is grouped in the 'help' output of its parent.
    pub GroupID: string,
    // Long is the long message shown in the 'help <this-command>' output.
    pub Long: string,
    // Example is examples of how to use the command.
    pub Example: string,
    // ValidArgs is list of all valid non-flag arguments that are accepted in shell completions
    pub ValidArgs: slice<Completion>,
    // Expected arguments
    pub Args: Option<PositionalArgs>,
    // ArgAliases is List of aliases for ValidArgs.
    pub ArgAliases: slice<string>,
    // Deprecated defines, if this command is deprecated and should print this string when used.
    pub Deprecated: string,
    // Annotations are key/value pairs that can be used by applications.
    pub Annotations: map<string, string>,
    // Version defines the version for this command.
    pub Version: string,

    // PersistentPreRun: children of this command will inherit and execute.
    pub PersistentPreRun: Option<RunFn>,
    // PersistentPreRunE: PersistentPreRun but returns an error.
    pub PersistentPreRunE: Option<RunEFn>,
    // PreRun: children of this command will not inherit.
    pub PreRun: Option<RunFn>,
    // PreRunE: PreRun but returns an error.
    pub PreRunE: Option<RunEFn>,
    // Run: Typically the actual work function. Most commands will only implement this.
    pub Run: Option<RunFn>,
    // RunE: Run but returns an error.
    pub RunE: Option<RunEFn>,
    // PostRun: run after the Run command.
    pub PostRun: Option<RunFn>,
    // PostRunE: PostRun but returns an error.
    pub PostRunE: Option<RunEFn>,
    // PersistentPostRun: children of this command will inherit and execute after PostRun.
    pub PersistentPostRun: Option<RunFn>,
    // PersistentPostRunE: PersistentPostRun but returns an error.
    pub PersistentPostRunE: Option<RunEFn>,

    // groups for subcommands
    pub commandgroups: slice<Group>,

    // args is actual args parsed from flags.
    pub args: slice<string>,
    // flags is full set of flags.
    pub flags: Option<alloc::boxed::Box<flag::FlagSet>>,
    // pflags contains persistent flags.
    pub pflags: Option<alloc::boxed::Box<flag::FlagSet>>,
    // lflags contains local flags (cache).
    pub lflags: Option<alloc::boxed::Box<flag::FlagSet>>,
    // iflags contains inherited flags (cache).
    pub iflags: Option<alloc::boxed::Box<flag::FlagSet>>,
    // parentsPflags is all persistent flags of cmd's parents.
    pub parentsPflags: Option<alloc::boxed::Box<flag::FlagSet>>,

    // usageFunc is usage func defined by user.
    pub usageFunc: Option<UsageFn>,
    // flagErrorFunc is func defined by user, called when flag parsing fails.
    pub flagErrorFunc: Option<FlagErrorFn>,
    // helpFunc is help func defined by user.
    pub helpFunc: Option<HelpFn>,
    // helpCommand is command with usage 'help'. If it's not defined by user,
    // cobra uses default help command. (Points into `commands`.)
    pub helpCommand: *mut Command,
    // helpCommandGroupID is the group id for the helpCommand
    pub helpCommandGroupID: string,

    // errPrefix is the error message prefix defined by user.
    pub errPrefix: string,

    // outWriter is a writer defined by the user that replaces stdout
    pub outWriter: Option<SharedBuf>,
    // errWriter is a writer defined by the user that replaces stderr
    pub errWriter: Option<SharedBuf>,

    // FParseErrWhitelist flag parse errors to be ignored
    pub FParseErrWhitelist: FParseErrWhitelist,

    // commandsAreSorted defines, if command slice are sorted or not.
    pub commandsAreSorted: bool,
    // commandCalledAs is the name or alias value used to call this command.
    pub commandCalledAs: commandCalledAs,

    // commands is the list of commands supported by this program.
    pub commands: alloc::vec::Vec<alloc::boxed::Box<Command>>,
    // parent is a parent command for this command.
    pub parent: *mut Command,
    // Max lengths of commands' string lengths for use in padding.
    pub commandsMaxUseLen: int,
    pub commandsMaxCommandPathLen: int,
    pub commandsMaxNameLen: int,

    // TraverseChildren parses flags on all parents before executing child command.
    pub TraverseChildren: bool,
    // Hidden defines, if this command is hidden and should NOT show up in the list of available commands.
    pub Hidden: bool,
    // SilenceErrors is an option to quiet errors down stream.
    pub SilenceErrors: bool,
    // SilenceUsage is an option to silence usage when an error occurs.
    pub SilenceUsage: bool,
    // DisableFlagParsing disables the flag parsing.
    pub DisableFlagParsing: bool,
    // DisableAutoGenTag defines, if gen tag will be printed by generating docs.
    pub DisableAutoGenTag: bool,
    // DisableFlagsInUseLine will disable the addition of [flags] to the usage line.
    pub DisableFlagsInUseLine: bool,
    // DisableSuggestions disables the suggestions based on Levenshtein distance.
    pub DisableSuggestions: bool,
    // SuggestionsMinimumDistance defines minimum levenshtein distance to display suggestions.
    pub SuggestionsMinimumDistance: int,
}

// go: none — raw parent pointers make Command !Send/!Sync by default; every
// closure field is Send+Sync and the tree is only ever driven from one
// goroutine at a time (as in Go).
unsafe impl Send for Command {}
unsafe impl Sync for Command {}

// go: none — Go's zero value for Command{}.
impl Default for Command {
    fn default() -> Self {
        Command {
            Use: Default::default(),
            Aliases: Default::default(),
            SuggestFor: Default::default(),
            Short: Default::default(),
            GroupID: Default::default(),
            Long: Default::default(),
            Example: Default::default(),
            ValidArgs: Default::default(),
            Args: None,
            ArgAliases: Default::default(),
            Deprecated: Default::default(),
            Annotations: make!(map[string]string),
            Version: Default::default(),
            PersistentPreRun: None,
            PersistentPreRunE: None,
            PreRun: None,
            PreRunE: None,
            Run: None,
            RunE: None,
            PostRun: None,
            PostRunE: None,
            PersistentPostRun: None,
            PersistentPostRunE: None,
            commandgroups: Default::default(),
            args: Default::default(),
            flags: None,
            pflags: None,
            lflags: None,
            iflags: None,
            parentsPflags: None,
            usageFunc: None,
            flagErrorFunc: None,
            helpFunc: None,
            helpCommand: core::ptr::null_mut(),
            helpCommandGroupID: Default::default(),
            errPrefix: Default::default(),
            outWriter: None,
            errWriter: None,
            FParseErrWhitelist: Default::default(),
            commandsAreSorted: false,
            commandCalledAs: Default::default(),
            commands: alloc::vec::Vec::new(),
            parent: core::ptr::null_mut(),
            commandsMaxUseLen: 0,
            commandsMaxCommandPathLen: 0,
            commandsMaxNameLen: 0,
            TraverseChildren: false,
            Hidden: false,
            SilenceErrors: false,
            SilenceUsage: false,
            DisableFlagParsing: false,
            DisableAutoGenTag: false,
            DisableFlagsInUseLine: false,
            DisableSuggestions: false,
            SuggestionsMinimumDistance: 0,
        }
    }
}

impl Command {
    // go: command.go:281 (SetArgs)
    pub fn SetArgs(&mut self, a: slice<string>) {
        self.args = a;
    }

    // go: command.go:289 (SetOutput) — Deprecated: Use SetOut and/or SetErr
    pub fn SetOutput(&mut self, output: Option<SharedBuf>) {
        self.outWriter = output.clone();
        self.errWriter = output;
    }

    // go: command.go:296 (SetOut)
    pub fn SetOut(&mut self, newOut: Option<SharedBuf>) {
        self.outWriter = newOut;
    }

    // go: command.go:302 (SetErr)
    pub fn SetErr(&mut self, newErr: Option<SharedBuf>) {
        self.errWriter = newErr;
    }

    // go: command.go:313 (SetUsageFunc)
    pub fn SetUsageFunc(&mut self, f: UsageFn) {
        self.usageFunc = Some(f);
    }

    // go: command.go:328 (SetFlagErrorFunc)
    pub fn SetFlagErrorFunc(&mut self, f: FlagErrorFn) {
        self.flagErrorFunc = Some(f);
    }

    // go: command.go:333 (SetHelpFunc)
    pub fn SetHelpFunc(&mut self, f: HelpFn) {
        self.helpFunc = Some(f);
    }

    // go: command.go:343 (SetHelpCommandGroupID)
    pub fn SetHelpCommandGroupID<S: Into<string>>(&mut self, groupID: S) {
        let groupID = groupID.into();
        if !self.helpCommand.is_null() {
            unsafe {
                (*self.helpCommand).GroupID = groupID.clone();
            }
        }
        // helpCommandGroupID is used if no helpCommand is defined by the user
        self.helpCommandGroupID = groupID;
    }

    // go: command.go:376 (SetErrPrefix)
    pub fn SetErrPrefix<S: Into<string>>(&mut self, s: S) {
        self.errPrefix = s.into();
    }

    // go: command.go:393 (OutOrStdout)
    pub fn OutOrStdout(&self) -> cmdWriter {
        self.getOut(cmdWriter::Std(os::Stdout()))
    }

    // go: command.go:398 (OutOrStderr)
    pub fn OutOrStderr(&self) -> cmdWriter {
        self.getOut(cmdWriter::Std(os::Stderr()))
    }

    // go: command.go:403 (ErrOrStderr)
    pub fn ErrOrStderr(&self) -> cmdWriter {
        self.getErr(cmdWriter::Std(os::Stderr()))
    }

    // go: command.go:412 (getOut)
    fn getOut(&self, def: cmdWriter) -> cmdWriter {
        if let Some(w) = &self.outWriter {
            return cmdWriter::Buf(w.clone());
        }
        if self.HasParent() {
            return unsafe { (*self.parent).getOut(def) };
        }
        def
    }

    // go: command.go:422 (getErr)
    fn getErr(&self, def: cmdWriter) -> cmdWriter {
        if let Some(w) = &self.errWriter {
            return cmdWriter::Buf(w.clone());
        }
        if self.HasParent() {
            return unsafe { (*self.parent).getErr(def) };
        }
        def
    }

    // go: command.go:444 (UsageFunc)
    pub fn UsageFunc(&mut self) -> UsageFn {
        if let Some(f) = &self.usageFunc {
            return f.clone();
        }
        if self.HasParent() {
            return unsafe { (*self.parent).UsageFunc() };
        }
        alloc::sync::Arc::new(|c: &mut Command| -> error {
            c.mergePersistentFlags();
            // go: command.go:453 getUsageTemplateFunc — custom templates
            // unsupported; the default is always defaultUsageFunc.
            let mut w = c.OutOrStderr();
            let err = defaultUsageFunc(&mut w, c);
            if err != nil {
                c.PrintErr(fmt::Sprintln!(err.Error()));
            }
            err
        })
    }

    // go: command.go:478 (Usage)
    pub fn Usage(&mut self) -> error {
        let f = self.UsageFunc();
        f(self)
    }

    // go: command.go:484 (HelpFunc)
    pub fn HelpFunc(&mut self) -> HelpFn {
        if let Some(f) = &self.helpFunc {
            return f.clone();
        }
        if self.HasParent() {
            return unsafe { (*self.parent).HelpFunc() };
        }
        alloc::sync::Arc::new(|c: &mut Command, _a: slice<string>| {
            c.mergePersistentFlags();
            // go: command.go:493 getHelpTemplateFunc — custom templates
            // unsupported; the default is always defaultHelpFunc.
            // The help should be sent to stdout
            // See https://github.com/spf13/cobra/issues/1002
            let mut w = c.OutOrStdout();
            let err = defaultHelpFunc(&mut w, c);
            if err != nil {
                c.PrintErr(fmt::Sprintln!(err.Error()));
            }
        })
    }

    // go: command.go:520 (Help)
    pub fn Help(&mut self) -> error {
        let f = self.HelpFunc();
        f(self, make!([]string, 0));
        nil.into()
    }

    // go: command.go:526 (UsageString)
    pub fn UsageString(&mut self) -> string {
        // Storing normal writers
        let tmpOutput = self.outWriter.clone();
        let tmpErr = self.errWriter.clone();

        let bb: SharedBuf = alloc::sync::Arc::new(sync::Mutex::new(bytes::Buffer::new()));
        self.outWriter = Some(bb.clone());
        self.errWriter = Some(bb.clone());

        CheckErr(self.Usage());

        // Setting things back to normal
        self.outWriter = tmpOutput;
        self.errWriter = tmpErr;

        let s = bb.Lock().String();
        s
    }

    // go: command.go:547 (FlagErrorFunc)
    pub fn FlagErrorFunc(&mut self) -> FlagErrorFn {
        if let Some(f) = &self.flagErrorFunc {
            return f.clone();
        }
        if self.HasParent() {
            return unsafe { (*self.parent).FlagErrorFunc() };
        }
        alloc::sync::Arc::new(|_c: &mut Command, err: error| -> error { err })
    }

    // go: command.go:563 (UsagePadding)
    pub fn UsagePadding(&self) -> int {
        if self.parent.is_null() || minUsagePadding > unsafe { (*self.parent).commandsMaxUseLen } {
            return minUsagePadding;
        }
        unsafe { (*self.parent).commandsMaxUseLen }
    }

    // go: command.go:573 (CommandPathPadding)
    pub fn CommandPathPadding(&self) -> int {
        if self.parent.is_null()
            || minCommandPathPadding > unsafe { (*self.parent).commandsMaxCommandPathLen }
        {
            return minCommandPathPadding;
        }
        unsafe { (*self.parent).commandsMaxCommandPathLen }
    }

    // go: command.go:583 (NamePadding)
    pub fn NamePadding(&self) -> int {
        if self.parent.is_null() || minNamePadding > unsafe { (*self.parent).commandsMaxNameLen } {
            return minNamePadding;
        }
        unsafe { (*self.parent).commandsMaxNameLen }
    }

    // go: command.go:643 (ErrPrefix)
    pub fn ErrPrefix(&self) -> string {
        if self.errPrefix != "" {
            return self.errPrefix.clone();
        }
        if self.HasParent() {
            return unsafe { (*self.parent).ErrPrefix() };
        }
        string("Error:")
    }

    // go: command.go:715 (argsMinusFirstX)
    pub(crate) fn argsMinusFirstX(&mut self, args: slice<string>, x: string) -> slice<string> {
        if args.Len() == 0 {
            return args;
        }
        self.mergePersistentFlags();
        let flags = self.flags.as_ref().unwrap();

        let mut pos: int = 0;
        while pos < args.Len() {
            let s = args[pos].clone();
            if s == "--" {
                // -- means we have reached the end of the parseable args.
                break;
            }
            if (strings::HasPrefix(s.clone(), "--")
                && !strings::Contains(s.clone(), "=")
                && !hasNoOptDefVal(s.slice(2, s.Len()), flags))
                || (strings::HasPrefix(s.clone(), "-")
                    && !strings::Contains(s.clone(), "=")
                    && s.Len() == 2
                    && !shortHasNoOptDefVal(s.slice(1, s.Len()), flags))
            {
                // Flag without '=': skip over the next arg (its value).
                // (Go: pos++ in the body plus the loop's pos++.)
                pos += 2;
                continue;
            }
            if !strings::HasPrefix(s.clone(), "-") {
                if s == x {
                    let mut ret: slice<string> = make!([]string, 0, args.Len() - 1);
                    ret = append!(ret, args.slice(0, pos)...);
                    ret = append!(ret, args.slice(pos + 1, args.Len())...);
                    return ret;
                }
            }
            pos += 1;
        }
        args
    }

    // go: command.go:757 (Find) — innerfind is the Go closure, lifted to a
    // method because it recurses through raw child pointers.
    fn innerfind(&mut self, innerArgs: slice<string>) -> (*mut Command, slice<string>) {
        let argsWOflags = stripFlags(innerArgs.clone(), self);
        if argsWOflags.Len() == 0 {
            return (self as *mut Command, innerArgs);
        }
        let nextSubCmd = argsWOflags[0usize].clone();

        let cmd = self.findNext(nextSubCmd.clone());
        if !cmd.is_null() {
            let rest = self.argsMinusFirstX(innerArgs, nextSubCmd);
            return unsafe { (*cmd).innerfind(rest) };
        }
        (self as *mut Command, innerArgs)
    }

    // go: command.go:757 (Find)
    pub fn Find(&mut self, args: slice<string>) -> (*mut Command, slice<string>, error) {
        let (commandFound, a) = self.innerfind(args);
        unsafe {
            if (*commandFound).Args.is_none() {
                let stripped = stripFlags(a.clone(), &mut *commandFound);
                let e = legacyArgs(&mut *commandFound, stripped);
                return (commandFound, a, e);
            }
        }
        (commandFound, a, nil.into())
    }

    // go: command.go:781 (findSuggestions)
    pub(crate) fn findSuggestions<S: Into<string>>(&mut self, arg: S) -> string {
        let arg = arg.into();
        if self.DisableSuggestions {
            return string("");
        }
        if self.SuggestionsMinimumDistance <= 0 {
            self.SuggestionsMinimumDistance = 2;
        }
        let mut sb = strings::Builder::new();
        let suggestions = self.SuggestionsFor(arg);
        if suggestions.Len() > 0 {
            let _ = sb.WriteString("\n\nDid you mean this?\n");
            for (_, s) in goish::range!(suggestions) {
                fmt::Fprintf!(sb, "\t%v\n", s.clone());
            }
        }
        sb.String()
    }

    // go: command.go:798 (findNext)
    pub(crate) fn findNext<S: Into<string>>(&mut self, next: S) -> *mut Command {
        let next = next.into();
        let mut matches: alloc::vec::Vec<*mut Command> = alloc::vec::Vec::new();
        for i in 0..self.commands.len() {
            let cmd: &mut Command = &mut self.commands[i];
            if commandNameMatches(cmd.Name(), next.clone()) || cmd.HasAlias(next.clone()) {
                cmd.commandCalledAs.name = next;
                return cmd as *mut Command;
            }
            if EnablePrefixMatching.load(Ordering::Relaxed) && cmd.hasNameOrAliasPrefix(next.clone())
            {
                matches.push(cmd as *mut Command);
            }
        }

        if matches.len() == 1 {
            return matches[0];
        }

        core::ptr::null_mut()
    }

    // go: command.go:821 (Traverse)
    pub fn Traverse(&mut self, args: slice<string>) -> (*mut Command, slice<string>, error) {
        let mut flags: slice<string> = make!([]string, 0);
        let mut inFlag = false;

        for (i, arg_) in goish::range!(args.clone()) {
            let arg = arg_.clone();
            // A long flag with a space separated value
            if strings::HasPrefix(arg.clone(), "--") && !strings::Contains(arg.clone(), "=") {
                // TODO: this isn't quite right, we should really check ahead for 'true' or 'false'
                inFlag = !hasNoOptDefVal(arg.slice(2, arg.Len()), self.Flags());
                flags = append!(flags, arg);
                continue;
            }
            // A short flag with a space separated value
            if strings::HasPrefix(arg.clone(), "-")
                && !strings::Contains(arg.clone(), "=")
                && arg.Len() == 2
                && !shortHasNoOptDefVal(arg.slice(1, arg.Len()), self.Flags())
            {
                inFlag = true;
                flags = append!(flags, arg);
                continue;
            }
            // The value for a flag
            if inFlag {
                inFlag = false;
                flags = append!(flags, arg);
                continue;
            }
            // A flag without a value, or with an `=` separated value
            if isFlagArg(arg.clone()) {
                flags = append!(flags, arg);
                continue;
            }

            let cmd = self.findNext(arg.clone());
            if cmd.is_null() {
                return (self as *mut Command, args, nil.into());
            }

            let err = self.ParseFlags(flags.clone());
            if err != nil {
                return (core::ptr::null_mut(), args, err);
            }
            return unsafe { (*cmd).Traverse(args.slice(i + 1, args.Len())) };
        }
        (self as *mut Command, args, nil.into())
    }

    // go: command.go:863 (SuggestionsFor)
    pub fn SuggestionsFor<S: Into<string>>(&self, typedName: S) -> slice<string> {
        let typedName = typedName.into();
        let mut suggestions: slice<string> = make!([]string, 0);
        for cmd in self.commands.iter() {
            if cmd.IsAvailableCommand() {
                let levenshteinDistance = ld(typedName.clone(), cmd.Name(), true);
                let suggestByLevenshtein = levenshteinDistance <= self.SuggestionsMinimumDistance;
                let suggestByPrefix = strings::HasPrefix(
                    strings::ToLower(cmd.Name()),
                    strings::ToLower(typedName.clone()),
                );
                if suggestByLevenshtein || suggestByPrefix {
                    suggestions = append!(suggestions, cmd.Name());
                }
                for (_, explicitSuggestion) in goish::range!(cmd.SuggestFor.clone()) {
                    if strings::EqualFold(typedName.clone(), explicitSuggestion.clone()) {
                        suggestions = append!(suggestions, cmd.Name());
                    }
                }
            }
        }
        suggestions
    }

    // go: command.go:884 (VisitParents)
    pub fn VisitParents(&mut self, fn_: &mut dyn FnMut(&mut Command)) {
        if self.HasParent() {
            let p = self.parent;
            unsafe {
                fn_(&mut *p);
                (*p).VisitParents(fn_);
            }
        }
    }

    // go: command.go:892 (Root)
    pub fn Root(&mut self) -> *mut Command {
        if self.HasParent() {
            return unsafe { (*self.parent).Root() };
        }
        self as *mut Command
    }

    // go: command.go:901 (ArgsLenAtDash)
    pub fn ArgsLenAtDash(&mut self) -> int {
        self.Flags().ArgsLenAtDash()
    }

    // go: command.go:905 (execute)
    pub(crate) fn execute(&mut self, a: slice<string>) -> error {
        if self.Deprecated.Len() > 0 {
            self.Print(fmt::Sprintf!(
                "Command %q is deprecated, %s\n",
                self.Name(),
                self.Deprecated.clone()
            ));
        }

        // initialize help and version flag at the last point possible to
        // allow for user overriding
        self.InitDefaultHelpFlag();
        self.InitDefaultVersionFlag();

        let err = self.ParseFlags(a.clone());
        if err != nil {
            let f = self.FlagErrorFunc();
            return f(self, err);
        }

        // If help is called, regardless of other flags, return we want help.
        // Also say we need help if the command isn't runnable.
        let (helpVal, err) = self.Flags().GetBool(helpFlagName);
        if err != nil {
            // should be impossible to get here as we always declare a help
            // flag in InitDefaultHelpFlag()
            self.Println("\"help\" flag declared as non-bool. Please correct your code");
            return err;
        }

        if helpVal {
            return flag::ErrHelp.into();
        }

        // for back-compat, only add version flag behavior if version is defined
        if self.Version != "" {
            let (versionVal, err) = self.Flags().GetBool("version");
            if err != nil {
                self.Println("\"version\" flag declared as non-bool. Please correct your code");
                return err;
            }
            if versionVal {
                // go: command.go:946 getVersionTemplateFunc — custom
                // templates unsupported; always defaultVersionFunc.
                let mut w = self.OutOrStdout();
                let err = defaultVersionFunc(&mut w, self);
                if err != nil {
                    self.Println(err.Error());
                }
                return err;
            }
        }

        if !self.Runnable() {
            return flag::ErrHelp.into();
        }

        self.preRun();

        goish::defer! {
            postRunFinalizers();
        }

        let mut argWoFlags = self.Flags().Args();
        if self.DisableFlagParsing {
            argWoFlags = a.clone();
        }

        let err = self.ValidateArgs(argWoFlags.clone());
        if err != nil {
            return err;
        }

        let mut parents: alloc::vec::Vec<*mut Command> = alloc::vec::Vec::new();
        {
            let mut p: *mut Command = self as *mut Command;
            while !p.is_null() {
                if EnableTraverseRunHooks.load(Ordering::Relaxed) {
                    // When EnableTraverseRunHooks is set:
                    // - Execute all persistent pre-runs from the root parent till this command.
                    // - Execute all persistent post-runs from this command till the root parent.
                    parents.insert(0, p);
                } else {
                    // Otherwise, execute only the first found persistent hook.
                    parents.push(p);
                }
                p = unsafe { (*p).parent };
            }
        }
        for &p in parents.iter() {
            let pprE = unsafe { (*p).PersistentPreRunE.clone() };
            if let Some(f) = pprE {
                let err = f(self, argWoFlags.clone());
                if err != nil {
                    return err;
                }
                if !EnableTraverseRunHooks.load(Ordering::Relaxed) {
                    break;
                }
            } else {
                let ppr = unsafe { (*p).PersistentPreRun.clone() };
                if let Some(f) = ppr {
                    f(self, argWoFlags.clone());
                    if !EnableTraverseRunHooks.load(Ordering::Relaxed) {
                        break;
                    }
                }
            }
        }
        if let Some(f) = self.PreRunE.clone() {
            let err = f(self, argWoFlags.clone());
            if err != nil {
                return err;
            }
        } else if let Some(f) = self.PreRun.clone() {
            f(self, argWoFlags.clone());
        }

        let err = self.ValidateRequiredFlags();
        if err != nil {
            return err;
        }
        // go: command.go:1010 ValidateFlagGroups — flag groups not ported.

        if let Some(f) = self.RunE.clone() {
            let err = f(self, argWoFlags.clone());
            if err != nil {
                return err;
            }
        } else {
            let f = self.Run.clone().unwrap();
            f(self, argWoFlags.clone());
        }
        if let Some(f) = self.PostRunE.clone() {
            let err = f(self, argWoFlags.clone());
            if err != nil {
                return err;
            }
        } else if let Some(f) = self.PostRun.clone() {
            f(self, argWoFlags.clone());
        }
        {
            let mut p: *mut Command = self as *mut Command;
            while !p.is_null() {
                let pprE = unsafe { (*p).PersistentPostRunE.clone() };
                if let Some(f) = pprE {
                    let err = f(self, argWoFlags.clone());
                    if err != nil {
                        return err;
                    }
                    if !EnableTraverseRunHooks.load(Ordering::Relaxed) {
                        break;
                    }
                } else {
                    let ppr = unsafe { (*p).PersistentPostRun.clone() };
                    if let Some(f) = ppr {
                        f(self, argWoFlags.clone());
                        if !EnableTraverseRunHooks.load(Ordering::Relaxed) {
                            break;
                        }
                    }
                }
                p = unsafe { (*p).parent };
            }
        }

        nil.into()
    }

    // go: command.go:1047 (preRun)
    fn preRun(&self) {
        let inits = initializers.Lock();
        for x in inits.iter() {
            x();
        }
    }

    // go: command.go:1053 (postRun)
    fn postRun(&self) {
        postRunFinalizers();
    }

    // go: command.go:1070 (Execute)
    pub fn Execute(&mut self) -> error {
        let (_, err) = self.ExecuteC();
        err
    }

    // go: none — see the ownership-model note at the top of this file: Box
    // gives children stable heap addresses, but the root itself may have
    // moved since AddCommand set the back-pointers; re-anchor before use.
    fn relinkParents(&mut self) {
        let me = self as *mut Command;
        for b in self.commands.iter_mut() {
            b.parent = me;
            b.relinkParents();
        }
    }

    // go: command.go:1084 (ExecuteC)
    pub fn ExecuteC(&mut self) -> (*mut Command, error) {
        // go: command.go:1085 ctx — context not ported.

        // Regardless of what command execute is called on, run on Root only
        if self.HasParent() {
            let root = self.Root();
            return unsafe { (*root).ExecuteC() };
        }

        // go: command.go:1095 preExecHookFn — nil on !windows
        // (command_notwin.go:20); windows mousetrap not ported.

        self.relinkParents();

        // initialize help at the last point to allow for user overriding
        self.InitDefaultHelpCmd();

        let mut args = self.args.clone();

        // Workaround FAIL with "go test -v" or "cobra.test -test.v", see #155
        if self.args == nil && filepath::Base(os::Args()[0usize].clone()) != "cobra.test" {
            let osArgs = os::Args();
            args = osArgs.slice(1, osArgs.Len());
        }

        // go: command.go:1110-1113 initCompleteCmd / InitDefaultCompletionCmd
        // — shell completion not ported.

        // Now that all commands have been created, let's make sure all groups
        // are properly created also
        self.checkCommandGroups();

        let cmd: *mut Command;
        let flags: slice<string>;
        let err: error;
        if self.TraverseChildren {
            let (c2, f2, e2) = self.Traverse(args.clone());
            cmd = c2;
            flags = f2;
            err = e2;
        } else {
            let (c2, f2, e2) = self.Find(args.clone());
            cmd = c2;
            flags = f2;
            err = e2;
        }
        if err != nil {
            // If found parse to a subcommand and then failed, talk about the subcommand
            let cptr: *mut Command = if !cmd.is_null() { cmd } else { self as *mut Command };
            unsafe {
                if !(*cptr).SilenceErrors {
                    let msg = fmt::Sprintln!((*cptr).ErrPrefix(), err.Error());
                    (*cptr).PrintErr(msg);
                    (*cptr).PrintErr(fmt::Sprintf!(
                        "Run '%v --help' for usage.\n",
                        (*cptr).CommandPath()
                    ));
                }
            }
            return (cptr, err);
        }

        unsafe {
            (*cmd).commandCalledAs.called = true;
            if (*cmd).commandCalledAs.name == "" {
                let n = (*cmd).Name();
                (*cmd).commandCalledAs.name = n;
            }

            // go: command.go:1144 ctx propagation — context not ported.

            let err = (*cmd).execute(flags);
            if err != nil {
                // Always show help if requested, even if SilenceErrors is in
                // effect
                if errors::Is(err.clone(), flag::ErrHelp) {
                    let hf = (*cmd).HelpFunc();
                    hf(&mut *cmd, args);
                    return (cmd, nil.into());
                }

                // If root command has SilenceErrors flagged,
                // all subcommands should respect it
                if !(*cmd).SilenceErrors && !self.SilenceErrors {
                    let msg = fmt::Sprintln!((*cmd).ErrPrefix(), err.Error());
                    self.PrintErr(msg);
                }

                // If root command has SilenceUsage flagged,
                // all subcommands should respect it
                if !(*cmd).SilenceUsage && !self.SilenceUsage {
                    let us = (*cmd).UsageString();
                    self.Println(us);
                }
            }
            (cmd, err)
        }
    }

    // go: command.go:1172 (ValidateArgs)
    pub fn ValidateArgs(&mut self, args: slice<string>) -> error {
        if self.Args.is_none() {
            return ArbitraryArgs(self, args);
        }
        let f = self.Args.clone().unwrap();
        f(self, args)
    }

    // go: command.go:1180 (ValidateRequiredFlags)
    pub fn ValidateRequiredFlags(&mut self) -> error {
        if self.DisableFlagParsing {
            return nil.into();
        }

        let mut missingFlagNames: slice<string> = make!([]string, 0);
        {
            let flags = self.Flags();
            flags.VisitAll(|pflag: &flag::Flag| {
                let (requiredAnnotation, found) =
                    pflag.Annotations.Get(string(BashCompOneRequiredFlag));
                if !found {
                    return;
                }
                if requiredAnnotation[0usize] == "true" && !pflag.Changed {
                    missingFlagNames = append!(missingFlagNames.clone(), pflag.Name.clone());
                }
            });
        }

        if missingFlagNames.Len() > 0 {
            return fmt::Errorf!(
                "required flag(s) \"%s\" not set",
                strings::Join(missingFlagNames, "\", \"")
            );
        }
        nil.into()
    }

    // go: command.go:1205 (checkCommandGroups)
    pub(crate) fn checkCommandGroups(&mut self) {
        for i in 0..self.commands.len() {
            // if Group is not defined let the developer know right away
            let gid = self.commands[i].GroupID.clone();
            if gid != "" && !self.ContainsGroup(gid.clone()) {
                let path = self.commands[i].CommandPath();
                let mut e = os::Stderr();
                fmt::Fprintf!(
                    e,
                    "group id '%s' is not defined for subcommand '%s'\n",
                    gid,
                    path
                );
                panic!("group id is not defined for subcommand");
            }
            self.commands[i].checkCommandGroups();
        }
    }

    // go: command.go:1219 (InitDefaultHelpFlag)
    pub fn InitDefaultHelpFlag(&mut self) {
        self.mergePersistentFlags();
        if self.Flags().Lookup(helpFlagName).is_none() {
            let mut usage = string("help for ");
            let name = self.DisplayName();
            if name == "" {
                usage = (usage) + ("this command");
            } else {
                usage = (usage) + (name);
            }
            self.Flags()
                .BoolP(string(helpFlagName), string("h"), false, usage);
            let _ = self.Flags().SetAnnotation(
                helpFlagName,
                FlagSetByCobraAnnotation,
                slice!([]string{"true"}),
            );
        }
    }

    // go: command.go:1238 (InitDefaultVersionFlag)
    pub fn InitDefaultVersionFlag(&mut self) {
        if self.Version == "" {
            return;
        }

        self.mergePersistentFlags();
        if self.Flags().Lookup("version").is_none() {
            let mut usage = string("version for ");
            if self.Name() == "" {
                usage = (usage) + ("this command");
            } else {
                usage = (usage) + (self.DisplayName());
            }
            if self.Flags().ShorthandLookup("v").is_none() {
                self.Flags()
                    .BoolP(string("version"), string("v"), false, usage);
            } else {
                self.Flags().Bool_flag(string("version"), false, usage);
            }
            let _ = self.Flags().SetAnnotation(
                "version",
                FlagSetByCobraAnnotation,
                slice!([]string{"true"}),
            );
        }
    }

    // go: command.go:1263 (InitDefaultHelpCmd)
    pub fn InitDefaultHelpCmd(&mut self) {
        if !self.HasSubCommands() {
            return;
        }

        if self.helpCommand.is_null() {
            let helpCmd = Command {
                Use: string("help [command]"),
                Short: string("Help about any command"),
                Long: string("Help provides help for any command in the application.\nSimply type ")
                    + (self.DisplayName())
                    + (" help [path to command] for full details."),
                // go: command.go:1274 ValidArgsFunction — completion not ported.
                Run: Some(alloc::sync::Arc::new(
                    |c: &mut Command, args: slice<string>| {
                        let root = c.Root();
                        unsafe {
                            let (cmd, _, e) = (*root).Find(args.clone());
                            if cmd.is_null() || e != nil {
                                // go: %#q of []string rendered via joined %v
                                // (goish fmt has no %#q).
                                c.Print(fmt::Sprintf!(
                                    "Unknown help topic %v\n",
                                    strings::Join(args.clone(), " ")
                                ));
                                CheckErr((*root).Usage());
                            } else {
                                (*cmd).InitDefaultHelpFlag(); // make possible 'help' flag to be shown
                                (*cmd).InitDefaultVersionFlag(); // make possible 'version' flag to be shown
                                CheckErr((*cmd).Help());
                            }
                        }
                    },
                )),
                GroupID: self.helpCommandGroupID.clone(),
                ..Default::default()
            };
            self.AddCommand(helpCmd);
            let idx = self.commands.len() - 1;
            self.helpCommand = &mut *self.commands[idx] as *mut Command;
        }
        // go: command.go:1312 RemoveCommand + AddCommand re-registration —
        // the port's helpCommand already lives in `commands`; nothing to do.
    }

    // go: command.go:1332 (Commands) — returns a sorted slice of child
    // commands (sorting per commandSorterByName, command.go:1325).
    pub fn Commands(&mut self) -> &mut alloc::vec::Vec<alloc::boxed::Box<Command>> {
        // do not sort commands if it already sorted or sorting was disabled
        if EnableCommandSorting.load(Ordering::Relaxed) && !self.commandsAreSorted {
            self.commands.sort_by(|a, b| a.Name().cmp(&b.Name()));
            self.commandsAreSorted = true;
        }
        &mut self.commands
    }

    // go: command.go:1342 (AddCommand)
    // KNOWN DIVERGENCE: Go is variadic and takes *Command; the port takes one
    // Command by value per call (the "Command can't be a child of itself"
    // panic is unreachable under move semantics).
    pub fn AddCommand(&mut self, x: Command) {
        let mut b = alloc::boxed::Box::new(x);
        b.parent = self as *mut Command;
        // update max lengths
        let usageLen = b.Use.Len();
        if usageLen > self.commandsMaxUseLen {
            self.commandsMaxUseLen = usageLen;
        }
        let commandPathLen = b.CommandPath().Len();
        if commandPathLen > self.commandsMaxCommandPathLen {
            self.commandsMaxCommandPathLen = commandPathLen;
        }
        let nameLen = b.Name().Len();
        if nameLen > self.commandsMaxNameLen {
            self.commandsMaxNameLen = nameLen;
        }
        // go: command.go:1362 globNormFunc propagation — not ported.
        self.commands.push(b);
        self.commandsAreSorted = false;
    }

    // go: command.go:1371 (Groups)
    pub fn Groups(&self) -> slice<Group> {
        self.commandgroups.clone()
    }

    // go: command.go:1376 (AllChildCommandsHaveGroup)
    pub fn AllChildCommandsHaveGroup(&self) -> bool {
        for sub in self.commands.iter() {
            let isHelp = (&**sub) as *const Command as *mut Command == self.helpCommand;
            if (sub.IsAvailableCommand() || isHelp) && sub.GroupID == "" {
                return false;
            }
        }
        true
    }

    // go: command.go:1386 (ContainsGroup)
    pub fn ContainsGroup<S: Into<string>>(&self, groupID: S) -> bool {
        let groupID = groupID.into();
        for (_, x) in goish::range!(self.commandgroups.clone()) {
            if x.ID == groupID {
                return true;
            }
        }
        false
    }

    // go: command.go:1396 (AddGroup)
    // KNOWN DIVERGENCE: Go is variadic; the port takes one Group per call.
    pub fn AddGroup(&mut self, group: Group) {
        self.commandgroups = append!(self.commandgroups.clone(), group);
    }

    // go: command.go:1435 (Print)
    // KNOWN DIVERGENCE: the Print family takes one pre-formatted string
    // (Go is variadic over interface{}); internal call sites pre-format
    // with fmt::Sprintf!/Sprintln!.
    pub fn Print<S: Into<string>>(&mut self, i: S) {
        let s = i.into();
        let mut w = self.OutOrStderr();
        fmt::Fprintf!(w, "%s", s);
    }

    // go: command.go:1440 (Println)
    pub fn Println<S: Into<string>>(&mut self, i: S) {
        let s = i.into();
        self.Print(fmt::Sprintln!(s));
    }

    // go: command.go:1450 (PrintErr)
    pub fn PrintErr<S: Into<string>>(&mut self, i: S) {
        let s = i.into();
        let mut w = self.ErrOrStderr();
        fmt::Fprintf!(w, "%s", s);
    }

    // go: command.go:1455 (PrintErrln)
    pub fn PrintErrln<S: Into<string>>(&mut self, i: S) {
        let s = i.into();
        self.PrintErr(fmt::Sprintln!(s));
    }

    // go: command.go:1465 (CommandPath)
    pub fn CommandPath(&self) -> string {
        if self.HasParent() {
            return unsafe { (*self.parent).CommandPath() } + (" ") + (self.Name());
        }
        self.DisplayName()
    }

    // go: command.go:1474 (DisplayName)
    pub fn DisplayName(&self) -> string {
        let (displayName, ok) = self.Annotations.Get(string(CommandDisplayNameAnnotation));
        if ok {
            return displayName;
        }
        self.Name()
    }

    // go: command.go:1482 (UseLine)
    pub fn UseLine(&self) -> string {
        let mut useline: string;
        let use_ = strings::Replace(self.Use.clone(), self.Name(), self.DisplayName(), 1);
        if self.HasParent() {
            useline = unsafe { (*self.parent).CommandPath() } + (" ") + (use_);
        } else {
            useline = use_;
        }
        if self.DisableFlagsInUseLine {
            return useline;
        }
        if self.hasAvailableFlagsRO() && !strings::Contains(useline.clone(), "[flags]") {
            useline = (useline) + (" [flags]");
        }
        useline
    }

    // go: none — UseLine needs HasAvailableFlags from a &self context; the
    // flag sets are already merged by the time usage is rendered, so read
    // the current `flags` set without the lazy-init &mut path.
    fn hasAvailableFlagsRO(&self) -> bool {
        match &self.flags {
            Some(f) => f.HasAvailableFlags(),
            None => false,
        }
    }

    // go: command.go:1541 (Name)
    pub fn Name(&self) -> string {
        let mut name = self.Use.clone();
        let i = strings::Index(name.clone(), " ");
        if i >= 0 {
            name = name.slice(0, i);
        }
        name
    }

    // go: command.go:1551 (HasAlias)
    pub fn HasAlias<S: Into<string>>(&self, s: S) -> bool {
        let s = s.into();
        for (_, a) in goish::range!(self.Aliases.clone()) {
            if commandNameMatches(a.clone(), s.clone()) {
                return true;
            }
        }
        false
    }

    // go: command.go:1562 (CalledAs)
    pub fn CalledAs(&self) -> string {
        if self.commandCalledAs.called {
            return self.commandCalledAs.name.clone();
        }
        string("")
    }

    // go: command.go:1571 (hasNameOrAliasPrefix)
    pub(crate) fn hasNameOrAliasPrefix<S: Into<string>>(&mut self, prefix: S) -> bool {
        let prefix = prefix.into();
        if strings::HasPrefix(self.Name(), prefix.clone()) {
            self.commandCalledAs.name = self.Name();
            return true;
        }
        for (_, alias) in goish::range!(self.Aliases.clone()) {
            if strings::HasPrefix(alias.clone(), prefix.clone()) {
                self.commandCalledAs.name = alias.clone();
                return true;
            }
        }
        false
    }

    // go: command.go:1586 (NameAndAliases)
    pub fn NameAndAliases(&self) -> string {
        let mut list: slice<string> = make!([]string, 0);
        list = append!(list, self.Name());
        list = append!(list, self.Aliases.clone()...);
        strings::Join(list, ", ")
    }

    // go: command.go:1591 (HasExample)
    pub fn HasExample(&self) -> bool {
        self.Example.Len() > 0
    }

    // go: command.go:1596 (Runnable)
    pub fn Runnable(&self) -> bool {
        self.Run.is_some() || self.RunE.is_some()
    }

    // go: command.go:1601 (HasSubCommands)
    pub fn HasSubCommands(&self) -> bool {
        self.commands.len() > 0
    }

    // go: command.go:1607 (IsAvailableCommand)
    pub fn IsAvailableCommand(&self) -> bool {
        if self.Deprecated.Len() != 0 || self.Hidden {
            return false;
        }

        if self.HasParent() {
            let me = self as *const Command as *mut Command;
            if unsafe { (*self.parent).helpCommand } == me {
                return false;
            }
        }

        if self.Runnable() || self.HasAvailableSubCommands() {
            return true;
        }

        false
    }

    // go: command.go:1628 (IsAdditionalHelpTopicCommand)
    pub fn IsAdditionalHelpTopicCommand(&self) -> bool {
        // if a command is runnable, deprecated, or hidden it is not a 'help' command
        if self.Runnable() || self.Deprecated.Len() != 0 || self.Hidden {
            return false;
        }

        // if any non-help sub commands are found, the command is not a 'help' command
        for sub in self.commands.iter() {
            if !sub.IsAdditionalHelpTopicCommand() {
                return false;
            }
        }

        // the command either has no sub commands, or no non-help sub commands
        true
    }

    // go: command.go:1648 (HasHelpSubCommands)
    pub fn HasHelpSubCommands(&self) -> bool {
        // return true on the first found available 'help' sub command
        for sub in self.commands.iter() {
            if sub.IsAdditionalHelpTopicCommand() {
                return true;
            }
        }

        // the command either has no sub commands, or no available 'help' sub commands
        false
    }

    // go: command.go:1662 (HasAvailableSubCommands)
    pub fn HasAvailableSubCommands(&self) -> bool {
        // return true on the first found available (non deprecated/help/hidden)
        // sub command
        for sub in self.commands.iter() {
            if sub.IsAvailableCommand() {
                return true;
            }
        }

        // the command either has no sub commands, or no available (non deprecated/help/hidden)
        // sub commands
        false
    }

    // go: command.go:1677 (HasParent)
    pub fn HasParent(&self) -> bool {
        !self.parent.is_null()
    }

    // go: command.go:1688 (Flags)
    pub fn Flags(&mut self) -> &mut flag::FlagSet {
        if self.flags.is_none() {
            let fs = flag::NewFlagSet(self.DisplayName(), flag::ContinueOnError);
            // go: command.go:1691 flagErrorBuf / SetOutput — not ported.
            self.flags = Some(alloc::boxed::Box::new(fs));
        }
        self.flags.as_mut().unwrap()
    }

    // go: command.go:1702 (LocalNonPersistentFlags)
    pub fn LocalNonPersistentFlags(&mut self) -> flag::FlagSet {
        self.PersistentFlags();

        let mut out = flag::NewFlagSet(self.DisplayName(), flag::ContinueOnError);
        {
            self.LocalFlags();
            let lflags = self.lflags.as_ref().unwrap();
            let persistentFlags = self.pflags.as_ref().unwrap();
            lflags.VisitAll(|f: &flag::Flag| {
                if persistentFlags.Lookup(f.Name.clone()).is_none() {
                    out.AddFlag(f);
                }
            });
        }
        out
    }

    // go: command.go:1716 (LocalFlags)
    pub fn LocalFlags(&mut self) -> &mut flag::FlagSet {
        self.mergePersistentFlags();

        if self.lflags.is_none() {
            let fs = flag::NewFlagSet(self.DisplayName(), flag::ContinueOnError);
            self.lflags = Some(alloc::boxed::Box::new(fs));
        }
        let sort = self.Flags().SortFlags;
        let mut lf = self.lflags.take().unwrap();
        lf.SortFlags = sort;
        // go: command.go:1727 globNormFunc — not ported.
        {
            let parentsPflags = self.parentsPflags.as_ref().unwrap();
            let flags = self.flags.as_ref().unwrap();
            let pflags = self.pflags.as_ref().unwrap();
            // go: command.go:1731 addToLocal — the Go identity check
            // (f != parentsPflags.Lookup(f.Name)) becomes a name-presence
            // check: the arena clones flags, so pointer identity is lost.
            // KNOWN DIVERGENCE: a local flag that shadows a parent's
            // persistent flag by name is treated as inherited.
            let mut addToLocal = |f: &flag::Flag| {
                if lf.Lookup(f.Name.clone()).is_none()
                    && parentsPflags.Lookup(f.Name.clone()).is_none()
                {
                    lf.AddFlag(f);
                }
            };
            flags.VisitAll(&mut addToLocal);
            pflags.VisitAll(&mut addToLocal);
        }
        self.lflags = Some(lf);
        self.lflags.as_mut().unwrap()
    }

    // go: command.go:1744 (InheritedFlags)
    pub fn InheritedFlags(&mut self) -> &mut flag::FlagSet {
        self.mergePersistentFlags();

        if self.iflags.is_none() {
            let fs = flag::NewFlagSet(self.DisplayName(), flag::ContinueOnError);
            self.iflags = Some(alloc::boxed::Box::new(fs));
        }

        self.LocalFlags();
        let mut ifl = self.iflags.take().unwrap();
        {
            let local = self.lflags.as_ref().unwrap();
            let parentsPflags = self.parentsPflags.as_ref().unwrap();
            parentsPflags.VisitAll(|f: &flag::Flag| {
                if ifl.Lookup(f.Name.clone()).is_none() && local.Lookup(f.Name.clone()).is_none() {
                    ifl.AddFlag(f);
                }
            });
        }
        self.iflags = Some(ifl);
        self.iflags.as_mut().unwrap()
    }

    // go: command.go:1770 (NonInheritedFlags)
    pub fn NonInheritedFlags(&mut self) -> &mut flag::FlagSet {
        self.LocalFlags()
    }

    // go: command.go:1775 (PersistentFlags)
    pub fn PersistentFlags(&mut self) -> &mut flag::FlagSet {
        if self.pflags.is_none() {
            let fs = flag::NewFlagSet(self.DisplayName(), flag::ContinueOnError);
            self.pflags = Some(alloc::boxed::Box::new(fs));
        }
        self.pflags.as_mut().unwrap()
    }

    // go: command.go:1801 (HasFlags)
    pub fn HasFlags(&mut self) -> bool {
        self.Flags().HasFlags()
    }

    // go: command.go:1806 (HasPersistentFlags)
    pub fn HasPersistentFlags(&mut self) -> bool {
        self.PersistentFlags().HasFlags()
    }

    // go: command.go:1811 (HasLocalFlags)
    pub fn HasLocalFlags(&mut self) -> bool {
        self.LocalFlags().HasFlags()
    }

    // go: command.go:1816 (HasInheritedFlags)
    pub fn HasInheritedFlags(&mut self) -> bool {
        self.InheritedFlags().HasFlags()
    }

    // go: command.go:1822 (HasAvailableFlags)
    pub fn HasAvailableFlags(&mut self) -> bool {
        self.Flags().HasAvailableFlags()
    }

    // go: command.go:1827 (HasAvailablePersistentFlags)
    pub fn HasAvailablePersistentFlags(&mut self) -> bool {
        self.PersistentFlags().HasAvailableFlags()
    }

    // go: command.go:1833 (HasAvailableLocalFlags)
    pub fn HasAvailableLocalFlags(&mut self) -> bool {
        self.LocalFlags().HasAvailableFlags()
    }

    // go: command.go:1839 (HasAvailableInheritedFlags)
    pub fn HasAvailableInheritedFlags(&mut self) -> bool {
        self.InheritedFlags().HasAvailableFlags()
    }

    // go: command.go:1868 (ParseFlags)
    pub fn ParseFlags(&mut self, args: slice<string>) -> error {
        if self.DisableFlagParsing {
            return nil.into();
        }

        // go: command.go:1873 flagErrorBuf warning capture — not ported (the
        // ported pflag prints deprecation warnings straight to stderr).
        self.mergePersistentFlags();

        // do it here after merging all flags and just before parse
        self.Flags().ParseErrorsAllowlist = self.FParseErrWhitelist.clone();

        let err = self.Flags().Parse(args);
        err
    }

    // go: command.go:1892 (Parent)
    pub fn Parent(&self) -> *mut Command {
        self.parent
    }

    // go: command.go:1898 (mergePersistentFlags)
    pub(crate) fn mergePersistentFlags(&mut self) {
        self.updateParentsPflags();
        self.PersistentFlags();
        let pflags = self.pflags.take().unwrap();
        self.Flags().AddFlagSet(&pflags);
        self.pflags = Some(pflags);
        let parentsPflags = self.parentsPflags.take().unwrap();
        self.Flags().AddFlagSet(&parentsPflags);
        self.parentsPflags = Some(parentsPflags);
    }

    // go: command.go:1907 (updateParentsPflags)
    pub(crate) fn updateParentsPflags(&mut self) {
        if self.parentsPflags.is_none() {
            let mut fs = flag::NewFlagSet(self.DisplayName(), flag::ContinueOnError);
            fs.SortFlags = false;
            self.parentsPflags = Some(alloc::boxed::Box::new(fs));
        }

        // go: command.go:1914 globNormFunc — not ported.

        {
            let root = self.Root();
            let cl = flag::COMMAND_LINE.Lock();
            unsafe {
                (*root).PersistentFlags().AddFlagSet(&cl);
            }
        }

        let mut parentsPflags = self.parentsPflags.take().unwrap();
        self.VisitParents(&mut |parent: &mut Command| {
            parentsPflags.AddFlagSet(parent.PersistentFlags());
        });
        self.parentsPflags = Some(parentsPflags);
    }
}

const minUsagePadding: int = 25;

const minCommandPathPadding: int = 11;

const minNamePadding: int = 11;

// go: command.go:654 (hasNoOptDefVal)
pub(crate) fn hasNoOptDefVal<S: Into<string>>(name: S, fs: &flag::FlagSet) -> bool {
    let name = name.into();
    match fs.Lookup(name) {
        None => false,
        Some(flag_) => flag_.NoOptDefVal != "",
    }
}

// go: command.go:662 (shortHasNoOptDefVal)
pub(crate) fn shortHasNoOptDefVal<S: Into<string>>(name: S, fs: &flag::FlagSet) -> bool {
    let name = name.into();
    if name.Len() == 0 {
        return false;
    }

    match fs.ShorthandLookup(name.slice(0, 1)) {
        None => false,
        Some(flag_) => flag_.NoOptDefVal != "",
    }
}

// go: command.go:674 (stripFlags)
pub(crate) fn stripFlags(args: slice<string>, c: &mut Command) -> slice<string> {
    if args.Len() == 0 {
        return args;
    }
    c.mergePersistentFlags();

    let mut commands: slice<string> = make!([]string, 0);
    let mut args = args;
    let flags = c.Flags();

    'Loop: while args.Len() > 0 {
        let s = args[0usize].clone();
        args = args.slice(1, args.Len());
        if s == "--" {
            // "--" terminates the flags
            break 'Loop;
        } else if (strings::HasPrefix(s.clone(), "--")
            && !strings::Contains(s.clone(), "=")
            && !hasNoOptDefVal(s.slice(2, s.Len()), flags))
            || (strings::HasPrefix(s.clone(), "-")
                && !strings::Contains(s.clone(), "=")
                && s.Len() == 2
                && !shortHasNoOptDefVal(s.slice(1, s.Len()), flags))
        {
            // If '--flag arg' / '-f arg' then delete arg from args or break
            // the loop if len(args) <= 1. (Go: two cases joined by
            // fallthrough.)
            if args.Len() <= 1 {
                break 'Loop;
            } else {
                args = args.slice(1, args.Len());
                continue;
            }
        } else if s != "" && !strings::HasPrefix(s.clone(), "-") {
            commands = append!(commands, s);
        }
    }

    commands
}

// go: command.go:750 (isFlagArg)
pub(crate) fn isFlagArg<S: Into<string>>(arg: S) -> bool {
    let arg = arg.into();
    (arg.Len() >= 3 && arg.slice(0, 2) == "--")
        || (arg.Len() >= 2 && arg[0] == b'-' && arg[1] != b'-')
}

// go: command.go:1928 (commandNameMatches)
pub(crate) fn commandNameMatches<S1: Into<string>, S2: Into<string>>(s: S1, t: S2) -> bool {
    let s = s.into();
    let t = t.into();
    if EnableCaseInsensitive.load(Ordering::Relaxed) {
        return strings::EqualFold(s, t);
    }

    s == t
}

// go: command.go:1053 (postRun body) — lifted to a free function so
// execute() can `defer` it without capturing &self.
pub(crate) fn postRunFinalizers() {
    let fins = finalizers.Lock();
    for x in fins.iter() {
        x();
    }
}

// go: command.go:1974 (defaultUsageFunc) — equivalent to executing
// defaultUsageTemplate.
pub(crate) fn defaultUsageFunc(w: &mut dyn io::Writer, c: &mut Command) -> error {
    fmt::Fprintf!(*w, "Usage:");
    if c.Runnable() {
        fmt::Fprintf!(*w, "\n  %s", c.UseLine());
    }
    if c.HasAvailableSubCommands() {
        fmt::Fprintf!(*w, "\n  %s [command]", c.CommandPath());
    }
    if c.Aliases.Len() > 0 {
        fmt::Fprintf!(*w, "\n\nAliases:\n");
        fmt::Fprintf!(*w, "  %s", c.NameAndAliases());
    }
    if c.HasExample() {
        fmt::Fprintf!(*w, "\n\nExamples:\n");
        fmt::Fprintf!(*w, "%s", c.Example.clone());
    }
    if c.HasAvailableSubCommands() {
        c.Commands(); // sort side effect; iterate c.commands below
        if c.Groups().Len() == 0 {
            fmt::Fprintf!(*w, "\n\nAvailable Commands:");
            for i in 0..c.commands.len() {
                let (avail, name, padding, short) = {
                    let subcmd = &c.commands[i];
                    (
                        subcmd.IsAvailableCommand(),
                        subcmd.Name(),
                        subcmd.NamePadding(),
                        subcmd.Short.clone(),
                    )
                };
                if avail || name == helpCommandName {
                    fmt::Fprintf!(*w, "\n  %s %s", rpad(name, padding), short);
                }
            }
        } else {
            for (_, group) in goish::range!(c.Groups()) {
                fmt::Fprintf!(*w, "\n\n%s", group.Title.clone());
                for i in 0..c.commands.len() {
                    let (gid, avail, name, padding, short) = {
                        let subcmd = &c.commands[i];
                        (
                            subcmd.GroupID.clone(),
                            subcmd.IsAvailableCommand(),
                            subcmd.Name(),
                            subcmd.NamePadding(),
                            subcmd.Short.clone(),
                        )
                    };
                    if gid == group.ID && (avail || name == helpCommandName) {
                        fmt::Fprintf!(*w, "\n  %s %s", rpad(name, padding), short);
                    }
                }
            }
            if !c.AllChildCommandsHaveGroup() {
                fmt::Fprintf!(*w, "\n\nAdditional Commands:");
                for i in 0..c.commands.len() {
                    let (gid, avail, name, padding, short) = {
                        let subcmd = &c.commands[i];
                        (
                            subcmd.GroupID.clone(),
                            subcmd.IsAvailableCommand(),
                            subcmd.Name(),
                            subcmd.NamePadding(),
                            subcmd.Short.clone(),
                        )
                    };
                    if gid == "" && (avail || name == helpCommandName) {
                        fmt::Fprintf!(*w, "\n  %s %s", rpad(name, padding), short);
                    }
                }
            }
        }
    }
    if c.HasAvailableLocalFlags() {
        fmt::Fprintf!(*w, "\n\nFlags:\n");
        fmt::Fprintf!(*w, "%s", trimRightSpace(c.LocalFlags().FlagUsages()));
    }
    if c.HasAvailableInheritedFlags() {
        fmt::Fprintf!(*w, "\n\nGlobal Flags:\n");
        fmt::Fprintf!(*w, "%s", trimRightSpace(c.InheritedFlags().FlagUsages()));
    }
    if c.HasHelpSubCommands() {
        fmt::Fprintf!(*w, "\n\nAdditional help topics:");
        for i in 0..c.commands.len() {
            let (isTopic, path, padding, short) = {
                let subcmd = &c.commands[i];
                (
                    subcmd.IsAdditionalHelpTopicCommand(),
                    subcmd.CommandPath(),
                    subcmd.CommandPathPadding(),
                    subcmd.Short.clone(),
                )
            };
            if isTopic {
                fmt::Fprintf!(*w, "\n  %s %s", rpad(path, padding), short);
            }
        }
    }
    if c.HasAvailableSubCommands() {
        fmt::Fprintf!(
            *w,
            "\n\nUse \"%s [command] --help\" for more information about a command.",
            c.CommandPath()
        );
    }
    fmt::Fprintln!(*w);
    nil.into()
}

// go: command.go:2047 (defaultHelpFunc) — equivalent to executing
// defaultHelpTemplate.
pub(crate) fn defaultHelpFunc(w: &mut dyn io::Writer, c: &mut Command) -> error {
    let mut usage = c.Long.clone();
    if usage == "" {
        usage = c.Short.clone();
    }
    usage = trimRightSpace(usage);
    if usage != "" {
        fmt::Fprintln!(*w, usage);
        fmt::Fprintln!(*w);
    }
    if c.Runnable() || c.HasSubCommands() {
        fmt::Fprintf!(*w, "%s", c.UsageString());
    }
    nil.into()
}

// go: command.go:2068 (defaultVersionFunc) — equivalent to executing
// defaultVersionTemplate.
pub(crate) fn defaultVersionFunc(w: &mut dyn io::Writer, c: &mut Command) -> error {
    let (_, err) = {
        let s = fmt::Sprintf!("%s version %s\n", c.DisplayName(), c.Version.clone());
        w.Write(bytes(s))
    };
    err
}
