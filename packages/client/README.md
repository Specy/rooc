# ROOC web client

The ROOC web client provides the browser editor, documentation site, and model runner.

## Development

Install dependencies in both `packages/ts-lib` and `packages/client`:

```bash
npm install
```

Start the development server from `packages/client`:

```bash
npm run dev
```

## Checks and build

```bash
npm run check
npm run lint
npm run build
```

`npm run build` generates the TypeScript API documentation before building the site.
