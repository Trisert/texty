; Rust highlights (Helix-style)
(identifier) @variable
(type_identifier) @type
(primitive_type) @type.builtin

(function_item
  name: (identifier) @function)

(macro_invocation
  macro: (identifier) @function.macro)

(field_identifier) @variable.member
(shorthand_field_identifier) @variable.member

(line_comment) @comment
(block_comment) @comment

(string_literal) @string
(char_literal) @string
(raw_string_literal) @string

(boolean_literal) @constant.builtin
(integer_literal) @constant.numeric.integer
(float_literal) @constant.numeric.float

(escape_sequence) @string.escape

[
  "="
  "-"
  "+"
  "*"
  "/"
  "%"
  "^"
  "&"
  "|"
  "!"
  ">"
  "<"
  ">="
  "<="
  "=="
  "!="
  "+="
  "-="
  "*="
  "/="
  "%="
  "^="
  "&="
  "|="
  "<<"
  ">>"
  "<<="
  ">>="
  "=>"
  "->"
  "<-"
  "::"
  ":"
  ";"
  ","
  "."
  ".."
  "..="
  "_"
] @operator

[
  "("
  ")"
  "["
  "]"
  "{"
  "}"
] @punctuation.bracket

[
  "as"
  "async"
  "await"
  "break"
  "const"
  "continue"
  "crate"
  "dyn"
  "else"
  "enum"
  "extern"
  "fn"
  "for"
  "if"
  "impl"
  "in"
  "let"
  "loop"
  "match"
  "mod"
  "move"
  "mut"
  "pub"
  "ref"
  "return"
  "static"
  "struct"
  "trait"
  "type"
  "union"
  "unsafe"
  "use"
  "where"
  "while"
] @keyword