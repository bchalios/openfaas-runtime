# OpenFaas Rust runtime

**DISCLAIMER**: This is in alpha state and should not be used in
production.

This is a runtime build on top of [hyper](https://docs.rs/hyper/0.14.14/hyper/index.html)
for writing Rust functions for OpenFaas.

It hides away the complexity of defining your own service for handling HTTP
requests, while allowing the development of OpenFaas functions that can
share state across invocations.

It has been heavily based on the efforts of the existing
[OpenFaas Rust template](https://github.com/openfaas-incubator/rust-http-template)
and the AWS lambda Rust [runtime](https://github.com/awslabs/aws-lambda-rust-runtime).
