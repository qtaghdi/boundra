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
- runs boundary checks at startup and after source updates
- clears resolved diagnostics automatically

The overlay is not injected during production builds. Use `cliPath` only in
controlled development or tests when the native CLI is not resolved through
the npm launcher.
