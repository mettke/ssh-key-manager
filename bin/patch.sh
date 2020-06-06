#!/usr/bin/env bash
sed -i '/jsonwebtoken/s/^/# /g' core/common/Cargo.toml
sed -i '/reqwest/s/^/# /g' core/common/Cargo.toml
sed -i '/rustls/s/^/# /g' web/hyper/Cargo.toml
sed -i '/tokio-rustls/s/^/# /g' web/hyper/Cargo.toml
cargo patch
sed -i '/jsonwebtoken/s/^# //g' core/common/Cargo.toml
sed -i '/reqwest/s/^# //g' core/common/Cargo.toml
sed -i '/rustls/s/^# //g' web/hyper/Cargo.toml
sed -i '/tokio-rustls/s/^# //g' web/hyper/Cargo.toml
