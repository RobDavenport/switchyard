# switchyard demo-wasm

Browser showcase for `switchyard`, inspired by the "death of tick" style of encounter orchestration.

## Build WASM package

```bash
wasm-pack build demo-wasm --target web --release --out-dir www/pkg
```

## Serve showcase

```bash
cd demo-wasm/www
python -m http.server 8080
```

Then open `http://localhost:8080` and verify the status line reports `WASM core ready`.
