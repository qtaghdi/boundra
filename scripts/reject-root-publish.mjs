console.error(
  [
    "BR-PUBLISH-001: The Boundra workspace root is not a publishable package.",
    "Publish packages/runtime as @boundra/runtime through the release workflow instead.",
  ].join("\n"),
);

process.exitCode = 1;
