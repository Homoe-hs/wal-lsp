# 5. Lists

## list

```
(list expr*) ↦ (list?)
    expr : WAL expression
```

Returns a list whose elements are the evaluated expressions in order.

## first / second / last

```
(first xs)
(second xs)
(last xs)
      xs : (list?)
```

Returns the first, second, or last element of list `xs`.

## rest

```
(rest xs)
      xs : (list?)
```

Returns a list containing all but the first element of `xs`.

## in

```
(in x xs)
        x : WAL Expression
       xs : (list?)
```

Returns true if `x` is an element in `xs`.

## min / max

```
(min xs)
(max xs)
       xs : (list?)
```

Returns the smallest or largest element in `xs`.

## sum

```
(sum xs)
       xs : (list?)
```

Returns the sum of all elements in `xs`.

## average

```
(average xs)
       xs : (list?)
```

Returns the average of all elements in `xs`.

## length

```
(length xs)
       xs : (list?)
```

Returns the number of elements in `xs`.

## map

```
(map f xs)
        f : Function (fn [x] ...)
       xs : (list?)
```

Returns a list containing `(f x)` for each `x` in `xs`.

## fold

```
(fold f init xs)
        f : Function (fn [acc x] ...)
     init : WAL expression
       xs : (list?)
```

Folds (reduces) `xs` using function `f` starting from `init`.

## zip

```
(zip xs ys)
       xs : (list?)
       ys : (list?)
```

Returns a list of pairs from `xs` and `ys`.
