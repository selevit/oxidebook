# Oxidebook

![Continuous integration](https://github.com/selevit/oxidebook/workflows/Continuous%20integration/badge.svg)

A toy implementation of a trading engine in Rust.

- The core stores anything in memory
- The state can be recovered by replaying inbox events
- Uses event sourcing approach
- RabbitMQ for message transport

Now you can:

- Place exchange orders and they matches with each other
- Cancel orders

How to run:

```
rabbitmq-server
```

And then:

```
cargo run
```

Then you can you REST API (take a look at its structure in the code :)
