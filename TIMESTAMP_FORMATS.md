# Supported Timestamp Formats

This document shows all the timestamp formats that `logpile` can automatically detect and parse.

## ✅ Supported Formats

### 1. ISO 8601 with Timezone
```
2025-10-03T12:34:56.789+00:00 INFO - User login successful
2025-10-03T12:34:56.789Z INFO - Request processed
```
**Format**: `%Y-%m-%dT%H:%M:%S%.f%:z` or `%Y-%m-%dT%H:%M:%S%.fZ`

### 2. ISO 8601 without Timezone
```
2025-10-03T12:34:56 INFO - Database connection established
2025-10-03T12:34:56.123 DEBUG - Cache hit
```
**Format**: `%Y-%m-%dT%H:%M:%S` or `%Y-%m-%dT%H:%M:%S%.f`

### 3. Standard DateTime Format
```
2025-10-03 12:34:56 INFO - Application started
2025-10-03 12:34:56.123456 ERROR - Request failed
```
**Format**: `%Y-%m-%d %H:%M:%S` or `%Y-%m-%d %H:%M:%S%.f`

### 4. Apache/Nginx Common Log Format
```
192.168.1.1 - - [03/Oct/2025:12:34:56 +0000] "GET /api/users HTTP/1.1" 200 1234
```
**Format**: `%d/%b/%Y:%H:%M:%S %z`

### 5. Syslog Format (RFC 3164)
```
Oct 03 12:34:56 myserver myapp[1234]: INFO - Service started
```
**Format**: `%b %d %H:%M:%S`
*Note: Year is automatically added from current year*

### 6. European Date Format
```
03/10/2025 12:34:56 DEBUG - Processing request
```
**Format**: `%d/%m/%Y %H:%M:%S`

### 7. US Date Format
```
10/03/2025 12:34:56 INFO - User authenticated
```
**Format**: `%m/%d/%Y %H:%M:%S`

### 8. Unix Timestamp
```
1727962496 INFO - Background job completed
```
**Format**: Unix epoch seconds (automatically detected)

### 9. RFC 2822
```
Fri, 03 Oct 2025 12:34:56 GMT INFO - Email sent
```
**Format**: `%a, %d %b %Y %H:%M:%S`

### 10. Java/Application Log Format
```
2025-10-03 12:34:56.789 INFO [http-nio-8080-exec-1] com.example.Service - Request processed
```
**Format**: `%Y-%m-%d %H:%M:%S%.f`

## Testing Timestamp Formats

Test files are provided in the `examples/` directory:

- ISO 8601 with timezone
- ISO 8601 without timezone
- Apache/Nginx format
- Syslog format
- DD/MM/YYYY format
- MM/DD/YYYY format
- Unix epoch timestamps
- High-precision timestamps
- RFC 2822 format
- Java application logs

## Custom Formats

If your log format isn't auto-detected, you can specify a custom format using the `--time-format` flag:

```bash
logpile "ERROR" my.log --time-format "%Y/%m/%d %H:%M:%S"
```

## How Auto-Detection Works

`logpile` uses regex patterns to extract potential timestamps from each log line, then tries multiple common formats until it finds a match. This happens automatically without any configuration needed.

The parser tries formats in this order:
1. Custom format (if specified)
2. Unix timestamps
3. ISO 8601 variants
4. Apache/Nginx format
5. RFC 2822 format
6. European/US date formats
7. Syslog format

## Examples

```bash
# Auto-detect format in Apache logs
logpile "404" /var/log/apache2/access.log --png errors.png

# Auto-detect format in application logs
logpile "ERROR" /var/log/myapp.log --csv errors.csv

# Works with mixed formats (uses first detected)
logpile "WARN" mixed-logs/* --json

# Works with compressed logs
logpile "Exception" app.log.gz --table
```

