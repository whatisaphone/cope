# cope-example-web-counter

Dev workflow:

- Run:

  ```sh
  cargo watch -i pkg -s \
    'wasm-pack build --dev --target web crates/example-web-todo/ --out-name index'
  ```

- Open `index.html`
