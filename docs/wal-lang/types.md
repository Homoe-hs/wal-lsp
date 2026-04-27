# 7. Types and Conversions

## atom?

```
(atom? x) ↦ (boolean?)
     x : WAL expression
```

Returns true if the argument is either a symbol, integer, boolean, or string.

## Type Predicates

```
(symbol? x) ↦ (boolean?)
(string? x) ↦ (boolean?)
   (int? x) ↦ (boolean?)
  (list? x) ↦ (boolean?)
       x : WAL expression
```

Predicate functions that return true if the argument is of the checked type.

## convert/bin

```
(convert/bin x width) ↦ (string?)
       x : (int?)
   width : (int?)
```

Converts an integer `x` to a binary string representation with a size of
`width` bits.
