<div align="center">

# Boundra

### Domain boundaries that fail before architecture drifts.

도메인 중심 TypeScript 애플리케이션을 위한 계약 runtime과 Rust 기반 정적 분석 CLI.

[![npm](https://img.shields.io/npm/v/boundra?color=315c45&label=npm)](https://www.npmjs.com/package/boundra)
[![CI](https://github.com/qtaghdi/boundra/actions/workflows/boundra.yml/badge.svg)](https://github.com/qtaghdi/boundra/actions/workflows/boundra.yml)
[![license](https://img.shields.io/badge/license-MIT-c8ed6b)](LICENSE)
[![node](https://img.shields.io/badge/node-%3E%3D20-315c45)](packages/runtime/package.json)

[Quick Start](#quick-start) · [Boundary Rules](#boundary-rules) · [Vite Overlay](#vite-development-overlay) · [Documentation](#documentation)

</div>

---

Boundra는 Next.js, Vite, ORM 또는 UI 프레임워크를 대체하지 않습니다. 애플리케이션의 **도메인 구조**, **공개 API**, **runtime 계약**이 코드베이스가 커져도 무너지지 않도록 강제합니다.

```txt
apps/web  ───────▶ domains/order/client/public
                          │
                 ┌────────┴────────┐
                 ▼                 ▼
              shared            server
             contracts         handlers
```

## Why Boundra?

| 문제 | Boundra가 제공하는 것 |
| --- | --- |
| client/server 코드가 서로 침범함 | BR-001/002로 실행 환경 경계 차단 |
| shared가 UI·DB에 오염됨 | BR-003으로 계약 계층 순수성 보장 |
| 다른 도메인의 내부 구현을 직접 참조함 | BR-004로 public API만 허용 |
| 앱이 도메인 내부 경로를 우회함 | BR-005로 composition root까지 보호 |
| manifest에 없는 숨은 도메인 결합이 생김 | BR-006과 순환 검사로 dependency graph 보호 |
| API 입력과 결과가 조용히 어긋남 | Zod 기반 input/result runtime 검증 |
| 오류가 추상적이고 수정 위치가 불명확함 | stable code, field path, suggestion, Vite overlay |

## Quick Start

Node.js 20 이상에서 runtime과 schema dependency를 설치합니다.

```bash
pnpm add boundra zod
pnpm exec boundra init --name my-app
pnpm exec boundra create-domain tasks
pnpm exec boundra generate resource tasks/task
pnpm exec boundra check-boundaries
```

`boundra` npm package는 TypeScript runtime과 native CLI launcher를 함께 제공합니다. CLI 최초 실행 시 현재 플랫폼의 checksummed Rust binary를 내려받아 user cache에 저장합니다.

### Define a contract

```ts
import { defineQuery, type InferSchema } from "boundra";
import { z } from "zod";

const getTaskInput = z.object({ id: z.string().min(1) });
const getTaskResult = z.object({
  id: z.string(),
  title: z.string(),
  done: z.boolean(),
});

export type Task = InferSchema<typeof getTaskResult>;

export const getTask = defineQuery({
  name: "get-task",
  input: getTaskInput,
  result: getTaskResult,
});
```

### Connect a client

```ts
import { createBoundraClient, createHttpTransport } from "boundra";

export const client = createBoundraClient(
  createHttpTransport({ baseUrl: "/api/boundra" }),
);

const task = await client.query(getTask, { id: "task-001" });
```

입력과 transport 결과는 모두 contract schema로 검증됩니다. 실패하면 `RUNTIME-001~003`과 field-level issue가 제공됩니다.
호출별 `{ signal }`을 세 번째 인자로 전달하면 custom/HTTP transport까지 취소가 전파되며, 의도적인 취소는 `RUNTIME-003`으로 감싸지 않습니다.

## Boundary Rules

```bash
pnpm exec boundra check-boundaries --format json
pnpm exec boundra graph-domains --format mermaid
```

| Rule | 차단하는 의존성 |
| --- | --- |
| `BR-001` | `client → server` |
| `BR-002` | `server → client` |
| `BR-003` | `shared → UI / DB / runtime infra` |
| `BR-004` | `domain → other domain internal` |
| `BR-005` | `app → domain internal` |
| `BR-006` | `domain → undeclared domain public API` |

진단은 rule code, file, line, import, message, suggestion을 포함하며 CI와 AI agent가 사용할 수 있는 안정적인 JSON 형식을 지원합니다.

## Vite Development Overlay

```ts
import { boundra } from "boundra/vite";
import { defineConfig } from "vite";

export default defineConfig({
  plugins: [boundra()],
});
```

개발 중 runtime contract 오류와 boundary 위반을 브라우저 overlay로 표시합니다. production build와 애플리케이션의 production error UI에는 개입하지 않습니다.

## CLI

| Command | 역할 |
| --- | --- |
| `init` | 비파괴적으로 Boundra workspace 초기화 |
| `create-domain <name>` | domain-first 디렉터리와 manifest 생성 |
| `generate route\|query\|mutation` | schema-backed 계약과 adapter 생성 |
| `generate resource` | CRUD 계약과 typed client API 생성 |
| `add-dependency` | domain graph 의존성 선언 |
| `check-boundaries` | BR-001~006 정적 검사 |
| `graph-domains` | Mermaid, JSON, DOT 의존성 그래프 출력 |

## Performance

Rust CLI의 synthetic workspace 기준선:

| Source files | Cold | Warm median |
| ---: | ---: | ---: |
| 1,000 | 20.74 ms | 18.92 ms |
| 10,000 | 163.32 ms | 164.20 ms |

환경과 측정 방법은 [Boundary Performance](docs/performance.md)에 기록하며 CI에서 catastrophic regression을 차단합니다.

## What Boundra Does Not Own

- JavaScript runtime 또는 bundler
- UI framework와 상태 관리
- ORM과 database abstraction
- HTTP router 또는 server framework
- test runner
- Next.js 대체

Boundra는 이 도구들 사이의 **구조와 계약**을 담당합니다.

## Documentation

| 시작하기 | 설계와 규칙 | 운영 |
| --- | --- | --- |
| [Quick Start](docs/quickstart.md) | [Architecture](docs/architecture.md) | [CI Integration](docs/ci-integration.md) |
| [CLI Install](docs/cli-install.md) | [Boundary Rules](docs/boundary-rules.md) | [Release Checklist](docs/release-checklist.md) |
| [Vite Integration](docs/vite-integration.md) | [Contract Schema](docs/contract-schema-spec.md) | [Security](SECURITY.md) |
| [HTTP Transport](docs/http-transport.md) | [Domain Manifest](docs/domain-manifest-spec.md) | [Contributing](CONTRIBUTING.md) |

전체 문서 목록은 [docs/readme.md](docs/readme.md)를 참고하세요.

## Status

Boundra는 현재 public preview입니다. CLI와 JSON schema는 가능한 additive하게 발전시키며 breaking architecture 결정은 [ADR](docs/adr)에 기록합니다.

<div align="center">

Built for codebases that humans and AI agents can change without dissolving their boundaries.

</div>
