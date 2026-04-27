# 3. Accessing Signals

Signals from a loaded waveform can be read by just using their name.
Their value depends on the current INDEX, which is the pointer into
the currently loaded waveform.

```wal
>-> (load "trace.vcd")
t0(0) >-> SIGNALS
("tb.a", "tb.b")
t0(0) >-> INDEX
0
t0(0) >-> tb.a
5
t0(0) >-> tb.a@1
6
t0(0) >-> INDEX
0
```

## get

```
(get signal) ↦ (int?)
  signal : symbol? or string?
```

Returns the signal value of the signal specified by argument name.

## slice

```
(slice signal upper lower) ↦ (int?)
  signal : symbol?
   upper : int?
   lower : int?
```

Returns the bits or list elements from `upper` to `lower`.
List slicing follows Python's list slicing semantics.

## reval

```
(reval expr offset) ↦ (int?)
     expr : WAL expression
   offset : int?
```

Evaluates `expr` at current index + `offset`.

## @ macro

```
expr@off ↦ (reval expr offset)
```

The `@` macro is transformed into a call to `reval`.

```wal
>-> INDEX
5
>-> (reval INDEX -1)
4
>-> INDEX@-1
4
>-> INDEX@(+ 2 2)
9
```
