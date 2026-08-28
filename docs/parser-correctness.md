# Parser Correctness

Boundra keeps a fixture-driven import extraction corpus under
`crates/parser/tests/fixtures/correctness`.

The oracle records exact `(source file, line, import path)` tuples. Cases that
must not be reported are intentionally present in source fixtures but absent
from `expected.tsv`, so the suite catches false positives and false negatives
together.

Current corpus coverage includes:

- static import/export forms and type-only forms
- multiline imports
- `require()` and dynamic `import()`
- multiple calls on the same line
- static and interpolated template literals
- non-literal calls followed by unrelated strings
- comments, string literals, and method-like call false positives
- import attributes and non-ASCII module specifiers
- Svelte script blocks while masking markup

Every parser correctness bug should add a minimal corpus case before the fix.
This suite is the stable local regression layer; larger real-repository
differential checks against an AST parser can be added without changing this
oracle format.
