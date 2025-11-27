## Frequently Asked Questions (FAQ)

### **1. How do I change the API endpoint?**
The environment variable for the API base URL has evolved across versions:
- **v0.2.0 and earlier**: Use `API_BASE`.
- **v0.3.0**: Use `OPENAI_API_BASE`.
- **v0.3.0+**: Both `API_BASE` and `OPENAI_API_BASE` work.

*Example*:
```bash
export OPENAI_API_BASE="https://your-custom-endpoint.com/v1"
```

---

### **2. How do I configure a model?**
Use a TOML file with the following schema. Press **Tab** in the config editor for auto-completion!

```toml
display_name = "GPT-OSS 20B"  # Friendly name for the UI
model_id = "openai/gpt-oss-20b"  # Model ID (no "online" suffix)

[capability]
image = true       # Supports image input
tool = true        # Supports tool/function calling
audio = true       # Supports audio input/output
json = true        # Supports structured JSON output
ocr = "native"     # OCR mode: "native", "text", "mistral", or "disabled"

[parameter]
top_k =            # Optional: Top-k sampling
top_p =            # Optional: Top-p (nucleus) sampling
repeat_penalty =   # Optional: Repeat penalty
temperature =      # Optional: Temperature
```

*Need more settings?* Check the [full documentation](https://github.com/pinkfuwa/llumen).

---

### **3. Why are some modes (Search/Deep Research) grayed out?**
Llumen disables advanced modes if it detects the model doesn’t support tool calling. To override this:
1. Explicitly enable `tool = true` in the model’s `[capability]` config.
2. Save the config and restart Llumen.

*Example*:
```toml
[capability]
tool = true  # Forces tool calling support
```

---

### **4. Why do I see "parameter tool not supported" errors?**
Earlier versions relied on OpenRouter’s API to detect tool support, but this was sometimes inaccurate. Now, **you must explicitly declare tool support** in the model config to disable tool call(see above).

---

### **Still stuck?**
Let me know what’s unclear—I’m happy to help! 😊

> [Eason0729](https://github.com/Eason0729/)
