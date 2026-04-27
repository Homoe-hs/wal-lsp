# 2. Waveform Handling

## load

```
(load file id?) ↦ ()
   file : string?
     id : symbol?
```

Loads waveform from `file` and registers it in the WAL kernel with `id`.
If no `id` is given, WAL automatically selects ids using the scheme `t0`, `t1`, ...

## unload

```
(unload id) ↦ ()
  id : symbol?
```

Removes the waveform specified by id from the WAL kernel.

## step

```
(step id amount) ↦ (boolean?)
      id : symbol?
  amount : int?
```

Step trace id by amount. If no id is provided all traces will be stepped.
Returns `#f` if the end of any loaded trace is reached.

## alias / unalias

```
(alias name signal) ↦ ()
(unalias name) ↦ ()
```

Introduces or removes an alias for `signal` such that it can be referenced
using `name`. Aliases are compatible with groups and scopes.

## whenever

```
(whenever cond body+)
    cond : WAL expression
    body : WAL expression
```

Evaluates the `body` expressions on each waveform index at which `cond`
evaluates to true. Returns the value of the last body expression evaluated
at the last matching index.

## find

```
(find cond) ↦ (list?)
    cond : WAL expression
```

Returns a list containing all indices at which `cond` evaluates to true.

## count

```
(count cond) ↦ (int?)
    cond : WAL expression
```

Returns the number of indices at which `cond` evaluates to true.

## timeframe

```
(timeframe body+)
    body: WAL expression
```

Stores the current `INDEX` of every loaded trace before the evaluation of
`body` and restores those indices after `body` is evaluated.

```wal
(print INDEX)
(timeframe
  (while (! ready) (step))
  (print INDEX))
(print INDEX)
```
