import {
  createBoundraErrorView,
  formatBoundraBoundarySummary,
  formatBoundraRuntimeSummary,
} from "../src/index";

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message);
}

const runtimeSummary = formatBoundraRuntimeSummary({
  name: "BoundraRuntimeError",
  code: "RUNTIME-002",
  contract: "list-projects",
  phase: "result",
  message: "contract result rejected",
  suggestion: "return contract-ready projects",
  issues: [
    { code: "invalid_type", path: ["projects", 0, "id"], message: "expected string" },
    { code: "too_big", path: ["projects"], message: "too many projects" },
  ],
});
assert(
  runtimeSummary.includes("Boundra runtime diagnostics (2)"),
  "runtime view should count every issue",
);
assert(
  runtimeSummary.includes("projects[0].id"),
  "runtime view should preserve nested paths",
);
assert(
  runtimeSummary.includes("too many projects"),
  "runtime view should include later issues",
);

const boundarySummary = formatBoundraBoundarySummary({
  source: "boundary",
  diagnostics: [
    {
      rule: "BR-005",
      file: "apps/web/src/main.ts",
      line: 1,
      import: "@domains/tasks/server/internal/store",
      message: "app imported a domain internal path",
      suggestion: "import the domain public API",
    },
    {
      rule: "BR-001",
      file: "domains/tasks/client/view.ts",
      line: 4,
      import: "../server/internal/store",
      message: "client imported server code",
      suggestion: "move the contract to shared",
    },
  ],
});
assert(
  boundarySummary.includes("Boundra boundary diagnostics (2)"),
  "boundary view should count every violation",
);
assert(
  boundarySummary.includes("BR-001"),
  "boundary view should include later violations",
);

let serverError: unknown;
try {
  createBoundraErrorView();
} catch (error) {
  serverError = error;
}
assert(
  serverError instanceof Error
    && serverError.message.includes("requires a browser document"),
  "error view should fail clearly when instantiated during server rendering",
);

console.log("error-view-test: OK");
