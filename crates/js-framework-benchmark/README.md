# js-framework-benchmark

## Dev workflow:

- Run:

  ```sh
  cargo watch -i pkg -s \
    'wasm-pack build --dev --target web crates/js-framework-benchmark/ --out-name index'
  ```

- Open `index.html`

## Running the benchmark

- In a copy of the `js-framework-benchmark` repo, symlink `frameworks/keyed/cope` to this repo's `crates/js-framework-benchmark`

- Build

  ```sh
  # In this repo
  wasm-pack build --release --target web crates/js-framework-benchmark/ --out-name index
  ```

- Run the benchmark

  ```sh
  # In the js-framework-benchmark fork
  npm start
  ```

  and in another terminal:

  ```
  rm -r webdriver-ts/results
  time for framework in keyed/{vanillajs,wasm-bindgen,react,solid,cope}; do
      npm run bench "$framework" --exitOnError
  done
  npm run results > /tmp/jsfbr.log

  # or

  rm -r webdriver-ts/results
  time for framework in vanillajs-keyed wasm-bindgen react-v16.8.6-keyed solid-v cope; do
      npm run bench -- --framework "$framework" --count 50 --exitOnError
  done
  npm run results > /tmp/jsfbr.log
  ```
