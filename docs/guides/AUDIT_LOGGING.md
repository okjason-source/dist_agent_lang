# Audit Logging Guide

## Overview

DAL provides **persistent file-based audit logging** for security-critical operations, especially for services using the `@secure` attribute. All audit events are automatically logged to files for compliance, security analysis, and incident response.

---

## Quick Start

### Enable File Logging

Set environment variables before running your DAL application:

```bash
# Enable file logging
export LOG_SINK=both          # Options: console, file, both, none
export LOG_DIR=./logs          # Directory for log files (default: ./logs)
export LOG_FILE=./logs/audit.log  # Specific log file (optional, defaults to LOG_DIR/audit.log)

# Optional: Configure rotation and retention
export LOG_ROTATE_SIZE=10485760   # Rotate at 10MB (default: 10MB)
export LOG_RETENTION_DAYS=30      # Keep logs for 30 days (default: 30)
```

### Automatic Audit Logging

When you use `@secure` on a service, audit logs are **automatically created**:

```dal
@secure
@trust("hybrid")
@chain("ethereum")
service SecureService {
    fn update_data(data: map<string, any>) {
        // This access is automatically logged to file
        // No manual logging needed!
    }
}
```

---

## Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `LOG_SINK` | `console` | Output destination: `console`, `file`, `both`, or `none` |
| `LOG_DIR` | `./logs` | Directory where log files are stored |
| `LOG_FILE` | `./logs/audit.log` | Specific log file path (overrides LOG_DIR) |
| `LOG_ROTATE_SIZE` | `10485760` | Rotate log file when it exceeds this size (bytes) |
| `LOG_RETENTION_DAYS` | `30` | Keep log files for this many days |
| `LOG_LEVEL` | `debug` | Minimum log level: `debug`, `info`, `warning`, `error`, `audit` |
| `LOG_MAX_ENTRIES` | `1000` | Maximum in-memory log entries |

### Log Sink Options

- **`console`** (default): Logs only to stdout/stderr
- **`file`**: Logs only to file (no console output)
- **`both`**: Logs to both console and file (recommended for development)
- **`none`**: Disables all logging output

---

## Log File Format

Audit logs are written as **JSON Lines** (one JSON object per line) for easy parsing:

```json
{"timestamp":1707234567,"level":"Audit","message":"secure_service_access","source":"runtime","data":{"service":"UserService","method":"update_profile","caller":"0x1234...","result":"allowed"}}
{"timestamp":1707234568,"level":"Audit","message":"secure_service_access_denied","source":"runtime","data":{"service":"AdminService","method":"delete_user","caller":"unauthenticated","result":"denied"}}
```

### Log Entry Structure

```json
{
  "timestamp": 1707234567,        // Unix timestamp
  "level": "Audit",               // Log level (Info, Warning, Error, Audit, Debug)
  "message": "secure_service_access",  // Event name
  "source": "runtime",            // Source identifier
  "data": {                       // Event-specific data
    "service": "UserService",
    "method": "update_profile",
    "caller": "0x1234...",
    "result": "allowed"
  }
}
```

---

## Automatic Audit Events

### `@secure` Service Access

When a service has `@secure`, the runtime automatically logs:

1. **`secure_service_access`** - Successful authenticated access
   ```json
   {
     "message": "secure_service_access",
     "data": {
       "service": "UserService",
       "method": "update_profile",
       "caller": "0x1234...",
       "result": "allowed"
     }
   }
   ```

2. **`secure_service_access_denied`** - Unauthorized access attempt
   ```json
   {
     "message": "secure_service_access_denied",
     "data": {
       "service": "AdminService",
       "method": "delete_user",
       "caller": "unauthenticated",
       "result": "denied"
     }
   }
   ```

---

## Log Rotation

Log files are automatically rotated when they exceed the size limit:

- **Rotation trigger**: File size >= `LOG_ROTATE_SIZE` (default: 10MB)
- **Rotated filename**: `audit.log.{timestamp}` (e.g., `audit.log.1707234567`)
- **New file**: A fresh `audit.log` is created
- **Old file cleanup**: Files older than `LOG_RETENTION_DAYS` are automatically deleted

### Example Rotation

```
logs/
  ├── audit.log              # Current log file
  ├── audit.log.1707148167   # Rotated file (older)
  └── audit.log.1707051767   # Rotated file (oldest, will be cleaned up)
```

---

## Manual Audit Logging

You can also manually log audit events:

```dal
log::audit("user_login", {
    "user_id": "123",
    "ip": "192.168.1.1",
    "success": true
}, Some("auth"));
```

All `log::audit()` calls are automatically written to the audit log file when file logging is enabled.

---

## Querying Audit Logs

