const OpenCC = require("opencc-js");

const TRADITIONAL_ONLY_CHARS = new Set(
  Array.from(
    "體臺灣網開關於點數據設計這樣與應讓為務裡專業總結產業機會導覽標題優勢學習變電腦資源還報導證據願景觀點",
  ),
);

const SIMPLIFIED_ONLY_CHARS = new Set(
  Array.from(
    "体台湾网开关于点数据设计这样与应让为务里专业总结产业机会导览标题优势学习变电脑资源还报道证据愿景观点",
  ),
);

const CJK_REGEX = /[\u3400-\u9fff]/g;
const LATIN_REGEX = /[A-Za-z]/g;
const URL_REGEX = /https?:\/\/[^\s)>\]}]+/g;

const grade = (pass, reason) => ({
  pass,
  score: pass ? 1 : 0,
  reason,
});

const asText = (output) => {
  if (typeof output === "string") {
    return output;
  }
  if (output == null) {
    return "";
  }
  return JSON.stringify(output);
};

const countMatches = (text, regex) => (text.match(regex) || []).length;

const countSetChars = (text, set) => {
  let count = 0;
  for (const char of text) {
    if (set.has(char)) {
      count += 1;
    }
  }
  return count;
};

const countCharDiffs = (left, right) => {
  const leftChars = Array.from(left);
  const rightChars = Array.from(right);
  const maxLength = Math.max(leftChars.length, rightChars.length);
  let count = 0;

  for (let index = 0; index < maxLength; index += 1) {
    if (leftChars[index] !== rightChars[index]) {
      count += 1;
    }
  }

  return count;
};

const convertChineseVariant = (text, targetLocale) => {
  const locale = normalizeLocale(targetLocale);
  if (locale === "zh-tw") {
    return OpenCC.Converter({ from: "cn", to: "tw" })(text);
  }
  if (locale === "zh-cn") {
    return OpenCC.Converter({ from: "tw", to: "cn" })(text);
  }

  return text;
};

const isRegionalIndicator = (char) => /\p{Regional_Indicator}/u.test(char);

const normalizeLocale = (value) => {
  const locale = String(value || "en").toLowerCase();
  if (locale.startsWith("zh-tw") || locale.startsWith("zh-hant")) {
    return "zh-tw";
  }
  if (locale.startsWith("zh-cn") || locale.startsWith("zh-hans")) {
    return "zh-cn";
  }
  return "en";
};

const parseToolCalls = (output) => {
  const parsedFromString = (() => {
    if (typeof output !== "string") {
      return output;
    }
    try {
      return JSON.parse(output);
    } catch {
      return output;
    }
  })();

  if (Array.isArray(parsedFromString)) {
    return parsedFromString;
  }

  if (parsedFromString && Array.isArray(parsedFromString.tool_calls)) {
    return parsedFromString.tool_calls;
  }

  if (
    parsedFromString &&
    parsedFromString.choices &&
    Array.isArray(parsedFromString.choices) &&
    parsedFromString.choices[0] &&
    parsedFromString.choices[0].message &&
    Array.isArray(parsedFromString.choices[0].message.tool_calls)
  ) {
    return parsedFromString.choices[0].message.tool_calls;
  }

  return [];
};

const parseToolArguments = (toolCall) => {
  if (!toolCall) {
    return {};
  }

  const functionPart = toolCall.function || toolCall;
  const rawArguments = functionPart.arguments || functionPart.args || {};
  if (typeof rawArguments === "object" && rawArguments !== null) {
    return rawArguments;
  }

  if (typeof rawArguments === "string") {
    try {
      const parsed = JSON.parse(rawArguments);
      return parsed && typeof parsed === "object" ? parsed : {};
    } catch {
      return {};
    }
  }

  return {};
};

const getFirstToolPrompt = (output) => {
  const toolCalls = parseToolCalls(output);
  const firstCall = toolCalls[0];
  const args = parseToolArguments(firstCall);
  const prompt = typeof args.prompt === "string" ? args.prompt.trim() : "";
  return {
    prompt,
    toolCalls,
    firstCall,
    args,
  };
};

const detectLanguageRatio = (text, expected, threshold) => {
  const cjkCount = countMatches(text, CJK_REGEX);
  const latinCount = countMatches(text, LATIN_REGEX);

  const traditionalCount = countSetChars(text, TRADITIONAL_ONLY_CHARS);
  const simplifiedCount = countSetChars(text, SIMPLIFIED_ONLY_CHARS);
  const variantDenominator = Math.max(1, traditionalCount + simplifiedCount);

  if (expected === "en") {
    const denominator = Math.max(1, latinCount + cjkCount);
    const latinRatio = latinCount / denominator;
    return {
      pass: latinRatio >= threshold,
      reason: `English ratio=${latinRatio.toFixed(2)} threshold=${threshold}`,
      latinRatio,
    };
  }

  if (cjkCount === 0) {
    return {
      pass: false,
      reason: "No CJK characters detected.",
    };
  }

  if (expected === "zh-tw") {
    if (traditionalCount + simplifiedCount === 0) {
      return {
        pass: true,
        reason: "No variant-specific chars; compatible with zh-tw.",
      };
    }

    const ratio = traditionalCount / variantDenominator;
    return {
      pass: ratio >= threshold,
      reason: `Traditional ratio=${ratio.toFixed(2)} threshold=${threshold}`,
      ratio,
    };
  }

  if (traditionalCount + simplifiedCount === 0) {
    return {
      pass: true,
      reason: "No variant-specific chars; compatible with zh-cn.",
    };
  }

  const ratio = simplifiedCount / variantDenominator;
  return {
    pass: ratio >= threshold,
    reason: `Simplified ratio=${ratio.toFixed(2)} threshold=${threshold}`,
    ratio,
  };
};

