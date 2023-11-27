## Frontend Receiving Format

A websocket connection should be made to ws://<endpoint>/ws/

Then, to view the current state of the orderbook, send a message with the format:

```
V <orderbook id>
```

This will return a message in the following format:

```
V: <price>:<qty>; <price>:<qty>; ... ;
```

Some simple edge cases to watch out for:
If the orderbook is empty, it will return V:
If the orderbook is full it will still end in a semicolon