### Using Command Line Tools

```bash
# View all audit logs
cat logs/audit.log | jq '.'

# Filter by event type
cat logs/audit.log | jq 'select(.message == "secure_service_access_denied")'

# Filter by service
cat logs/audit.log | jq 'select(.data.service == "UserService")'

# Filter by time range (last hour)
cat logs/audit.log | jq "select(.timestamp > $(date -d '1 hour ago' +%s))"

# Count denied access attempts
cat logs/audit.log | jq 'select(.message == "secure_service_access_denied")' | wc -l
```

### Using DAL Runtime

```dal
// Get all audit entries
let audit_entries = log::get_entries_by_level(LogLevel::Audit);

// Get entries by source
let runtime_audits = log::get_entries_by_source("runtime");

// Get log statistics
let stats = log::get_stats();
// Returns: {"total_entries": 150, "count_audit": 45, "source_runtime": 30, ...}
```

---

## Production Best Practices

### 1. **Enable File Logging in Production**

```bash
export LOG_SINK=file          # File only (no console spam)
export LOG_DIR=/var/log/dal   # Use system log directory
export LOG_RETENTION_DAYS=90  # Keep logs longer for compliance
```

### 2. **Secure Log Directory**

```bash
# Create log directory with proper permissions
sudo mkdir -p /var/log/dal
sudo chown dal:dal /var/log/dal
sudo chmod 750 /var/log/dal
```

### 3. **Monitor Log Size**

Set appropriate rotation size based on your volume:

```bash
# High-volume: Rotate more frequently
export LOG_ROTATE_SIZE=5242880   # 5MB

# Low-volume: Rotate less frequently
export LOG_ROTATE_SIZE=52428800  # 50MB
```

### 4. **Integrate with Log Aggregation**

Forward audit logs to your log aggregation system:

```bash
# Example: Forward to syslog
tail -f /var/log/dal/audit.log | logger -t dal-audit

# Example: Forward to ELK stack
filebeat -c filebeat.yml
```

### 5. **Regular Audit Review**

Set up alerts for security events:

```bash
# Alert on denied access attempts
grep "secure_service_access_denied" /var/log/dal/audit.log | \
  mail -s "Security Alert" admin@example.com
```

---

## Troubleshooting

### Logs Not Being Written

**Check:**
1. `LOG_SINK` is set to `file` or `both`
2. `LOG_DIR` directory exists and is writable
3. Application has permissions to write to log directory

**Test:**
```bash
export LOG_SINK=both
export LOG_DIR=./test_logs
dal run your_service.dal
ls -la test_logs/audit.log
```

### Log File Too Large

**Solution:** Reduce `LOG_ROTATE_SIZE` or increase rotation frequency:

```bash
export LOG_ROTATE_SIZE=1048576  # 1MB (rotate more frequently)
```

### Old Logs Not Being Cleaned

**Check:** `LOG_RETENTION_DAYS` is set correctly:

```bash
export LOG_RETENTION_DAYS=7  # Keep only 7 days
```

### Performance Impact

File logging is **asynchronous** and should have minimal performance impact. If you experience issues:

1. Use `LOG_SINK=file` (disable console output)
2. Increase `LOG_ROTATE_SIZE` to reduce rotation frequency
3. Consider using a faster storage backend (SSD)

---

## Integration Examples

### Docker

```dockerfile
ENV LOG_SINK=file
ENV LOG_DIR=/app/logs
VOLUME /app/logs
```

### Kubernetes

```yaml
env:
  - name: LOG_SINK
    value: "file"
  - name: LOG_DIR
    value: "/var/log/dal"
volumeMounts:
  - name: audit-logs
    mountPath: /var/log/dal
```

### Systemd Service

```ini
[Service]
Environment="LOG_SINK=file"
Environment="LOG_DIR=/var/log/dal"
Environment="LOG_RETENTION_DAYS=90"
```

---

## Security Considerations

1. **Log File Permissions**: Restrict access to audit logs (chmod 600)
2. **Encryption**: Consider encrypting log files at rest for sensitive data
3. **Backup**: Regularly backup audit logs for compliance
4. **Monitoring**: Set up alerts for suspicious patterns (many denied attempts)
5. **Retention**: Comply with data retention requirements (GDPR, SOC2, etc.)

---

## Summary

- ✅ **Automatic**: `@secure` services automatically generate audit logs
- ✅ **Persistent**: Logs are written to files for long-term storage
- ✅ **Configurable**: Control via environment variables
- ✅ **Rotated**: Automatic log rotation prevents disk space issues
- ✅ **Queryable**: JSON format enables easy analysis and filtering

Audit logging is **production-ready** and provides the foundation for security compliance and incident response.
