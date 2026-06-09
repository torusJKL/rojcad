## ADDED Requirements

### Requirement: Standard macro definitions available at REPL

The system SHALL make the full suite of standard Janet macros available to REPL clients, as defined in upstream Janet's `src/boot/boot.janet`.

The following macros MUST be available with behavior identical to standard Janet v1.41.2:
- `defmacro`, `defn`, `defn-`, `def-`, `defmacro-`, `var-`
- `comment`, `if-not`, `when`, `unless`, `cond`, `case`, `let`
- `and`, `or`, `try`, `protect`, `defer`, `edefer`, `prompt`, `label`, `return`
- `with`, `when-with`, `if-with`
- `->`, `->>`, `-?>`, `-?>>`, `as->`, `as?->`
- `each`, `eachk`, `eachp`, `for`, `forv`, `loop`, `seq`, `catseq`, `tabseq`
- `repeat`, `forever`, `generate`, `coro`
- `default`, `toggle`, `++`, `--`, `+=`, `-=`, `*=`, `/=`, `%=`
- `if-let`, `when-let`, `with-syms`, `assert`, `assertf`
- `juxt`, `tracev`, `comp`, `complement`
- `map`, `filter`, `count`, `keep`, `reduce`, `reduce2`, `accumulate`, `accumulate2`
- `take`, `drop`, `take-while`, `drop-while`, `take-until`, `drop-until`
- `sort`, `sorted`, `sort-by`, `sorted-by`
- `find`, `find-index`, `index-of`
- `every?`, `any?`, `reverse`, `reverse!`
- `walk`, `postwalk`, `prewalk`
- `doc`, `doc-of`, `doc-format`
- `partial`, `identity`, `juxt*`, `comp`
- `errorf`, `maclintf`
- All predicate helpers (`nil?`, `true?`, `false?`, `boolean?`, `number?`, `string?`, `symbol?`, `keyword?`, `buffer?`, `function?`, `cfunction?`, `table?`, `struct?`, `array?`, `tuple?`, `fiber?`, `empty?`, `nan?`, `zero?`, `pos?`, `neg?`, `one?`, `even?`, `odd?`, `truthy?`, `idempotent?`)
- Helper functions (`inc`, `dec`, `sum`, `mean`, `geomean`, `product`, `min`, `max`, `min-of`, `max-of`, `first`, `last`, `extreme`, `compare`, `compare=`, `compare<`, `compare>`, `compare<=`, `compare>=`, `has-key?`, `has-value?`)
- Dynamic binding helpers (`defdyn`, `with-dyns`, `with-env`, `with-vars`)

#### Scenario: defmacro works at REPL
- **WHEN** a client sends `(defmacro twice [x] (tuple '+ x x))` followed by `(twice 5)`
- **THEN** the REPL returns `10`

#### Scenario: defn works at REPL
- **WHEN** a client sends `(defn square [x] (* x x))` followed by `(square 4)`
- **THEN** the REPL returns `16`

#### Scenario: threading macros work at REPL
- **WHEN** a client sends `(-> 5 (+ 3) (* 2))`
- **THEN** the REPL returns `16`

#### Scenario: each loop works at REPL
- **WHEN** a client sends `(def a @[]) (each x [1 2 3] (array/push a (* x 2))) a`
- **THEN** the REPL returns `@[2 4 6]`

#### Scenario: loop macro works at REPL
- **WHEN** a client sends `(loop [x :range [0 5]] (print x))`
- **THEN** the REPL prints `0` through `4`

#### Scenario: case macro works at REPL
- **WHEN** a client sends `(case 2 1 "one" 2 "two" "other")`
- **THEN** the REPL returns `"two"`

#### Scenario: let binding works at REPL
- **WHEN** a client sends `(let [a 10 b 20] (+ a b))`
- **THEN** the REPL returns `30`

#### Scenario: try/catch works at REPL
- **WHEN** a client sends `(try (error "fail") ([e] (string "caught: " e)))`
- **THEN** the REPL returns `"caught: fail"`

### Requirement: Loaded before rojcad boot code

The upstream macros SHALL be loaded into the Janet environment before rojcad's `boot.janet` executes, so that rojcad boot code may optionally use them.

#### Scenario: Macro available during boot.janet execution
- **WHEN** rojcad's `boot.janet` contains a form that uses `each`, `for`, `defn`, or other upstream macros
- **THEN** the form compiles and executes without error

### Requirement: No breaking changes to existing rojcad functionality

The addition of upstream macros SHALL NOT break any existing rojcad CAD function, REPL behavior, or startup sequence.

#### Scenario: CAD functions still work after loading upstream macros
- **WHEN** a client connects to the REPL
- **THEN** all CAD functions (`box`, `sphere`, `cylinder`, `torus`, `cut`, `common`, `fuse`, `translate`, `rotate`, `scale`, `mirror`, `extrude`, `revolve`, `hide`, `show`, `purge`, etc.) are still available and functional

#### Scenario: rojcad doc function overrides upstream doc
- **WHEN** a client sends `(doc box)`
- **THEN** the REPL returns rojcad's documentation string for `box`, not the upstream `doc` macro's behavior

### Requirement: Source file is the upstream boot.janet verbatim

The embedded source file SHALL be the exact upstream Janet `src/boot/boot.janet`, unmodified, corresponding to the same Janet version as the vendored `vendor/janetconf.h`.

#### Scenario: Source matches upstream
- **WHEN** comparing the embedded `upstream.janet` against the upstream Janet repository's `src/boot/boot.janet` at the matching version tag
- **THEN** the files are identical
