## bailout-canary

An extremely simple websocket client which tests the functionality of the [Bailout!](https://bailoutapp.io/) API server.

Unexpected conditions are simply raised as errors which eventually bubble up to the Lambda's CloudWatch error metric, in lieu of an actual testing framework (todo?).

There is absolutely no reason for this to be written in Rust except as a first chance to dip my toes in the water. Don't judge; it works.

### Building for Lambda

```
TARGET_CC=x86_64-linux-musl-gcc cargo build --release --target x86_64-unknown-linux-musl
cp target/x86_64-unknown-linux-musl/release/bailout-canary ./bootstrap
zip lambda.zip bootstrap
```