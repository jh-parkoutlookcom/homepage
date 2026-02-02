# 이전 배포로 롤백
kubectl rollout undo deployment/web-server -n homepage

# 특정 revision으로 롤백
kubectl rollout history deployment/web-server -n homepage
kubectl rollout undo deployment/web-server -n homepage --to-revision=2