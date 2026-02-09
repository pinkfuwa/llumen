const DEFAULT_USER_PREFIX =
  "Please generate a concise title, starting with a emoji";
const TRIM_REGEX = /^[\n \t`"'*#]+|[\n \t`"'*#]+$/g;

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

const buildTitleMessages = (vars) => {
  const messages = [];
  if (vars.system_message) {
    messages.push({ role: "system", content: vars.system_message });
  }
  if (vars.user_prompt) {
    messages.push({ role: "user", content: vars.user_prompt });
  }
  if (vars.agent_answer) {
    const assistantText = truncateChars(vars.agent_answer, 300);
    if (assistantText) {
      messages.push({ role: "assistant", content: assistantText });
    }
  }
  const userPrefix = vars.user_prefix || DEFAULT_USER_PREFIX;
  messages.push({ role: "user", content: userPrefix });
  return { messages, userPrefix };
};

const transformVars = (vars) => {
  const { messages, userPrefix } = buildTitleMessages(vars);
  return {
    ...vars,
    user_prefix: userPrefix,
    title_messages: messages,
  };
};

const transformOutput = (output, context) => {
  let title = typeof output === "string" ? output : String(output ?? "");
  if (!title) {
    const fallback = context?.vars?.user_prompt;
    if (fallback) {
      title = fallback;
    }
  }
  let trimmed = truncateChars(trimMatches(title), 60);
  if (trimmed.includes("\n")) {
    trimmed = trimMatches(trimmed.split("\n")[0] || "");
  }
  return trimmed;
};

module.exports = {
  transformVars,
  transformOutput,
};
