# Error View

Boundra provides one framework-neutral browser error view from `boundra/ui`.
It renders normalized diagnostics only and never reads contract inputs, results,
schema-provider errors, or causes.

## Vite

Vite applications normally need only the plugin:

```ts
import { boundra } from "boundra/vite";
import { defineConfig } from "vite";

export default defineConfig({
  plugins: [boundra()],
});
```

During `vite serve`, Boundra installs the view, captures unhandled
`BoundraRuntimeError` values, and publishes boundary diagnostics from the CLI.
It is not injected into production builds.

## Next.js and Other Browser Hosts

Next.js does not run the Vite plugin. Install the view from a Client Component
and explicitly forward a normalized Boundra error:

```tsx
"use client";

import { useEffect } from "react";
import { createBoundraErrorView } from "boundra/ui";
import type { BoundraRuntimeErrorJson } from "boundra";

export const BoundraErrorView = ({
  error,
}: {
  error?: BoundraRuntimeErrorJson;
}) => {
  useEffect(() => {
    if (!error || process.env.NODE_ENV === "production") return;

    const view = createBoundraErrorView();
    view.reportRuntime(error);
    return () => view.dispose();
  }, [error]);

  return null;
};
```

Use `error.toJSON()` before crossing a server/client boundary. Do not send the
original error, cause, request, response, input, or result to the browser.

## Handled Browser Errors

Unhandled browser errors are captured automatically only by the Vite adapter.
For a handled error, keep one view instance and report it directly:

```ts
import { BoundraRuntimeError } from "boundra";
import { createBoundraErrorView } from "boundra/ui";

const view = createBoundraErrorView();

try {
  await loadProjects();
} catch (error) {
  if (error instanceof BoundraRuntimeError) {
    view.reportRuntime(error.toJSON());
  }
}
```

The returned controller supports:

- `reportRuntime(error)` to replace the runtime diagnostics
- `reportBoundary(payload)` to replace boundary diagnostics
- `clear("runtime" | "boundary")` to clear one source
- `dispose()` to remove the view and release event handlers

Boundary editor navigation is optional. Vite configures it automatically;
other hosts can pass `openInEditor(location)` when creating the view.
