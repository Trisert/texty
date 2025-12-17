; JavaScript highlights (Helix-style)
(identifier) @variable

(function_declaration
  name: (identifier) @function)

(function_expression
  name: (identifier)? @function)

(generator_function_declaration
  name: (identifier) @function)

(class_declaration
  name: (identifier) @type)

(method_definition
  name: (property_identifier) @function.method)

(call_expression
  function: (identifier) @function.call)

(member_expression
  object: (identifier) @variable
  property: (identifier) @variable.member)

(subscript_expression
  object: (identifier) @variable)

(assignment_expression
  left: (identifier) @variable)

(variable_declarator
  name: (identifier) @variable)

(formal_parameters
  (identifier) @variable.parameter)

(comment) @comment

(string) @string
(template_string) @string
(escape_sequence) @string.escape

(regex) @string.regex
(regex_flags) @string.regex

(number) @constant.numeric.integer

(boolean) @constant.builtin
(null) @constant.builtin
(undefined) @constant.builtin

[
  "async"
  "await"
  "break"
  "case"
  "catch"
  "class"
  "const"
  "continue"
  "debugger"
  "default"
  "delete"
  "do"
  "else"
  "export"
  "extends"
  "finally"
  "for"
  "function"
  "if"
  "import"
  "in"
  "instanceof"
  "let"
  "new"
  "of"
  "return"
  "static"
  "super"
  "switch"
  "this"
  "throw"
  "try"
  "typeof"
  "var"
  "void"
  "while"
  "with"
  "yield"
] @keyword

[
  "="
  "-"
  "+"
  "*"
  "/"
  "%"
  "**"
  "=="
  "!="
  "==="
  "!=="
  "<"
  "<="
  ">"
  ">="
  "&&"
  "||"
  "!"
  "??"
  "?"
  ":"
  "+="
  "-="
  "*="
  "/="
  "%="
  "**="
  "&&="
  "||="
  "??="
  "<<"
  ">>"
  ">>>"
  "<<="
  ">>="
  ">>>="
  "&"
  "|"
  "^"
  "~"
  "&="
  "|="
  "^="
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
  ","
  "."
  ";"
] @punctuation.delimiter