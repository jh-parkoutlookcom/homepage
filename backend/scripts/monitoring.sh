# 배포 상태 모니터링
kubectl rollout status deployment/web-server -n homepage-backend

# 실시간 로그 확인
kubectl logs -f -l app=web-server -n homepage-backend

# 리소스 사용량 확인
kubectl top pods -n homepage-backend