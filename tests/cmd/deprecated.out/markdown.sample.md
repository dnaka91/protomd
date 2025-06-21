# markdown.sample

## SimpleService

The simplest server.

**This service is deprecated**

**The definition file of this service is deprecated**

### Methods {#simpleservice-methods}

- [Call](#simpleservice-call)

---

#### Call {#simpleservice-call}

**This method is deprecated**

Request type: `unary`

Call it!

##### Input {#simpleservice-call-input}

This is a simple message.

**This message is deprecated**

```proto
message Simple {
  option deprecated = true;
  // A single integer.
  uint32 value = 1 [deprecated = true];
  Other other = 2;
}
```

Message referenced in `Simple`.

**This message is deprecated**

```proto
message Other {
  option deprecated = true;
}
```

##### Output {#simpleservice-call-output}

This is a simple message.

**This message is deprecated**

```proto
message Simple {
  option deprecated = true;
  // A single integer.
  uint32 value = 1 [deprecated = true];
  Other other = 2;
}
```

Message referenced in `Simple`.

**This message is deprecated**

```proto
message Other {
  option deprecated = true;
}
```
