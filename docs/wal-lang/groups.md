# 4. Groups and Scopes

Hardware designs often contain structural redundancy. WAL supports two ways
of writing generic code: **groups** and **scopes**.

## Groups

### groups

```
(groups posts*) ↦ (list?)
  posts : symbol?, string?
```

Returns all partial signal names `pre` for which `pre` + `post`
for every `post` in `posts` is a valid signal name.

For example, `(groups '("valid" "ready"))` would return groups like
`'("top.in_" "top.out_")` if `top.in_ready`, `top.in_valid`,
`top.out_ready`, and `top.out_valid` are valid signal names.

### in-groups

```
(in-groups groups expr) ↦ (list?)
groups : list?
  expr : WAL expression
```

Evaluates `expr` in every group in `groups`.
When an expression is evaluated in a group, the group is prepended to
every signal name that starts with `#`.

The variable **CG** is a special variable that returns the current group.

```wal
(in-groups '("top.in_" "top.out_")
  (print CG ":")
  (whenever (&& top.clk (! top.reset) #ready #valid)
    (print INDEX)))
```

### resolve-group

```
(resolve-group name) ↦ (int?)
  name : (symbol?)
```

Evaluates the signal `name` appended by `CG` and returns the signal value
at the current `INDEX`.

### # macro

```
#name ↦ (resolve-group name)
```

A shorthand for the `resolve-group` function. For example, `#valid` evaluated
in group `top.in_` would evaluate signal `top.in_valid`.

## Scopes

### in-scope

```
(in-scope scope body+)
  scope : symbol?
   body : WAL expression
```

Evaluates `body` in the given scope.

### in-scopes

```
(in-scopes scopes body+)
 scopes : list?
   body : WAL expression
```

Evaluates `body` in each scope in `scopes`.

### all-scopes

```
(all-scopes) ↦ (list?)
```

Returns all available scopes.
