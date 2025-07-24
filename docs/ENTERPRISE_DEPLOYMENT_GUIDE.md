# StableRWA Platform - Enterprise Deployment Guide

**Version:** 1.0.0  
**Author:** arkSong (arksong2018@gmail.com)  
**Framework:** StableRWA - Enterprise RWA Tokenization Technology Framework Platform

---

## 🎯 Overview

This guide provides comprehensive instructions for deploying the StableRWA Platform in enterprise production environments. The deployment follows industry best practices for security, scalability, and reliability.

---

## 🏗️ Architecture Requirements

### Infrastructure Requirements

| Component | Minimum | Recommended | Enterprise |
|-----------|---------|-------------|------------|
| **CPU** | 8 cores | 16 cores | 32+ cores |
| **Memory** | 32 GB | 64 GB | 128+ GB |
| **Storage** | 500 GB SSD | 1 TB NVMe | 2+ TB NVMe |
| **Network** | 1 Gbps | 10 Gbps | 25+ Gbps |
| **Nodes** | 3 | 5 | 10+ |

### Software Requirements

- **Kubernetes**: 1.28+
- **Docker**: 24.0+
- **PostgreSQL**: 15+
- **Redis**: 7.0+
- **Prometheus**: 2.45+
- **Grafana**: 10.0+

---

## 🔐 Security Configuration

### 1. Network Security

```yaml
# network-policy.yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: stablerwa-network-policy
spec:
  podSelector:
    matchLabels:
      app: stablerwa
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          name: stablerwa-system
    ports:
    - protocol: TCP
      port: 8080
  egress:
  - to:
    - namespaceSelector:
        matchLabels:
          name: stablerwa-database
    ports:
    - protocol: TCP
      port: 5432
```

### 2. TLS Configuration

```yaml
# tls-secret.yaml
apiVersion: v1
kind: Secret
metadata:
  name: stablerwa-tls
  namespace: stablerwa-system
type: kubernetes.io/tls
data:
  tls.crt: LS0tLS1CRUdJTi... # Base64 encoded certificate
  tls.key: LS0tLS1CRUdJTi... # Base64 encoded private key
```

### 3. RBAC Configuration

```yaml
# rbac.yaml
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: stablerwa-operator
rules:
- apiGroups: [""]
  resources: ["pods", "services", "configmaps", "secrets"]
  verbs: ["get", "list", "watch", "create", "update", "patch", "delete"]
- apiGroups: ["apps"]
  resources: ["deployments", "statefulsets"]
  verbs: ["get", "list", "watch", "create", "update", "patch", "delete"]
```

---

## 🚀 Deployment Steps

### Step 1: Prepare Environment

```bash
# Create namespace
kubectl create namespace stablerwa-system

# Apply RBAC
kubectl apply -f k8s/rbac.yaml

# Create secrets
kubectl apply -f k8s/secrets/
```

### Step 2: Deploy Database Layer

```bash
# Deploy PostgreSQL cluster
kubectl apply -f k8s/database/postgresql-cluster.yaml

# Deploy Redis cluster
kubectl apply -f k8s/database/redis-cluster.yaml

# Wait for databases to be ready
kubectl wait --for=condition=ready pod -l app=postgresql -n stablerwa-system --timeout=300s
kubectl wait --for=condition=ready pod -l app=redis -n stablerwa-system --timeout=300s
```

### Step 3: Deploy Core Services

```bash
# Deploy in dependency order
kubectl apply -f k8s/services/gateway/
kubectl apply -f k8s/services/asset/
kubectl apply -f k8s/services/oracle/
kubectl apply -f k8s/services/custody/
kubectl apply -f k8s/services/ai/

# Verify deployments
kubectl get pods -n stablerwa-system
```

### Step 4: Configure Monitoring

```bash
# Deploy Prometheus
kubectl apply -f k8s/monitoring/prometheus/

# Deploy Grafana
kubectl apply -f k8s/monitoring/grafana/

# Deploy AlertManager
kubectl apply -f k8s/monitoring/alertmanager/
```

### Step 5: Setup Ingress

```bash
# Deploy ingress controller
kubectl apply -f k8s/ingress/nginx-controller.yaml

# Configure ingress rules
kubectl apply -f k8s/ingress/stablerwa-ingress.yaml
```

---

## 📊 Configuration Management

### Environment Variables

```yaml
# configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: stablerwa-config
  namespace: stablerwa-system
data:
  DATABASE_URL: "postgresql://stablerwa:password@postgresql:5432/stablerwa"
  REDIS_URL: "redis://redis:6379"
  LOG_LEVEL: "info"
  METRICS_ENABLED: "true"
  TRACING_ENABLED: "true"
  SECURITY_LEVEL: "enterprise"
```

### Secrets Management

```bash
# Create database secrets
kubectl create secret generic stablerwa-db-secret \
  --from-literal=username=stablerwa \
  --from-literal=password=secure-password \
  -n stablerwa-system

# Create API keys
kubectl create secret generic stablerwa-api-keys \
  --from-literal=openai-key=sk-... \
  --from-literal=jwt-secret=secure-jwt-secret \
  -n stablerwa-system
```

---

## 🔍 Health Checks & Monitoring

### Health Check Configuration

```yaml
# health-check.yaml
apiVersion: v1
kind: Service
metadata:
  name: stablerwa-health
spec:
  selector:
    app: stablerwa-gateway
  ports:
  - name: health
    port: 8080
    targetPort: 8080
---
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: stablerwa-health-ingress
spec:
  rules:
  - host: health.stablerwa.com
    http:
      paths:
      - path: /health
        pathType: Prefix
        backend:
          service:
            name: stablerwa-health
            port:
              number: 8080
```

