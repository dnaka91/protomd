# markdown.sample

## OtherService

### Methods {#otherservice-methods}

- [Other](#otherservice-other)

---

#### Other {#otherservice-other}

Request type: `unary`

##### Input {#otherservice-other-input}

```proto
message Empty {}
```

##### Output {#otherservice-other-output}

```proto
message Empty {}
```

## SimpleService

The simplest server.

### Methods {#simpleservice-methods}

- [Call](#simpleservice-call)

---

#### Call {#simpleservice-call}

Request type: `unary`

Call it!

##### Input {#simpleservice-call-input}

Message from `types.proto`.

```proto
message Simple {
  // A single integer.
  uint32 value = 1;
}
```

##### Output {#simpleservice-call-output}

Message from `types.proto`.

```proto
message Simple {
  // A single integer.
  uint32 value = 1;
}
```
