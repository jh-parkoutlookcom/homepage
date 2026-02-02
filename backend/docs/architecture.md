GitHub Push → GitHub Actions (CI/Build) → Harbor Registry
                     ↓
                 Jenkins (Integration Tests)
                     ↓
              ArgoCD (GitOps Deploy) → Kubernetes


전체 플로우
1. 개발자가 main 브랜치에 Push
2. GitHub Actions 실행
    - Test & Lint
    - Docker 이미지 빌드 및 Harbor 푸시
    - Jenkins 통합 테스트 트리거
3. Jenkins 실행
    - Integration Tests
    - E2E Tests
    - Performance Tests
    - Security Tests
4. 테스트 성공 시
    - GitHub Actions가 k8s-manifests 레포의 이미지 태그 업데이트
    - Git commit & push
5. ArgoCD 감지
    - Manifest 변경 감지
    - Kubernetes에 자동 배포
    - Health check 및 동기화
6. 알림
    - Slack으로 배포 완료 통지