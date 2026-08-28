const required = require("./required");
const lazy = import("./lazy");
const staticTemplate = import(`./template`);
const dynamicTemplate = import(`./${name}`);
const dynamicName = import(moduleName);
const pair = [require("./first"), require("./second")];
const lazyPair = Promise.all([import("./lazy-a"), import("./lazy-b")]);
const misleading = "require('./string') import('./string-lazy')";
// require("./comment");
/* import("./block-comment"); */
const unrelated = import(moduleName); const text = "./not-an-import";
const withOptions = import("./json", { with: { type: "json" } });

const multiline = import(
  "./multiline-lazy"
);
const multilineRequire = require(
  "./multiline-required"
);
const methodImport = loader.import("./method");
const methodRequire = loader.require("./method-require");
const namedRequire = myrequire("./named-require");
