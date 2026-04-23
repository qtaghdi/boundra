# Roadmap

## Phase 0: Foundation Design (Now)

- 철학/구조/규칙 문서화
- CLI와 manifest 명세 고정
- MVP 범위 잠금

## Phase 1: Prototype (MVP)

목표:
- `create-domain`
- `check-boundaries`
- 샘플 도메인 2개

완료 기준:
- 최소 4개 boundary rule 동작
- CI에서 위반 시 실패
- 팀 내 수동 검증 가능

## Phase 2: Internal Platform

목표:
- `graph-domains`
- `generate route/query/mutation`
- 규칙/진단 강화

완료 기준:
- 신규 도메인 온보딩 시간 단축
- 코드 생성 반복 패턴 정착

## Phase 3: Core Stabilization

목표:
- crate 분리 정돈
- 성능 개선(대형 모노레포 대상)
- 에러/진단 품질 개선

완료 기준:
- 분석 속도/메모리 기준 충족
- 코어 API 안정화

## Phase 4: External Release

목표:
- 문서/예제 공개
- CLI 패키징/배포
- 빠른 시작 경로 제공

## Phase 5: MCP Expansion

목표:
- domain MCP adapter 공식화
- tool/resource/prompt 규격화
- 권한 모델 정립
