# MVP Implementation Plan

## Goal

2~3주 내 `check-boundaries` 중심의 실사용 가능한 MVP를 완성한다.

## Week 1

- Rust workspace + crate 기본 세팅
- parser/analyzer 최소 동작 (TS import 추출)
- BR-001, BR-002 구현

## Week 2

- BR-003, BR-004 구현
- CLI 출력 포맷(text/json) 완성
- fixture 기반 테스트 세트 구축

## Week 3

- `create-domain` 스캐폴드 구현
- CI 연결 (`check-boundaries` 실행)
- 샘플 도메인(2개)로 end-to-end 검증

## Definition of Done

- `check-boundaries`가 규칙 4개를 안정적으로 검출
- 위반 시 exit code 1 반환
- 진단 메시지에 rule/file/line/suggestion 포함
- README + docs 인덱스 기준으로 온보딩 가능
