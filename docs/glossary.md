# Glossary

- Domain: 비즈니스 기능 단위의 최상위 모듈
- Layer: client/server/shared/mcp/tests 중 하나의 역할 구분
- Boundary: layer/domain 간 허용된 의존 방향
- Public API: 도메인 외부에 노출 가능한 공식 진입점
- Contract: schema 중심의 타입/입출력 규약
- Manifest: 도메인 메타데이터 선언 파일(`domain.json`)
- Rule Code: 경계 위반 유형 식별 코드(BR-001 등)
- Analyzer: import graph 및 의존관계를 계산하는 엔진
- Codegen: 반복적인 보일러플레이트를 계약 기반으로 자동 생성
- Runtime Surface: generated contracts가 import하는 TypeScript helper API
- Starlark: future policy/codegen hook DSL candidate
- Lua: possible later local automation scripting candidate
