import { readdir, readFile } from "node:fs/promises";
import { resolve } from "node:path";

const repositoryRoot = resolve(import.meta.dirname, "..");
const releasesDirectory = resolve(repositoryRoot, "docs/releases");
const requestedVersion = process.argv[2]?.replace(/^v/, "").trim();
const requiredSections = [
  "Release Summary",
  "Highlights",
  "Installation",
  "Breaking Changes",
  "Known Limitations",
  "Verification",
];

function noteFileName(version) {
  if (!/^\d+\.\d+\.\d+$/.test(version)) {
    throw new Error(`invalid release version '${version}'`);
  }
  return `v${version.replaceAll(".", "-")}.md`;
}

function versionFromFileName(fileName) {
  const match = fileName.match(/^v(\d+)-(\d+)-(\d+)\.md$/);
  if (!match) {
    throw new Error(
      `release note '${fileName}' must use v<major>-<minor>-<patch>.md`,
    );
  }
  return `${match[1]}.${match[2]}.${match[3]}`;
}

async function validateReleaseNote(fileName, expectedVersion) {
  const version = expectedVersion ?? versionFromFileName(fileName);
  const content = await readFile(resolve(releasesDirectory, fileName), "utf8");
  const title = `# Boundra v${version}`;

  if (!content.startsWith(`${title}\n`)) {
    throw new Error(`${fileName} must start with '${title}'`);
  }

  let previousIndex = content.indexOf(title);
  for (const section of requiredSections) {
    const heading = `## ${section}`;
    const index = content.indexOf(heading);
    if (index === -1) {
      throw new Error(`${fileName} is missing '${heading}'`);
    }
    if (index <= previousIndex) {
      throw new Error(`${fileName} has '${heading}' out of order`);
    }
    previousIndex = index;
  }

  if (/\b(?:TBD|TODO)\b|작성 예정/i.test(content)) {
    throw new Error(`${fileName} contains an unfinished placeholder`);
  }
}

if (requestedVersion) {
  const fileName = noteFileName(requestedVersion);
  await validateReleaseNote(fileName, requestedVersion);
  console.log(`release-notes: OK (${fileName})`);
} else {
  const files = (await readdir(releasesDirectory))
    .filter((fileName) => fileName.endsWith(".md"))
    .sort();
  if (files.length === 0) {
    throw new Error("at least one release note is required");
  }
  for (const fileName of files) {
    await validateReleaseNote(fileName);
  }
  console.log(`release-notes: OK (${files.length} files)`);
}
