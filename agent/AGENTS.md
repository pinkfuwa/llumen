## Prompt and Promptfoo

- Keep each promptfoo regression family in its own YAML file.
- Store promptfoo result JSON under `agent/promptfoo/.promptfoo/`.
- Use judge-only models only inside rubric assertions, not in the tested provider list.

## Testcases

- `promptfoo/normal_chat.yaml`: normal conversational chat, no leaked `<task>` tags.
- `promptfoo/llumen_related.yaml`: llumen info and hallucination/link correctness.
- `promptfoo/multilingual.yaml`: English, Simplified Chinese, Traditional Chinese, plus Traditional Chinese on mainland models with extra judges.
- `promptfoo/title.yaml`: chat title generation, judged with `minimax/minimax-m2.5`.
- `promptfoo/image_generation.yaml`: image tool-call prompt generation only, no actual image API call.
- `promptfoo/video_generation.yaml`: video workflow validation only, no actual video API call.
- `promptfoo/search.yaml`: search prompt regression with citations.
- Keep all promptfoo result files in `promptfoo/.promptfoo/`.

The folder contain prompt(jinja2 template) and promptfoo config.

## Workflow

### Add test:

1. Modify the config(yaml)
2. Run `promptfoo validate` to validate config(NO NEED to actually run the eval)

### Improve prompt

1. Find particular testsuit/section/prompt we would like to improve. 
2. Run eval to get current score of all related testsuit of that prompt.
3. Count token(skip if we are NOT optimizing for token count)
4. Modify the prompt.
5. Run ONLY the related testsuit for that section(For example: language section->multilingual.yaml).
6. Iterate the prompt until target number is reached.
7. Run other eval to see if performance regression of other testsuit is acceptable.
