# 6. Arrays

Arrays in WAL are a hashmap data structure.

## array

```
(array (id expr)*) ↦ (array?)
  id : WAL value
  expr : WAL expression
```

Constructs an array initialized with key-value pairs.
Keys are always stored as strings. Arrays are printed in curly braces `{}`.

```wal
>-> (array)
{}
>-> (array ['x 10] ['y 20])
{("x" 10) ("y" 20)}
>-> (array [5 5])
{("5" 5)}
```

## seta

```
(seta array key value) ↦ WAL value
  array : (array?)
  key : WAL value
  value: WAL expression
```

Evaluates `key`, converts to string, and inserts/updates `value` in `array`.

```wal
>-> (seta (array) 'x 10)
{("x" 10)}
>-> (seta (array ['x 10]) 'y 20)
{("x" 10) ("y" 20)}
>-> (define some-array (array))
{}
>-> (define data '("test" "data"))
("test")
>-> (seta some-array 0 data)
{("0" ("test" "data"))}
```

## geta

```
(geta array key) ↦ WAL value
  array : (array?)
  key : WAL value
```

Evaluates `key`, converts to string, and returns the value at `key`.

```wal
>-> (geta (array ['x 10]) 'x)
10
>-> (define i 5)
5
>-> (geta (array ['i 0] [5 "test"]) i)
"test"
```

## geta/default

```
(geta/default array default key) ↦ WAL value
  array : (array?)
  default : WAL expression
  key : WAL value
```

Returns value at `key` from `array` if present, else evaluates and returns `default`.

```wal
>-> (geta/default (array ['x 10]) 5 'x)
10
>-> (geta/default (array ['x 10]) 5 'y)
5
```

## dela

```
(dela array key) ↦ WAL value
  array : (array?)
  key : WAL value
```

Removes the value at `key` from `array`.

```wal
>-> (dela (array ['x 10] ['y 20]) 'x)
{["y" 20]}
```

## mapa

```
(mapa f array) ↦ (list?)
  f : (fn?) (fn [key value] ...)
  array : (array?)
```

Applies function `f` to every (key value) pair in `array`.
`f` must take exactly two parameters.

```wal
>-> (mapa (fn [k v] (list k v)) (array ['x 10] ['y 20]))
(("x" 10) ("y" 20))
```
