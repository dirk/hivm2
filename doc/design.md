# Design Abstract

## Assembly

An extremely high-level representation of the program's execution. Assembly is entirely untyped and composed of just a limited set of primitive statements and structures:

- mod
- const
- local
- fn
- return
- call
- if
- elseif
- else
- while
- break

Below is a short hello world demonstrating some of the features of the assembly:

```ruby
# hello_world.hasm
mod hello_world

const @hello_world = _.std.string.new "Hello world!"

fn main() {
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

#### Constant globals

Constant globals are prefixed with a `@` and defined and assigned once at the beginning of the module.

```ruby
const @foo = _.std.string.new "bar"
```

#### Static variables

Static variables are prefixed with `$`, defined at the beginning of the module (initialized as a null value), and may then be assigned zero or more times in the module.

```ruby
static $foo
const @bar = _.std.string.new "Baz!"

fn entry() {
  $foo = @bar
}
```

#### Local variables

Local variables have no prefix. They must also have their slot allocated (on the stack frame) with `local` before being used, however the `:=` allocate-and-assign shorthand is provided for this common use case.

```ruby
fn foo() {
  # Explicit allocation then assignment
  local bar
  bar = @something

  # Allocate-and-assign shorthand
  baz := bar
}
```

### Commands

#### `const`

Constant definitions are expanded into calls to a constant constructor function. This function must guarantee its idempotence and that it will produce a new value.

The function may receive zero (0) or one (1) arguments. If it receives an argument that argument will *always* be a builtin string value.

For example, the following constant declaration in assembly:

```
const @hello_world = _.std.string.new "Hello world!"
```

Will be expanded at load-time into a `call` to `_.std.string.new`. The return of that call will be placed in `@hello_world`.

#### `fn`

Assembly provides both named and anonymous functions. Named functions may be defined at any level of the module but cannot capture any local variables (ie. no closures). Anonymous functions may be defined inside any other function and can capture local variables (ie. closures allowed).

```ruby
fn foo() {
  bar := @some_thing

  baz := fn(x) {
    bar = call _.std.string.concat(bar, x)
  }

  # Use the `_.fn.call` builtin to call a function value
  call _.std.fn.call(baz, @another_thing)

  return bar
}

# If @something = "foo" and @another_thing = "bar", then calling `foo` will
# return "foobar".
```

### Macros

Assembly provides an optional set of macros to make interfacing with the builtins easier. These can be enabled with the `macros builtins` keyword.

```ruby
macros builtins

# Without builtins macros
const @hello_world1 = _.std.string.new "Hello world!"
# With builtsin macros
const @hello_world2 = %string "Hello world!"

# Without
foo = call _.std.string.concat(bar, baz)
# With
foo = %string+ bar, baz
```
