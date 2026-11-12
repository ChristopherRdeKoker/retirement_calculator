# Retirement calculator

A small retirement calculator for South Africans. Enter your age, savings and expectations, and it shows how long your savings last, what your pot is worth at retirement (in future rand and in today's money) and whether your desired income is sustainable. Up to three named scenarios can be saved in the browser and compared side by side. Nothing leaves the browser.

## Stack

- Rust, compiled to WebAssembly
- [Leptos](https://leptos.dev) 0.8 (client-side rendering)
- [Trunk](https://trunkrs.dev) for building and bundling
- Hand-built inline SVG chart, plain CSS with light and dark themes, `localStorage` for persistence

## Layout

- `src/calc/`: the maths. `inputs.rs` holds the inputs and their validation, `projection.rs` the month-by-month simulation and the sustainable-income search. No web dependencies.
- `src/format.rs`: rand formatting.
- `src/form.rs`: the input fields, parsing what people type, and the form state. `src/form/deposits.rs` reads the deposit rows.
- `src/outcome.rs`: a projection plus the sustainable income, and the "lasts until" wording.
- `src/scenarios.rs`: saved-scenario rules (naming, updating, the limit of three).
- `src/store.rs`: reading and writing `localStorage`.
- `src/components/`: the Leptos components. `src/app.rs` wires them together.
- `src/chart.rs` and `src/chart/`: the SVG chart component, its geometry and its drawing layers.

## Model

- Returns compound monthly (annual rate / 12).
- Contributions are paid at the end of each month and rise once a year.
- A lump-sum deposit for year N (year 1 starts at your current age) is added at the start of that year, before its first month of growth and contributions, so it earns the whole year's return. Deposits are in rand as paid, not adjusted for inflation, and must fall before retirement.
- The desired income is given in today's money, inflated to the retirement date, then raised with inflation once a year.
- Savings run out in the first month the balance cannot cover the withdrawal. The projection stops at age 100.
- Tax and fees are not modelled.

## Run and test

Requires Rust with the `wasm32-unknown-unknown` target and Trunk 0.22.

```sh
cargo test
trunk serve
RUSTFLAGS="--remap-path-prefix=$HOME=~" trunk build --release --public-url /apps/retirement/
```

`trunk serve` runs a local development server with live reload; the app is served under the `public_url` set in `Trunk.toml`.

The browser tests in `tests/e2e/` need Node and Playwright with Chromium. Build the release version first, then serve `dist/` with the production CSP and run them:

```sh
python3 tests/e2e/serve.py dist 8731 &
PLAYWRIGHT_MODULE=/path/to/playwright/index.mjs node tests/e2e/e2e.mjs
```

`PLAYWRIGHT_MODULE` defaults to `playwright`, `PORT` to 8731, and screenshots go to `tests/e2e/screenshots/` unless `SHOTS_DIR` is set (field-help screenshots go in its `help/` folder).

## Deploying

`dist/` is fully static and expects to live under `/apps/retirement/`. Serve `.wasm` files as `application/wasm`.

The page needs no inline scripts or styles. It runs under this Content-Security-Policy:

```
default-src 'none'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self'; img-src 'self'; connect-src 'self'; base-uri 'self'; form-action 'none'; frame-ancestors 'none'
```

`'wasm-unsafe-eval'` is required to compile the WebAssembly module and `connect-src 'self'` lets the loader fetch it. File names are not content-hashed, so give `index.html`, `loader.js`, `retire-calc.js` and `retire-calc_bg.wasm` a short cache lifetime or revalidation.
