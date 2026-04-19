const DEFAULT_TIME = "Monday, 12:00, 15 December 2025";
const DEFAULT_TITLE_PREFIX =
  "Please generate a concise title, starting with a emoji";
const DEFAULT_IMAGE_MODEL_ID = "google/gemini-2.5-flash-image-preview";
const DEFAULT_VIDEO_MODEL_ID = "google/veo-3-fast";
const DEFAULT_IMAGE_PARAMETERS = [
  "prompt",
  "aspect_ratio",
  "reference_file_names",
];
const DEFAULT_VIDEO_PARAMETERS = [
  "prompt",
  "duration",
  "resolution",
  "aspect_ratio",
  "size",
  "generate_audio",
  "reference_file_names",
];

const TRIM_REGEX = /^[\n \t`\"'*#]+|[\n \t`\"'*#]+$/g;

const stripReasoningPreamble = (value) => {
  if (typeof value !== "string") {
    return value;
  }

  const normalized = value.replace(/\r\n/g, "\n");
  const lines = normalized.split("\n");

  while (lines.length > 0 && /^(thinking|reasoning)\s*:/i.test(lines[0].trim())) {
    lines.shift();
    while (lines.length > 0 && lines[0].trim() === "") {
      lines.shift();
    }
  }

  return lines.join("\n");
};
const THINKING_REGEX = /^(?:Thinking|Reasoning)\s*:\s*.*?(?=\n\S|$)/is;

const ensureArray = (value, fallback) => {
  if (Array.isArray(value)) {
    return value;
  }
  if (value == null || value === "") {
    return fallback;
  }
  return [value];
};

const normalizeParameterList = (value, fallback) => {
  return ensureArray(value, fallback)
    .map((item) => String(item || "").trim())
    .filter((item) => item.length > 0)
    .join(", ");
};

const normalizeLocale = (value) => {
  const lowered = String(value || "en").toLowerCase();
  if (lowered === "zh-hant" || lowered === "zh-tw" || lowered === "zh_tw") {
    return "zh-tw";
  }
  if (lowered === "zh-hans" || lowered === "zh-cn" || lowered === "zh_cn") {
    return "zh-cn";
  }
  if (lowered.startsWith("zh-tw") || lowered.startsWith("zh-hant")) {
    return "zh-tw";
  }
  if (lowered.startsWith("zh-cn") || lowered.startsWith("zh-hans")) {
    return "zh-cn";
  }
  return "en";
};

const extractModelId = (vars, context) => {
  if (vars.model_id) {
    return String(vars.model_id);
  }

  const providerId = context?.provider?.id || context?.provider;
  if (typeof providerId !== "string") {
    return "";
  }

  if (providerId.startsWith("openrouter:")) {
    return providerId.slice("openrouter:".length);
  }
  if (providerId.startsWith("openai:chat:")) {
    return providerId.slice("openai:chat:".length);
  }
  if (providerId.startsWith("openai:responses:")) {
    return providerId.slice("openai:responses:".length);
  }

  const split = providerId.split(":");
  return split[split.length - 1] || "";
};

const truncateChars = (value, limit) => {
  if (!value) {
    return "";
  }
  return Array.from(value).slice(0, limit).join("");
};

const trimMatches = (value) => {
  if (!value) {
    return "";
  }
  return value.replace(TRIM_REGEX, "");
};

const isLlumenRelated = (query) => /\bllumen\b/i.test(String(query || ""));

const buildRenderVars = (vars, context) => {
  const locale = normalizeLocale(vars.locale);
  const modelId = extractModelId(vars, context);

  return {
    locale,
    model_id: modelId,
    model_name: vars.model_name || modelId,
    model_provider: vars.model_provider || "",
    model_supported_parameters: vars.model_supported_parameters || [],
    llumen_related: isLlumenRelated(vars.user_query || vars.user_prompt || ""),
    time: vars.time || DEFAULT_TIME,
    chat_title: vars.chat_title || "",
  };
};

const transformNormalVars = (vars, context) => {
  const renderVars = buildRenderVars(vars, context);
  const userQuery = String(vars.user_query || vars.user_prompt || "");

  const imageParams = normalizeParameterList(
    vars.image_model_supported_parameters,
    DEFAULT_IMAGE_PARAMETERS
  );
  const videoParams = normalizeParameterList(
    vars.video_model_supported_parameters,
    DEFAULT_VIDEO_PARAMETERS
  );

  return {
    ...vars,
    ...renderVars,
    user_query: userQuery,
    image_model_id: vars.image_model_id || DEFAULT_IMAGE_MODEL_ID,
    video_model_id: vars.video_model_id || DEFAULT_VIDEO_MODEL_ID,
    image_model_supported_parameters: imageParams,
    video_model_supported_parameters: videoParams,
    image_model_supported_parameters_display: imageParams,
    video_model_supported_parameters_display: videoParams,
  };
};

const transformMediaVars = (vars, context) => {
  const renderVars = buildRenderVars(vars, context);
  const userQuery = String(vars.user_query || vars.user_prompt || "");

  const imageParams = normalizeParameterList(
    vars.image_model_supported_parameters,
    DEFAULT_IMAGE_PARAMETERS
  );
  const videoParams = normalizeParameterList(
    vars.video_model_supported_parameters,
    DEFAULT_VIDEO_PARAMETERS
  );

  return {
    ...vars,
    ...renderVars,
    user_query: userQuery,
    image_model_id: vars.image_model_id || DEFAULT_IMAGE_MODEL_ID,
    video_model_id: vars.video_model_id || DEFAULT_VIDEO_MODEL_ID,
    image_model_supported_parameters: imageParams,
    video_model_supported_parameters: videoParams,
    image_model_supported_parameters_display: imageParams,
    video_model_supported_parameters_display: videoParams,
  };
};

const transformTitleVars = (vars, context) => {
  const renderVars = buildRenderVars(vars, context);
  const userQuery = String(vars.user_query || vars.user_prompt || "");
  const assistantAnswer = truncateChars(String(vars.agent_answer || ""), 300);
  const userPrefix = vars.user_prefix || DEFAULT_TITLE_PREFIX;

  return {
    ...vars,
    ...renderVars,
    user_query: userQuery,
    assistant_truncated: assistantAnswer,
    user_prefix: userPrefix,
  };
};

const transformTitleOutput = (output, context) => {
  let title = typeof output === "string" ? output : String(output ?? "");
  if (!title) {
    title = String(context?.vars?.user_query || context?.vars?.user_prompt || "");
  }

  title = stripReasoningPreamble(title).replace(THINKING_REGEX, "");
  let trimmed = truncateChars(trimMatches(title), 60);
  if (trimmed.includes("\n")) {
    trimmed = trimMatches(trimmed.split("\n")[0] || "");
  }
  return trimmed;
};

const transformThinkingOutput = (output) => {
  return stripReasoningPreamble(output);
};

module.exports = {
  transformNormalVars,
  transformMediaVars,
  transformTitleVars,
  transformTitleOutput,
  transformThinkingOutput,
};