function assertNoTaskTag(output) {
  const text = asText(output);
  const hasTaskTag = /<\/?task>/i.test(text);
  return grade(
    !hasTaskTag,
    hasTaskTag ? "Found <task> tag." : "No <task> tag.",
  );
}

function assertLanguageByScript(output, context) {
  const config = context?.config || {};
  const expected = normalizeLocale(config.expected || context?.vars?.locale);
  const threshold = Number(config.threshold || 0.8);
  const text = asText(output);

  const detection = detectLanguageRatio(text, expected, threshold);
  return grade(detection.pass, detection.reason);
}

function assertChineseOpenccChangeLimit(output, context) {
  const config = context?.config || {};
  const expected = normalizeLocale(config.expected || context?.vars?.locale);
  const maxChanges = Number(config.maxChanges || 3);
  const text = asText(output);

  if (expected !== "zh-cn" && expected !== "zh-tw") {
    return grade(true, `Skipped OpenCC check for ${expected}.`);
  }

  const converted = convertChineseVariant(text, expected);
  const changes = countCharDiffs(text, converted);

  return grade(
    changes <= maxChanges,
    `OpenCC conversion to ${expected} changed ${changes} chars (max ${maxChanges}).`,
  );
}

function assertOfficialLlumenLinks(output) {
  const text = asText(output);
  const urls = text.match(URL_REGEX) || [];

  const expectedLinks = [
    "https://github.com/pinkfuwa/llumen",
    "https://pinkfuwa.github.io/llumen/",
  ];

  const hasExpected = expectedLinks.some((link) => text.includes(link));
  if (!hasExpected) {
    return grade(false, "Missing official llumen links.");
  }

  for (const rawUrl of urls) {
    let url;
    try {
      url = new URL(rawUrl);
    } catch {
      continue;
    }

    const host = url.hostname.toLowerCase();
    const path = url.pathname;
    if (host === "github.com" && path.toLowerCase().includes("llumen")) {
      if (!path.toLowerCase().startsWith("/pinkfuwa/llumen")) {
        return grade(false, `Suspicious llumen GitHub link: ${rawUrl}`);
      }
    }

    if (host.includes("pinkfuwa.github.io") && !path.startsWith("/llumen")) {
      return grade(false, `Suspicious llumen docs link: ${rawUrl}`);
    }
  }

  return grade(true, "Official llumen links look correct.");
}

function assertToolName(output, context) {
  const expectedName = String(context?.config?.expected || "");
  const toolCalls = parseToolCalls(output);
  if (!toolCalls.length) {
    return grade(false, "No tool call found.");
  }

  const firstName = toolCalls[0]?.function?.name || toolCalls[0]?.name || "";
  return grade(
    firstName === expectedName,
    `Tool name=${firstName || "<empty>"}, expected=${expectedName}`,
  );
}

function assertNoToolCall(output) {
  const toolCalls = parseToolCalls(output);
  return grade(
    toolCalls.length === 0,
    toolCalls.length === 0 ? "No tool call." : "Unexpected tool call.",
  );
}

function assertMediaPromptLanguage(output, context) {
  const threshold = Number(context?.config?.threshold || 0.8);
  const locale = normalizeLocale(context?.vars?.locale || "en");

  const { prompt } = getFirstToolPrompt(output);
  if (!prompt) {
    return grade(false, "No prompt found in tool call arguments.");
  }

  const cjkCount = countMatches(prompt, CJK_REGEX);
  const latinCount = countMatches(prompt, LATIN_REGEX);
  const denominator = Math.max(1, latinCount + cjkCount);
  const latinRatio = latinCount / denominator;

  if (latinRatio < threshold) {
    return grade(
      false,
      `Prompt is not mostly English. ratio=${latinRatio.toFixed(2)}`,
    );
  }

  if (cjkCount > 0 && (locale === "zh-cn" || locale === "zh-tw")) {
    const variantCheck = detectLanguageRatio(prompt, locale, threshold);
    if (!variantCheck.pass) {
      return grade(false, `Chinese variant mismatch. ${variantCheck.reason}`);
    }
  }

  return grade(
    true,
    `Prompt language check passed. English ratio=${latinRatio.toFixed(2)}`,
  );
}

function assertTitleFormat(output) {
  const text = asText(output).trim();
  if (!text) {
    return grade(false, "Title is empty.");
  }

  if (text.includes("``")) {
    return grade(false, "Title contain codeblock");
  }

  if (text.includes("\n")) {
    return grade(false, "Title is not a single line.");
  }

  const chars = Array.from(text);
  const first = chars[0] || "";
  const second = chars[1] || "";
  const third = chars[2] || "";

  const hasSingleEmojiPrefix =
    (/\p{Extended_Pictographic}/u.test(first) && second === " ") ||
    (isRegionalIndicator(first) &&
      isRegionalIndicator(second) &&
      third === " ");

  if (!hasSingleEmojiPrefix) {
    return grade(false, "Title must start with one emoji and a space.");
  }

  if (/[*#`]/.test(text)) {
    return grade(false, "Title contains markdown markers.");
  }

  return grade(true, "Title format looks valid.");
}

function extractToolPromptOnly(output) {
  return getFirstToolPrompt(output).prompt || "";
}

module.exports = {
  assertNoTaskTag,
  assertLanguageByScript,
  assertChineseOpenccChangeLimit,
  assertOfficialLlumenLinks,
  assertToolName,
  assertNoToolCall,
  assertMediaPromptLanguage,
  assertTitleFormat,
  extractToolPromptOnly,
};
