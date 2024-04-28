---
title: CRUST - Barely working C Compiler/Interpreter
author: Ryan Hedgecock
sub_title: written in Rust
theme:
  name: terminal-dark
  override:
    code:
      theme_name: Solarized (dark)
---

# Overview

- Project Requirements
- Tooling and Design Processes
- Implementation Details
- Demo

<!-- end_slide -->

# Project Requirements

- Broken into two parts **_Compiler_** and **_Interpreter_**
- Compile C code into some intermediate representation
- Compiler must provide helpful error messages
- Compiler must attempt to recover from parsing errors
- Interpreter can load, and run the pre-compiled IR code

<!-- end_slide -->

# Tooling and Design Processes

## Rust - Language the compiler is written in

I found rust useful for this project because of its algebraic type system.

<!-- pause -->

what the heck is that...

<!-- end_slide -->

# Tooling and Design Processes (cont.)

### Algebraic Type Systems

Algebraic type systems are pretty much just fancy tagged unions supported by the compiler

```rust
enum MyCoolEnum {
    Nothing,
    Something,
    AnotherThing(OtherEnum)
    CoolThing {
        label: String,
        other: OtherEnum,
    }
}

enum OtherEnum {
    ...
}
```

<!-- end_slide -->

# Tooling and Design Processes (cont.)

### Parsing with **_chomsky_**

The compiler uses the **_chomsky_** library for parsing text into tokens, and for converting tokens into an AST.

**_chomsky_** is a library that exposes an API for more easily working and building a variation of PEGs.

PEGs are useful because they are deterministic through the ordering of the expressions.

<!-- end_slide -->

# Tooling and Design Processes (cont.)

#### Parsing Process

Building the AST for the program is done in two phases.

1. Converting the source code into tokens
2. Converting the tokens into an AST

After an AST is built by the compiler it is then saved to the disk in a _json_ like format

<!-- end_slide -->

# Tooling and Design Processes (cont.)

#### Why break down the process into phases?

By converting source code to tokens first, complexity can be stripped away from the beginning.

Building an AST is clearer because every token is known to be valid.

You dont need to worry about

- Whitespace
- Comments
- Invalid Tokens

<!-- end_slide -->

# Implementation Details

## Tokens

```rust
pub enum Token {
    Return,
    Struct,
    Op(char),
    Ident(String),
    Ctrl(char),
    Num(String),
}
```

### Parsing Example

```rust
// PEG parsing api
let comment = just("//")
                .then(take_until(just('\n')))
                .padded();
```

<!-- end_slide -->

# Implementation Details (cont.)

## Building an AST

```rust
// build a ast parser for a function call
let call = parse_ident()
    .then(
        expr.clone()
            .separated_by(just(Token::Ctrl(',')))
            .delimited_by(/*omitted for brevity*/)
            .recover_with(nested_delimiters(
                Token::Ctrl('('),
                Token::Ctrl(')'),
            )),
    );
```

<!-- end_slide -->

# Implementation Details (cont.)

## Saving IR code

```json
{
  "defs": [
    {
      "Func": {
        "name": "main",
        "params": [],
        "ret": "int",
        "body": [
          {
            "Return": {
              "Add": [{ "Int": 1 }, { "Int": 1 }]
            }
          }
        ]
      }
    }
  ]
}
```

<!-- end_slide -->

# Implementation Details (cont.)

## Running IR code

```rust
fn eval(&self, v: ..., f: ...) -> i32 {
  match self {
    Self::Int(value) => *value as i32,
    ...
    Self::Neg(expr) => -expr.eval(vars, funcs),
    Self::Add(l, r) => l.eval(v, f) + r.eval(v, f),
    Self::Sub(l, r) => l.eval(v, f) - r.eval(v, f),
    Self::Mul(l, r) => l.eval(v, f) * r.eval(v, f),
    Self::Div(l, r) => l.eval(v, f) / r.eval(v, f),
    ...
  }
}
```

<!-- end_slide -->

<!-- column_layout: [1, 1, 1] -->
<!-- column: 1 -->

# DEMO TIME
