# Design Abstract

## Assembly

An extremely high-level representation of the program's execution. Assembly is entirely untyped and composed of just a limited set of primitive statements and structures:

- mod
- const
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

@hello_world = const _.types.string.new "Hello world!"

fn main() {
  message := @hello_world
  call _.std.print(message)

  # This is a shorthand for:
  #   @int0 = const _.types.int.from_string 0
  #   x0 := @int0
  #   return x0
  return const _.types.int.from_string 0
}
```

You may notice that there are no actual primitives values in the assembly. This is by design.

The only values even marginally like primitives are those constructed by calls to builtin value constructors in `const` statements. It might look like these constructors (such as `_.types.int.from_string`) receive primitive values, but that is actually a syntactical shorthand of the assembly language. All constructors receive builtin string values, and it is the responsibility of the constructor to return the correct new value based upon that input string.

### Commands

#### `const`

Constant definitions are expanded into calls to a constant constructor function. This function must guarantee its idempotence and that it will produce a new value.

The function may receive zero (0) or one (1) arguments. If it receives an argument that argument will *always* be a builtin string value.

For example, the following constant declaration in assembly:

```
@hello_world = const _.types.string.new "Hello world!"
```

Will be expanded at load-time into a `call` to `_.types.string.new`. The return of that call will be placed in `@hello_world`.
