# TodoList REST API

러스트로 작성된 간단한 TodoList REST API 서버입니다.

## 기능

- 할일 등록
- 할일 목록 조회
- 할일 완료/미완료 토글

## API 엔드포인트

1. 할일 등록

   - POST /todos
   - 요청 본문: `{ "title": "할일 제목" }`

2. 할일 목록 조회

   - GET /todos

3. 할일 완료/미완료 토글
   - POST /todos/{id}/toggle

## 실행 방법

```bash
cargo run
```

서버는 http://localhost:8080 에서 실행됩니다.

## 의존성

- actix-web: 웹 프레임워크
- serde: 직렬화/역직렬화
- chrono: 날짜/시간 처리
- uuid: 고유 식별자 생성
