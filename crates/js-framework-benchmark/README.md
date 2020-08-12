# cope-example-web-counter

Dev workflow:

- Run:

  ```sh
  cargo watch -i pkg -s \
    'wasm-pack build --dev --target web crates/js-framework-benchmark/ --out-name index'
  ```

- Open `index.html`