### Monitoring Dashboards

```bash
# Import Grafana dashboards
kubectl apply -f k8s/monitoring/dashboards/

# Configure alerts
kubectl apply -f k8s/monitoring/alerts/
```

---

## 🔄 Backup & Recovery

### Database Backup

```bash
# Automated backup script
#!/bin/bash
BACKUP_DIR="/backups/$(date +%Y%m%d)"
mkdir -p $BACKUP_DIR

# PostgreSQL backup
kubectl exec -n stablerwa-system postgresql-0 -- \
  pg_dump -U stablerwa stablerwa > $BACKUP_DIR/postgresql.sql

# Redis backup
kubectl exec -n stablerwa-system redis-0 -- \
  redis-cli BGSAVE
kubectl cp stablerwa-system/redis-0:/data/dump.rdb $BACKUP_DIR/redis.rdb
```

### Disaster Recovery

```bash
# Recovery procedure
kubectl apply -f k8s/recovery/

# Restore from backup
kubectl exec -n stablerwa-system postgresql-0 -- \
  psql -U stablerwa -d stablerwa < /backups/latest/postgresql.sql
```

---

## 📈 Scaling Configuration

### Horizontal Pod Autoscaler

```yaml
# hpa.yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: stablerwa-gateway-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: stablerwa-gateway
  minReplicas: 3
  maxReplicas: 50
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
```

### Vertical Pod Autoscaler

```yaml
# vpa.yaml
apiVersion: autoscaling.k8s.io/v1
kind: VerticalPodAutoscaler
metadata:
  name: stablerwa-gateway-vpa
spec:
  targetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: stablerwa-gateway
  updatePolicy:
    updateMode: "Auto"
  resourcePolicy:
    containerPolicies:
    - containerName: gateway
      maxAllowed:
        cpu: 2
        memory: 4Gi
      minAllowed:
        cpu: 100m
        memory: 128Mi
```

---

## 🔐 Security Hardening

### Pod Security Standards

```yaml
# pod-security-policy.yaml
apiVersion: policy/v1beta1
kind: PodSecurityPolicy
metadata:
  name: stablerwa-psp
spec:
  privileged: false
  allowPrivilegeEscalation: false
  requiredDropCapabilities:
    - ALL
  volumes:
    - 'configMap'
    - 'emptyDir'
    - 'projected'
    - 'secret'
    - 'downwardAPI'
    - 'persistentVolumeClaim'
  runAsUser:
    rule: 'MustRunAsNonRoot'
  seLinux:
    rule: 'RunAsAny'
  fsGroup:
    rule: 'RunAsAny'
```

### Network Policies

```yaml
# network-security.yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: stablerwa-deny-all
spec:
  podSelector: {}
  policyTypes:
  - Ingress
  - Egress
---
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: stablerwa-allow-internal
spec:
  podSelector:
    matchLabels:
      app: stablerwa
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - podSelector:
        matchLabels:
          app: stablerwa
  egress:
  - to:
    - podSelector:
        matchLabels:
          app: stablerwa
```

---

## 🧪 Testing Deployment

### Smoke Tests

```bash
# Run deployment smoke tests
kubectl apply -f k8s/tests/smoke-tests.yaml

# Check test results
kubectl logs -f job/stablerwa-smoke-tests -n stablerwa-system
```

### Load Testing

```bash
# Deploy load testing
kubectl apply -f k8s/tests/load-tests.yaml

# Monitor during load test
kubectl top pods -n stablerwa-system
```

---

## 📋 Maintenance Procedures

### Rolling Updates

```bash
# Update deployment
kubectl set image deployment/stablerwa-gateway \
  gateway=stablerwa/gateway:v1.1.0 \
  -n stablerwa-system

# Monitor rollout
kubectl rollout status deployment/stablerwa-gateway -n stablerwa-system
```

### Rollback Procedures

```bash
# Rollback to previous version
kubectl rollout undo deployment/stablerwa-gateway -n stablerwa-system

# Rollback to specific revision
kubectl rollout undo deployment/stablerwa-gateway --to-revision=2 -n stablerwa-system
```

---

## 🚨 Troubleshooting

### Common Issues

1. **Pod Startup Failures**
   ```bash
   kubectl describe pod <pod-name> -n stablerwa-system
   kubectl logs <pod-name> -n stablerwa-system
   ```

2. **Database Connection Issues**
   ```bash
   kubectl exec -it <pod-name> -n stablerwa-system -- \
     psql -h postgresql -U stablerwa -d stablerwa
   ```

3. **Network Connectivity**
   ```bash
   kubectl exec -it <pod-name> -n stablerwa-system -- \
     nslookup postgresql.stablerwa-system.svc.cluster.local
   ```

### Emergency Procedures

```bash
# Emergency scale down
kubectl scale deployment stablerwa-gateway --replicas=0 -n stablerwa-system

# Emergency maintenance mode
kubectl apply -f k8s/maintenance/maintenance-mode.yaml
```

---

## 📞 Support & Contacts

### Emergency Contacts

- **Platform Team**: platform@stablerwa.com
- **Security Team**: security@stablerwa.com
- **On-call Engineer**: +1-XXX-XXX-XXXX

### Documentation Links

- **API Documentation**: https://docs.stablerwa.com/api
- **Operations Runbook**: https://docs.stablerwa.com/ops
- **Security Procedures**: https://docs.stablerwa.com/security

---

**Deployment Guide Version:** 1.0.0  
**Last Updated:** 2024-07-24  
**Next Review:** 2024-08-24
