# ADR 0001: Domain-First Architecture

- Status: Accepted
- Date: 2026-04-23

## Context

기존 앱 중심 구조는 기능 증가 시 관심사가 분산되고, 경계가 약해지며, AI 자동화 시 문맥 탐색 비용이 커진다.

## Decision

Boundra는 앱보다 도메인을 우선 배치하고, 각 도메인을 `client/server/shared/mcp/tests`로 구성한다.

## Consequences

긍정:
- 변경 영향 범위 예측 용이
- 경계 강제 자동화 용이
- AI 작업 단위 명확화

부정:
- 초기 구조 설계 비용 증가
- 자유도 감소에 대한 학습 필요

## Follow-up

- BR 규칙 코드를 구현하고 CLI 진단 포맷 확정
- domain manifest 스키마 고정
