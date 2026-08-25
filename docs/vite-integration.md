# Vite Integration

```ts
import react from "@vitejs/plugin-react";
import { boundra } from "boundra/vite";
import { defineConfig } from "vite";

export default defineConfig({
  plugins: [boundra(), react()],
});
```

During development the plugin:

- renders unhandled `BoundraRuntimeError` details in a browser overlay
- shows all normalized validation issues with contract, phase, code, and field
  paths
- runs boundary checks at startup and after source updates
- shows all boundary violations with file/line locations and imports
- copies a safe text summary for issue reports without including input, result,
  or cause values
- keeps runtime and boundary state independent so a clean boundary scan does
  not hide an active runtime error
- reacts to TypeScript, JavaScript, and Svelte source updates
- clears resolved boundary diagnostics automatically; runtime diagnostics can
  be dismissed with the close button or `Escape`

The overlay is not injected during production builds. Use `cliPath` only in
controlled development or tests when the native CLI is not resolved through
the npm launcher.
