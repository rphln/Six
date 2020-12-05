# Six

Six is structured in a number of conceptual layers.

## Mode

At the heart of the editor, lies the `Mode` state machine, which interprets
micro-operations based on the current state and applies operations on the editor
through a `Context`.

The state machine has no knowledge of user-facing inputs, such as keyboard or
mouse events. Instead, it receives a series of atomic operations and handles
them sequentially until they are exhausted or one of them halts the machine.

As a data structure, `Mode` is fairly simple: it amounts to an enumeration
storing a small amount of ephemeral state and a huge `match` expression.

In the current implementation, every operation happens inside the core.
Multi-step operations are callback based.

## State

The bare minimum of state that `Mode` needs to function is a text buffer and a
cursor.

## Context

Between the `Mode` state machine and the higher level `Editor`, lies the
`Context` communication channel.

Its purpose is to provide access to the buffer, cursor and other auxiliary data
structures (such as the Lua interpreter and the key map) from the `Editor` to
the `Mode`.

## Editor

The highest non-interface layer of Six is the `Editor`, which holds all the
non-ephemeral state, such as the cursor, the buffer, the key map and more.

The key map is the structure responsible for converting user input, such as
series of key presses, into batches of micro-operations.
