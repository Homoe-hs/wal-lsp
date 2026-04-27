# WAL Programmer Manual v0.8.2

This is the complete WAL (Waveform Analysis Language) documentation,
fetched from [wal-lang.org/documentation](https://wal-lang.org/documentation).

## Table of Contents

1. [Core Language](core.md)
   - Arithmetic (+, -, *, /, **)
   - Logic and Comparisons (!, &&, ||, =, !=, >, <, >=, <=)
   - Program State (let, define, set!)
   - Functions (defun, fn, closures)
   - Control Flow (do, when, unless, if, cond, case)
   - Printing (print, printf)
   - Utility (eval-file, exit)

2. [Waveform Handling](waveforms.md)
   - load, unload, step, alias, unalias
   - whenever, find, count, timeframe

3. [Accessing Signals](signals.md)
   - get, slice, reval, @

4. [Groups and Scopes](groups.md)
   - groups, in-groups, resolve-group, #
   - in-scope, in-scopes, all-scopes

5. [Lists](lists.md)
   - list, first, second, last, rest, in
   - map, fold, zip, max, min, sum, average, length

6. [Arrays](arrays.md)
   - array, seta, geta, geta/default, dela, mapa

7. [Types and Conversions](types.md)
   - atom?, Type Predicates, convert/bin

---

Source: [Institute for Complex Systems](https://ics.jku.at/) | [GitHub](https://github.com/ics-jku/wal)
