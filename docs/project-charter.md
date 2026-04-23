# Project Charter

## 1. Mission

Boundra의 미션은 도메인 중심 구조를 도구로 강제해, 풀스택 TypeScript 개발의 구조적 일관성과 확장성을 확보하는 것입니다.

## 2. Positioning

- 지향: AI 시대에 최적화된 도메인 기반 개발 플랫폼
- 비지향: 범용 런타임/번들러/프레임워크 대체

## 3. Problem Statement

### 3.1 도메인 단위 분리 부재
서버, 클라이언트, 타입, 정책이 산발적으로 배치되어 변경 영향 범위를 예측하기 어렵다.

### 3.2 모노레포 경계 붕괴
import 자유도가 높아 shared 남용과 순환 의존이 발생한다.

### 3.3 구조 일관성의 인적 의존
팀이 커질수록 규약 준수가 개인 습관에 의존해 시스템 품질이 하락한다.

## 4. Core Values

- Structure over convention: 규약 권고가 아니라 구조 강제
- Contract over ad-hoc types: 임시 타입 공유보다 계약 중심 통합
- Domain over layer sprawl: 앱 중심 분산보다 도메인 중심 응집
- AI-ready by design: 위치 규칙과 생성 규칙을 통해 AI 작업 문맥 최소화

## 5. Non-Goals (Phase 0~2)

- 자체 JS 런타임
- 자체 번들러
- 자체 UI 프레임워크
- 자체 ORM
- 자체 상태관리
- 자체 테스트 러너
- Next.js 대체

## 6. Design Principles

1. 앱보다 도메인이 먼저다
2. shared는 최소화한다
3. 경계는 규칙이 아니라 도구로 강제한다
4. 타입 공유보다 계약(shared contract)을 우선한다
5. 자유보다 안전한 제약을 제공한다
6. AI가 이해하기 쉬운 구조를 만든다
7. MCP는 코어가 아니라 확장 레이어다

## 7. Success Criteria

- 설명 없이 구조를 탐색 가능
- 도메인 추가 리드타임이 일정하게 유지
- 경계 위반이 CI에서 조기 차단
- AI가 도메인 기준으로 코드 생성/수정 시 오작동이 감소
- 팀 확장(2~3명) 이후에도 구조 일관성 유지

## 8. Decision Policy

아래 질문 중 하나라도 "아니오"면 스코프에서 제외한다.
- 도메인 경계 강제에 직접 기여하는가?
- AI 작업 정확도 개선에 기여하는가?
- 초기 팀(1~3명)의 유지보수 효율을 높이는가?
