import defaultExport from "./default";
import { named, type Typed } from "./named";
import type { Shape } from "./types";
import "./side-effect";
export { value } from "./exported";
export type { Contract } from "./contract";
export * from "./star";
export * as namespace from "./namespace";
import data from "./json" with { type: "json" };
import { localized } from "./도메인";

import {
  first,
  second,
} from "./multiline";
