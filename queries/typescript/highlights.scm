; TypeScript highlights (Helix-style)
(identifier) @variable
(type_identifier) @type
(predefined_type) @type.builtin

(function_declaration
  name: (identifier) @function)

(function_expression
  name: (identifier)? @function)

(generator_function_declaration
  name: (identifier) @function)

(class_declaration
  name: (identifier) @type)

(interface_declaration
  name: (type_identifier) @type)

(enum_declaration
  name: (identifier) @type)

(type_alias_declaration
  name: (type_identifier) @type)

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
  "abstract"
  "as"
  "async"
  "await"
  "break"
  "case"
  "catch"
  "class"
  "const"
  "continue"
  "debugger"
  "declare"
  "default"
  "delete"
  "do"
  "else"
  "enum"
  "export"
  "extends"
  "finally"
  "for"
  "from"
  "function"
  "get"
  "if"
  "implements"
  "import"
  "in"
  "instanceof"
  "interface"
  "is"
  "keyof"
  "let"
  "namespace"
  "never"
  "new"
  "of"
  "package"
  "private"
  "protected"
  "public"
  "readonly"
  "return"
  "set"
  "static"
  "super"
  "switch"
  "this"
  "throw"
  "try"
  "type"
  "typeof"
  "var"
  "void"
  "while"
  "with"
  "yield"
] @keyword

(type_arguments
  "<" @punctuation.bracket
  ">" @punctuation.bracket)

(type_parameters
  "<" @punctuation.bracket
  ">" @punctuation.bracket)

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