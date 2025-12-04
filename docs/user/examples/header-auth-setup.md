# Header-Based Authentication Setup Examples

This document provides practical examples for setting up Llumen with header-based authentication behind various reverse proxies and SSO solutions.

## Quick Start Checklist

Before configuring header auth, ensure:
- [ ] Users exist in Llumen with the same usernames as your SSO system
- [ ] Your reverse proxy is properly authenticated and injecting headers
- [ ] The proxy is configured to forward the authentication header to Llumen
- [ ] Llumen is only accessible through the authenticated reverse proxy

## Example 1: Authelia + Nginx

Authelia is a Single Sign-On and Authorization server that provides authentication for your applications.

### Authelia Configuration

```yaml
# docker-compose.yml
version: '3'
services:
  authelia:
    image: authelia/authelia:latest
    environment:
      AUTHELIA_JWT_SECRET: your-jwt-secret-here
      AUTHELIA_SESSION_SECRET: your-session-secret-here
    volumes:
      - ./authelia-config.yml:/config/configuration.yml
    ports:
      - "9091:9091"

  nginx:
    image: nginx:latest
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf
    ports:
      - "80:80"
    depends_on:
      - authelia
      - llumen

  llumen:
    image: llumen:latest
    environment:
      API_KEY: "your-openrouter-api-key"
      TRUSTED_HEADER: "Remote-User"
    depends_on:
      - authelia
```

### Authelia Config File

```yaml
# authelia-config.yml
default_redirection_url: http://localhost

server:
  host: 0.0.0.0
  port: 9091

log:
  level: debug

session:
  domain: localhost
  name: authelia_session
  secret: your-session-secret-here
  expiration: 1h
  inactivity: 10m

authentication_backend:
  file:
    path: /config/users.yml

authorization:
  default_policy: deny
  rules:
    - domain: localhost
      policy: one_factor
      subject:
        - "group:users"

access_control:
  default_policy: deny
  rules:
    - domain: localhost
      policy: one_factor
```

### Users File

```yaml
# users.yml
users:
  admin:
    displayname: "Admin User"
    password: "$argon2id$v=19$m=65540,t=3,p=4$..."  # Use authelia generate password
    email: admin@example.com
    groups:
      - users
  
  john:
    displayname: "John Doe"
    password: "$argon2id$v=19$m=65540,t=3,p=4$..."
    email: john@example.com
    groups:
      - users
```

### Nginx Configuration

```nginx
# nginx.conf
http {
    upstream authelia {
        server authelia:9091;
    }

    upstream llumen {
        server llumen:8001;
    }

    server {
        listen 80;
        server_name localhost;

        location / {
            # Forward auth to Authelia
            auth_request /auth;
            auth_request_set $user $upstream_http_remote_user;
            auth_request_set $groups $upstream_http_remote_groups;

            # Proxy to Llumen with authenticated username
            proxy_pass http://llumen;
            proxy_set_header Remote-User $user;
            proxy_set_header X-Remote-User $user;
            proxy_set_header X-Remote-Groups $groups;
        }

        # Authelia auth endpoint
        location /auth {
            internal;
            proxy_pass http://authelia/api/verify;
            proxy_pass_request_body off;
            proxy_set_header Content-Length "";
        }

        # Authelia login page
        location /auth/login {
            proxy_pass http://authelia;
        }
    }
}
```

### Docker Compose Startup

```bash
# Generate users file with hashed passwords
docker run --rm authelia/authelia:latest authelia crypto hash generate argon2id \
  --password "your-password"

# Start the stack
docker-compose up -d

# Access Llumen at http://localhost
# You'll be redirected to Authelia login
```

## Example 2: OAuth2-Proxy + Nginx

OAuth2-Proxy integrates with OAuth2 providers (Google, GitHub, etc.).

### Docker Compose Setup

```yaml
version: '3'
services:
  oauth2-proxy:
    image: quay.io/oauth2-proxy/oauth2-proxy:latest
    environment:
      OAUTH2_PROXY_PROVIDER: google
      OAUTH2_PROXY_CLIENT_ID: your-client-id.apps.googleusercontent.com
      OAUTH2_PROXY_CLIENT_SECRET: your-client-secret
      OAUTH2_PROXY_REDIRECT_URL: http://localhost/oauth2/callback
      OAUTH2_PROXY_COOKIE_SECRET: your-cookie-secret-here
      OAUTH2_PROXY_EMAIL_DOMAIN: "*"
      OAUTH2_PROXY_COOKIE_HTTPONLY: "true"
      OAUTH2_PROXY_SET_XAUTHREQUEST: "true"
      OAUTH2_PROXY_SKIP_PROVIDER_BUTTON: "true"
    ports:
      - "4180:4180"

  nginx:
    image: nginx:latest
    volumes:
      - ./nginx-oauth2.conf:/etc/nginx/nginx.conf
    ports:
      - "80:80"
    depends_on:
      - oauth2-proxy
      - llumen

  llumen:
    image: llumen:latest
    environment:
      API_KEY: "your-openrouter-api-key"
      TRUSTED_HEADER: "X-Auth-Request-User"
```

### Nginx Configuration for OAuth2-Proxy

```nginx
# nginx-oauth2.conf
http {
    upstream oauth2_proxy {
        server oauth2-proxy:4180;
    }

    upstream llumen {
        server llumen:8001;
    }

    server {
        listen 80;
        server_name localhost;

        location /oauth2 {
            proxy_pass http://oauth2_proxy;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
        }

        location /oauth2/auth {
            internal;
            proxy_pass http://oauth2_proxy/auth;
            proxy_pass_request_body off;
            proxy_set_header Content-Length "";
        }

        location / {
            auth_request /oauth2/auth;
            auth_request_set $user $upstream_http_x_auth_request_user;

            proxy_pass http://llumen;
            proxy_set_header X-Auth-Request-User $user;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
        }
    }
}
```

