# Troubleshooting Guide

This guide helps you resolve common issues with Llumen.

## Connection Issues

### Cannot Connect to Llumen

**Symptoms:**
- Browser shows "connection refused"
- Page doesn't load

**Solutions:**

1. **Check if Llumen is running:**
   ```bash
   docker ps  # For Docker
   # or check if the process is running
   ```

2. **Verify the port binding:**
   - Docker default: port 80
   - Binary default: port 8001
   - Ensure no other service is using the port

3. **Check firewall settings:**
   ```bash
   # Linux
   sudo ufw status
   # or check your cloud provider's security groups
   ```

### API Connection Errors

**Symptoms:**
- "Failed to connect to API" error
- Models not loading

**Solutions:**

1. **Verify your API key:**
   - Check that `API_KEY` is set correctly
   - Ensure the key is valid and not expired

2. **Check the API endpoint:**
   - Default: OpenRouter (`https://openrouter.ai/api`)
   - Custom endpoint: Verify `OPENAI_API_BASE` is correct

3. **Test the API directly:**
   ```bash
   curl -H "Authorization: Bearer $API_KEY" \
        https://openrouter.ai/api/v1/models
   ```

## Authentication Issues

### Cannot Log In

**Symptoms:**
- Login fails with default credentials
- "Invalid credentials" error

**Solutions:**

1. **Use correct default credentials:**
   - Username: `admin`
   - Password: `P@88w0rd`

2. **Check for database issues:**
   - Ensure the data volume is mounted correctly
   - Check if `db.sqlite` exists in the data directory

3. **Reset the database** (last resort):
   - Stop Llumen
   - Remove or rename `data/db.sqlite`
   - Restart Llumen (creates new database with default user)

### Token Expired

**Symptoms:**
- Logged out unexpectedly
- "Unauthorized" errors

**Solutions:**
- Log in again to get a new token
- Tokens expire after a set period for security

## Chat and Model Issues

### Search/Deep Research Modes Grayed Out

**Symptoms:**
- Cannot select Search or Deep Research mode
- Modes appear disabled

**Cause:** The selected model doesn't support tool calling.

**Solutions:**

1. **Select a different model** that supports tool calling

2. **Force enable tool support:**
   - Go to Settings > Models
   - Edit the model configuration
   - Add:
     ```toml
     [capability]
     tool = true
     ```
   - Save and restart

### "Parameter tool not supported" Error

**Cause:** Earlier versions auto-detected tool support, but this was unreliable.

**Solution:** Explicitly declare tool support in model config:
```toml
[capability]
tool = true
```

### Model Not Appearing

**Solutions:**

1. **Refresh the model list** from settings
2. **Check API key permissions** - some keys have model restrictions
3. **Verify the model ID** is correct in your configuration

### Slow Responses

**Possible causes and solutions:**

1. **Large model selected** - Try a smaller, faster model
2. **Long conversation history** - Start a new chat
3. **Network latency** - Check your connection to the API
4. **API rate limiting** - Wait and retry

## File Upload Issues

### Upload Fails

**Solutions:**

1. **Check file size** - Very large files may fail
2. **Verify file type** - Ensure it's a supported format
3. **Check storage** - Ensure disk space is available

### Images Not Analyzed

**Cause:** Selected model may not support vision.

**Solution:** 
1. Choose a model with vision capability
2. Or add to model config:
   ```toml
   [capability]
   image = true
   ```

## Docker-Specific Issues

### Container Crashes

**Check logs:**
```bash
docker logs llumen
```

**Common causes:**
- Out of memory - Increase container memory limit
- Database corruption - Reset or restore database

### Data Not Persisting

**Ensure volume is mounted:**
```bash
docker run -v "$(pwd)/data:/data" ...
```

**Check permissions:**
```bash
ls -la data/
# Ensure the directory is writable
```

### Port Already in Use

**Error:** "address already in use"

**Solutions:**

1. **Find and stop the conflicting process:**
   ```bash
   lsof -i :80  # or your port
   ```

2. **Use a different port:**
   ```bash
   docker run -p 8080:80 ...
   ```

## Bug

### View Logs

**Docker:**
```bash
docker logs -f llumen
```

**Binary:**
Logs output to stderr by default.

## Getting More Help

If these solutions don't resolve your issue:

1. **Search existing issues:** [GitHub Issues](https://github.com/pinkfuwa/llumen/issues)
2. **Open a new issue** with:
   - Llumen version
   - How to reproduce the problem
   - Error messages and logs
   - Your configuration (without API keys)

## Quick Fixes Summary

| Issue | Quick Fix |
|-------|-----------|
| Can't log in | Username: `admin`, Password: `P@88w0rd` |
| Modes grayed out | Add `tool = true` to model config |
| API errors | Check `API_KEY` is set correctly |
| Data lost | Mount the `/data` volume |
| Port conflict | Change `-p 80:80` to `-p 8080:80` |
