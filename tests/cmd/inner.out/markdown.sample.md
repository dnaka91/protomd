# markdown.sample

## SimpleService

The simplest server.

### Methods

- [Call](#call)

---

#### Call

Request type: `unary`

Call it!

##### Input

This is a simple message.

```proto
message Simple {
  // A single integer.
  uint32 value = 1;
  Other other = 2;
}
```

Message referenced in `Simple`.

```proto
message Other {}
```

##### Output

This is a simple message.

```proto
message Simple {
  // A single integer.
  uint32 value = 1;
  Other other = 2;
}
```

Message referenced in `Simple`.

```proto
message Other {}
```
