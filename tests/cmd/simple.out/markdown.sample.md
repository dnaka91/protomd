# markdown.sample

## SimpleService

The simplest server.

### Methods {#simpleservice-methods}

- [Call](#simpleservice-call)

---

#### Call {#simpleservice-call}

Request type: `unary`

Call it!

##### Input {#simpleservice-call-input}

This is a simple message.

```proto
message Simple {
  // A single integer.
  uint32 value = 1;
}
```

##### Output {#simpleservice-call-output}

This is a simple message.

```proto
message Simple {
  // A single integer.
  uint32 value = 1;
}
```
