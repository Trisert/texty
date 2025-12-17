; Python highlights (Helix-style)
(identifier) @variable
(type
  (identifier) @type.builtin)

(function_definition
  name: (identifier) @function)

(class_definition
  name: (identifier) @type)

(parameters
  (identifier) @variable.parameter)

(attribute
  object: (identifier) @variable
  attribute: (identifier) @variable.member)

(comment) @comment

(string) @string
(escape_sequence) @string.escape

[
  "and"
  "as"
  "assert"
  "async"
  "await"
  "break"
  "class"
  "continue"
  "def"
  "del"
  "elif"
  "else"
  "except"
  "finally"
  "for"
  "from"
  "global"
  "if"
  "import"
  "in"
  "is"
  "lambda"
  "nonlocal"
  "not"
  "or"
  "pass"
  "raise"
  "return"
  "try"
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
  "//"
  "=="
  "!="
  "<"
  "<="
  ">"
  ">="
  "+="
  "-="
  "*="
  "/="
  "%="
  "**="
  "//="
  "&"
  "|"
  "^"
  "~"
  "<<"
  ">>"
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
  ":"
  ";"
] @punctuation.delimiter

(boolean) @constant.builtin
(none) @constant.builtin
(integer) @constant.numeric.integer
(float) @constant.numeric.float