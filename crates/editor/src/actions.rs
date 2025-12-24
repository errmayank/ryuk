use gpui::Action;

/// Delete the character before cursor
#[derive(PartialEq, Clone, Default, Action)]
#[action(namespace = editor)]
pub struct Backspace;

/// Delete from cursor to beginning of the line
#[derive(PartialEq, Clone, Default, Action)]
#[action(namespace = editor)]
pub struct DeleteToBeginningOfLine;

/// Delete the character after cursor
#[derive(PartialEq, Clone, Default, Action)]
#[action(namespace = editor)]
pub struct Delete;

/// Delete from cursor to end of the line
#[derive(PartialEq, Clone, Default, Action)]
#[action(namespace = editor)]
pub struct DeleteToEndOfLine;

/// Toggle bold formatting on selected text
#[derive(PartialEq, Clone, Default, Action)]
#[action(namespace = editor)]
pub struct ToggleBold;

/// Toggle italic formatting on selected text
#[derive(PartialEq, Clone, Default, Action)]
#[action(namespace = editor)]
pub struct ToggleItalic;

/// Toggle underline formatting on selected text
#[derive(PartialEq, Clone, Default, Action)]
#[action(namespace = editor)]
pub struct ToggleUnderline;

/// Insert newline at cursor position
#[derive(PartialEq, Clone, Default, Action)]
#[action(namespace = editor)]
pub struct Newline;

/// Move cursor up one line
#[derive(PartialEq, Clone, Default, Action)]
#[action(namespace = editor)]
pub struct MoveUp;

/// Move cursor down one line
#[derive(PartialEq, Clone, Default, Action)]
#[action(namespace = editor)]
pub struct MoveDown;

/// Move cursor left one character
#[derive(PartialEq, Clone, Default, Action)]
#[action(namespace = editor)]
pub struct MoveLeft;

/// Move cursor right one character
#[derive(PartialEq, Clone, Default, Action)]
#[action(namespace = editor)]
pub struct MoveRight;