## Example 3: Traefik with ForwardAuth

Traefik is a modern reverse proxy that works well with Kubernetes.

### Docker Compose Example

```yaml
version: '3'
services:
  traefik:
    image: traefik:latest
    command:
      - "--api.insecure=true"
      - "--providers.docker=true"
      - "--entryPoints.web.address=:80"
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
    ports:
      - "80:80"
      - "8080:8080"

  authelia:
    image: authelia/authelia:latest
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.authelia.rule=Host(`auth.localhost`)"
      - "traefik.http.services.authelia.loadbalancer.server.port=9091"

  llumen:
    image: llumen:latest
    environment:
      API_KEY: "your-openrouter-api-key"
      TRUSTED_HEADER: "Remote-User"
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.llumen.rule=Host(`localhost`)"
      - "traefik.http.routers.llumen.middlewares=forward-auth"
      - "traefik.http.middlewares.forward-auth.forwardauth.address=http://authelia:9091/api/verify"
      - "traefik.http.services.llumen.loadbalancer.server.port=8001"
```

## Example 4: Kubernetes with External Auth Service

For Kubernetes deployments using ingress-nginx with an external authentication service.

### Kubernetes Deployment

```yaml
---
# Namespace
apiVersion: v1
kind: Namespace
metadata:
  name: llumen

---
# ConfigMap for Llumen
apiVersion: v1
kind: ConfigMap
metadata:
  name: llumen-config
  namespace: llumen
data:
  TRUSTED_HEADER: "X-Remote-User"

---
# Secret for API Key (in production, use sealed-secrets or external-secrets)
apiVersion: v1
kind: Secret
metadata:
  name: llumen-secrets
  namespace: llumen
type: Opaque
stringData:
  api_key: "your-openrouter-api-key"

---
# Llumen Deployment
apiVersion: apps/v1
kind: Deployment
metadata:
  name: llumen
  namespace: llumen
spec:
  replicas: 1
  selector:
    matchLabels:
      app: llumen
  template:
    metadata:
      labels:
        app: llumen
    spec:
      containers:
      - name: llumen
        image: llumen:latest
        ports:
        - containerPort: 8001
        env:
        - name: API_KEY
          valueFrom:
            secretKeyRef:
              name: llumen-secrets
              key: api_key
        - name: TRUSTED_HEADER
          valueFrom:
            configMapKeyRef:
              name: llumen-config
              key: TRUSTED_HEADER
        - name: BIND_ADDR
          value: "0.0.0.0:8001"
        resources:
          requests:
            memory: "512Mi"
            cpu: "250m"
          limits:
            memory: "1Gi"
            cpu: "500m"

---
# Llumen Service
apiVersion: v1
kind: Service
metadata:
  name: llumen
  namespace: llumen
spec:
  selector:
    app: llumen
  ports:
  - protocol: TCP
    port: 80
    targetPort: 8001
  type: ClusterIP

---
# Ingress with Forward Auth
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: llumen-ingress
  namespace: llumen
  annotations:
    nginx.ingress.kubernetes.io/auth-url: http://auth-service:80/verify
    nginx.ingress.kubernetes.io/auth-signin: http://auth-service:80/login
    nginx.ingress.kubernetes.io/auth-response-headers: X-Remote-User
spec:
  ingressClassName: nginx
  rules:
  - host: llumen.example.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: llumen
            port:
              number: 80
```

## Troubleshooting

### Users Can't Log In

1. **Verify header is being sent:**
   ```bash
   curl -v -H "Remote-User: testuser" http://localhost:8001/api/auth/header \
     -H "Content-Type: application/json" \
     -d '{"username":"testuser"}'
   ```

2. **Check TRUSTED_HEADER environment variable:**
   ```bash
   docker exec <container-id> env | grep TRUSTED_HEADER
   ```

3. **Verify user exists in database:**
   - Log in to Llumen admin panel
   - Create the user if it doesn't exist with the exact username your SSO uses

### Header Not Reaching Backend

1. **Check nginx/proxy logs:**
   ```bash
   docker logs <nginx-container> | grep Remote-User
   ```

2. **Verify header name in config matches proxy output**

3. **Check for header name case sensitivity in HTTP (headers are case-insensitive but values are case-sensitive)**

### Token Renewal Keeps Failing

1. Verify the reverse proxy is still setting the header on renewal requests
2. Check backend logs for detailed error messages
3. Fall back to password login temporarily to test

## Best Practices

1. **Always test password auth first** - Set up and test regular login before configuring header auth
2. **Use HTTPS in production** - Never use plain HTTP with authentication
3. **Secure the reverse proxy** - Use strong credentials and keep the proxy updated
4. **Monitor header injection** - Log and audit which users are accessing Llumen
5. **Have a fallback plan** - Keep password auth enabled as a fallback
6. **Create admin user with password** - In case header auth fails completely
7. **Test header auth failure paths** - Ensure graceful fallback to login page

## References

- [Authelia Documentation](https://www.authelia.com/)
- [OAuth2-Proxy Documentation](https://oauth2-proxy.github.io/oauth2-proxy/)
- [Traefik ForwardAuth](https://doc.traefik.io/traefik/middlewares/http/forwardauth/)
- [Nginx auth_request](https://nginx.org/en/docs/http/ngx_http_auth_request_module.html)
