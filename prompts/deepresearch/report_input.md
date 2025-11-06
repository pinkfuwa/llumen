# Research Requirements

## Task

{{ plan_title }}

## Description

{{ enhanced_prompt }}

---

IMPORTANT: Structure your report according to the format in the prompt. Remember to include:

1. Key Points - A bulleted list of the most important findings
2. Overview - A brief introduction to the topic
3. Detailed Analysis - Organized into logical sections
4. Survey Note (optional) - For more comprehensive reports
5. Key Citations - List all references at the end

For citations, DO NOT include inline citations in the text. Instead, place all citations in the 'Key Citations' section at the end using the format: `- [Source Title](URL)`. Include an empty line between each citation for better readability.

PRIORITIZE USING MARKDOWN TABLES for data presentation and comparison. Use tables whenever presenting comparative data, statistics, features, or options. Structure tables with clear headers and aligned columns. Example table format:

| Feature | Description | Pros | Cons |
|---------|-------------|------|------|
| Feature 1 | Description 1 | Pros 1 | Cons 1 |
| Feature 2 | Description 2 | Pros 2 | Cons 2 |

---

Below are some observations for the research task:
{% for step in completed_steps %}

{{ step.content }}
{% endfor %}
