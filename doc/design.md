# Design Abstract

## Assembly

A high-level representation of the program's execution. Assembly is entirely untyped and composed of just a small set of primitive statements and structures:

- mod
- extern
- const
- static
- local
- defn
- fn
- return
- call
- test
- if
- then
- else
- while
- do
- break
- macros

Below is a short hello world demonstrating some of the features of the assembly:

```ruby
# hello_world.hasm
mod hello_world

const @hello_world = _.std.string.new "Hello world!"

defn main() {
  message := @hello_world
  call _.std.print(message)

  # This is a shorthand for:
  #   const @int0 = _.std.int.from_string 0
  #   x0 := @int0
  #   return x0
  return const _.std.int.from_string 0
}
```

You may notice that there are no actual primitives values in the assembly. This is by design.

The only values even marginally like primitives are those constructed by calls to builtin value constructors in `const` statements. It might look like these constructors (such as `_.std.int.from_string`) receive primitive values, but that is actually a syntactical shorthand of the assembly language. All constructors receive builtin string values, and it is the responsibility of the constructor to return the correct new value based upon that input string.

### Storage

The assembly provides for two kinds of storage: constant globals, static variables, and local variables.

##### Constant globals

Constant globals are prefixed with a `@` and defined and assigned once at the beginning of the module.

```ruby
const @foo = _.std.string.new "bar"
```

##### Static variables

Static variables are prefixed with `$`, defined at the beginning of the module (initialized as a null value), and may then be assigned zero or more times in the module.

```ruby
static $foo
const @bar = _.std.string.new "Baz!"

defn entry() {
  $foo = @bar
}
```

##### Local variables

Local variables have no prefix. They must also have their slot allocated (on the stack frame) with `local` before being used, however the `:=` allocate-and-assign shorthand is provided for this common use case.

```ruby
defn foo() {
  # Explicit allocation then assignment
  local bar
  bar = @something

  # Allocate-and-assign shorthand
  baz := bar
}
```

### Statements

#### `const`

Constant definitions are expanded into calls to a constant constructor function. This function must guarantee its idempotence and that it will produce a new value.

The function may receive zero (0) or one (1) arguments. If it receives an argument that argument will *always* be a builtin string value.

For example, the following constant declaration in assembly:

```
const @hello_world = _.std.string.new "Hello world!"
```

Will be expanded at load-time into a `call` to `_.std.string.new`. The return of that call will be placed in `@hello_world`.

#### `defn`

Assembly provides both named and anonymous functions. Named functions (`defn`) may be defined at any level of the module but cannot capture any local variables (ie. no closures). Anonymous functions (`fn`) may be defined inside any other function and can capture local variables (ie. closures allowed).

```ruby
defn foo() {
  bar := @some_thing

  baz := fn(x) {
    bar = call _.std.string.concat(bar, x)
  }

  # Use the `_.std.fn.call` builtin to call a function value
  call _.std.fn.call(baz, @another_thing)

  return bar
}

# If @something = "foo" and @another_thing = "bar", then calling `foo` will
# return "foobar".
```

#### `return`

Returns from the current function; accepts a single storage argument for a value to be returned. The formal syntax is:

```
"return" ARGUMENT?
```

If no argument is specified then the null value will be returned. The single argument can be any kind of storage (constant, static, or local).

#### `call`

Call is used to invoke a function by identifier. It has the syntax:

```
"call" IDENTIFIER "(" ( ARGUMENT ( "," ARGUMENT )* )? ")"
```

The identifier may be one of two things:

1. Another function defined in the current module.
2. The fully-qualified identifier of a function in another module.

Arguments may be any kind of storage (constant, static, or local). Some examples are as follows:

```ruby
# Calling a module-local function with constant and static values as arguments
call foo(@bar, $baz)

# Calling an externally-defined function with a local value
call foo.bar.baz(bop)
```

#### `test`

Test is the final statement in a condition basic block. It takes any storage as an argument and yields that to the control structure for it to use to determine control flow.

```ruby
if {
  x := call y()
  test x
} then {
  ...
}
```

#### `if`, `then`, and `else`

Standard conditional branching control structure. `if` requires a basic block ending with a `test` statement and must be followed by a `then` statement.

```ruby
defn foo() {
  bar := ...

  if { test bar } then {
    ...
  } else {
    baz := ...

    if { test baz } then {
      ...
    } else {
      ...
    }
  }
}
```

#### `while`, `do`, and `break`

While will repeat while the condition is not the null value. Break will immediately jump to the position immediately after the nearest while. The condition of while must be a block ending with a `test` statement.

```ruby
defn foo() {
  bar := ...
  baz := ...

  while { test bar } do {
    if { test baz } then {
      break
    }
  }

  # The inverted do-while is also permitted
  do {
    ...
  } while { test bar }
}
```

#### `extern`

Define an external module to be used by the current module.

```ruby
mod a
extern b

defn foo() {
  bar := call b.bar()

  return bar
}
```

### Values

Certain patterns and statements may also function as values in assignment, `call`, `return`, and `test` statements. These statements are:

- Names of const, static, or local storage (unless otherwise specified by the statement)
- Paths to external const or static storage
- Anonymous functions (`fn`)
- Call statements (`call`)

**Note**: Names, paths, and anonymous functions are *not* considered statements and as such may *only* appear as values.

### Macros

Assembly provides an optional set of macros to make interfacing with the builtins easier. These can be enabled with the `macros builtins` keyword.

```ruby
macros builtins

# Without builtins macros
const @hello_world1 = _.std.string.new "Hello world!"
# With builtins macros
const @hello_world2 = %string "Hello world!"

# Without
foo = call _.std.string.concat(bar, baz)
# With
foo = %string+ bar, baz
```
