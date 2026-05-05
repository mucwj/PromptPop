import type { LocaleCode, PromptInput } from "./types";

type StarterSnippetLocale = LocaleCode;

const zhCNStarterSnippetInputs: PromptInput[] = [
  {
    title: "改写更清晰",
    body: "请把下面内容改写得更清晰、更自然，保留原意：",
    alias: "clarify",
    notes: "短片段。适合邮件、文档和产品文案。",
    isFavorite: true,
    tags: ["写作", "改写", "常用"]
  },
  {
    title: "缩短文本",
    body: "请把下面内容压缩得更短，保留关键信息：",
    alias: "shorten",
    notes: "用于把长段落变成更轻的表达。",
    isFavorite: true,
    tags: ["写作", "压缩", "常用"]
  },
  {
    title: "扩写成段落",
    body: "请把下面要点扩写成一段完整、自然的文字：",
    alias: "expand",
    notes: "适合从提纲生成正文。",
    isFavorite: false,
    tags: ["写作", "扩写"]
  },
  {
    title: "语气更专业",
    body: "请用更专业但不生硬的语气改写：",
    alias: "tone",
    notes: "适合工作沟通和客户回复。",
    isFavorite: false,
    tags: ["写作", "语气"]
  },
  {
    title: "总结要点",
    body: "请总结下面内容的关键要点，用简短 bullet 输出：",
    alias: "sum",
    notes: "适合文章、聊天记录和文档摘要。",
    isFavorite: true,
    tags: ["总结", "阅读", "常用"]
  },
  {
    title: "提取行动项",
    body: "请提取行动项，按“事项 / 负责人 / 截止时间 / 状态”输出：",
    alias: "todo",
    notes: "适合会议纪要和项目同步。",
    isFavorite: true,
    tags: ["会议", "行动项", "总结"]
  },
  {
    title: "提取问题风险",
    body: "请指出下面内容里的问题、风险和需要确认的点：",
    alias: "risk",
    notes: "适合方案、PRD 和客户反馈。",
    isFavorite: false,
    tags: ["风险", "分析", "产品"]
  },
  {
    title: "解释概念",
    body: "请用通俗语言解释下面内容，并给一个例子：",
    alias: "explain",
    notes: "适合学习、阅读和技术概念。",
    isFavorite: false,
    tags: ["学习", "解释"]
  },
  {
    title: "给 3 个备选说法",
    body: "请给出 3 个不同表达方式，分别偏简洁、专业、友好：",
    alias: "alt",
    notes: "适合命名、文案和回复措辞。",
    isFavorite: false,
    tags: ["写作", "备选"]
  },
  {
    title: "检查错别字语病",
    body: "请检查错别字、语病和不自然表达，并给出修改版：",
    alias: "proof",
    notes: "适合最终发送前检查。",
    isFavorite: false,
    tags: ["写作", "校对"]
  },
  {
    title: "翻译成自然中文",
    body: "请翻译成自然中文，保留原有语气和格式：",
    alias: "zh",
    notes: "适合英文内容转中文。",
    isFavorite: true,
    tags: ["翻译", "中文", "常用"]
  },
  {
    title: "翻译成自然英文",
    body: "Please translate this into natural English while preserving tone and formatting:",
    alias: "en",
    notes: "Useful for turning Chinese drafts into natural English.",
    isFavorite: false,
    tags: ["翻译", "英文"]
  },
  {
    title: "代码审查",
    body: "请审查下面代码，优先指出 bug、边界情况和缺少的测试：",
    alias: "review",
    notes: "适合提交前自查。",
    isFavorite: true,
    tags: ["代码", "审查", "测试"]
  },
  {
    title: "解释代码",
    body: "请解释这段代码在做什么，重点说明输入、输出和关键逻辑：",
    alias: "code",
    notes: "适合读陌生代码。",
    isFavorite: false,
    tags: ["代码", "解释"]
  },
  {
    title: "生成测试用例",
    body: "请为下面函数/模块生成覆盖主要分支和边界情况的测试用例：",
    alias: "test",
    notes: "适合补测试。",
    isFavorite: false,
    tags: ["代码", "测试"]
  },
  {
    title: "调试分析",
    body: "请根据下面现象分析可能原因，并给出最小排查步骤：",
    alias: "debug",
    notes: "适合错误日志和异常现象。",
    isFavorite: false,
    tags: ["代码", "调试"]
  },
  {
    title: "写简短回复",
    body: "请帮我写一段简短、礼貌、可直接发送的回复：",
    alias: "reply",
    notes: "适合邮件、IM 和客户沟通。",
    isFavorite: true,
    tags: ["回复", "工作", "常用"]
  },
  {
    title: "转成清单",
    body: "请把下面内容整理成清晰的清单：",
    alias: "list",
    notes: "适合整理杂乱信息。",
    isFavorite: false,
    tags: ["整理", "清单"]
  }
];

const enStarterSnippetInputs: PromptInput[] = [
  {
    title: "Rewrite for clarity",
    body: "Rewrite the following text to be clearer and more natural while preserving the meaning:",
    alias: "clarify",
    notes: "Short snippet for emails, docs, and product copy.",
    isFavorite: true,
    tags: ["Writing", "Rewrite", "Common"]
  },
  {
    title: "Shorten text",
    body: "Make the following text shorter while keeping the key information:",
    alias: "shorten",
    notes: "Good for trimming long paragraphs.",
    isFavorite: true,
    tags: ["Writing", "Shorten", "Common"]
  },
  {
    title: "Expand into a paragraph",
    body: "Turn the following notes into one complete, natural paragraph:",
    alias: "expand",
    notes: "Useful when turning rough points into prose.",
    isFavorite: false,
    tags: ["Writing", "Expand"]
  },
  {
    title: "Make tone professional",
    body: "Rewrite this in a more professional but still natural tone:",
    alias: "tone",
    notes: "Useful for work messages and customer replies.",
    isFavorite: false,
    tags: ["Writing", "Tone"]
  },
  {
    title: "Summarize key points",
    body: "Summarize the key points below using short bullets:",
    alias: "sum",
    notes: "Useful for articles, chats, and docs.",
    isFavorite: true,
    tags: ["Summary", "Reading", "Common"]
  },
  {
    title: "Extract action items",
    body: "Extract action items as Task / Owner / Due date / Status:",
    alias: "todo",
    notes: "Useful for meeting notes and project syncs.",
    isFavorite: true,
    tags: ["Meeting", "Action items", "Summary"]
  },
  {
    title: "Find risks and questions",
    body: "Point out issues, risks, and open questions in the following content:",
    alias: "risk",
    notes: "Useful for plans, PRDs, and feedback.",
    isFavorite: false,
    tags: ["Risk", "Analysis", "Product"]
  },
  {
    title: "Explain a concept",
    body: "Explain the following in plain language and include one example:",
    alias: "explain",
    notes: "Useful for learning and technical concepts.",
    isFavorite: false,
    tags: ["Learning", "Explain"]
  },
  {
    title: "Give 3 alternatives",
    body: "Give 3 alternative phrasings: concise, professional, and friendly:",
    alias: "alt",
    notes: "Useful for naming, copy, and replies.",
    isFavorite: false,
    tags: ["Writing", "Alternatives"]
  },
  {
    title: "Proofread",
    body: "Check grammar, typos, and awkward phrasing, then provide a revised version:",
    alias: "proof",
    notes: "Useful before sending final text.",
    isFavorite: false,
    tags: ["Writing", "Proofread"]
  },
  {
    title: "Translate to natural Chinese",
    body: "Translate this into natural Chinese while preserving tone and formatting:",
    alias: "zh",
    notes: "Useful for translating English into Chinese.",
    isFavorite: true,
    tags: ["Translation", "Chinese", "Common"]
  },
  {
    title: "Translate to natural English",
    body: "Translate this into natural English while preserving tone and formatting:",
    alias: "en",
    notes: "Useful for translating Chinese drafts into English.",
    isFavorite: false,
    tags: ["Translation", "English"]
  },
  {
    title: "Code review",
    body: "Review the code below. Prioritize bugs, edge cases, and missing tests:",
    alias: "review",
    notes: "Useful before submitting a change.",
    isFavorite: true,
    tags: ["Code", "Review", "Test"]
  },
  {
    title: "Explain code",
    body: "Explain what this code does, focusing on inputs, outputs, and key logic:",
    alias: "code",
    notes: "Useful when reading unfamiliar code.",
    isFavorite: false,
    tags: ["Code", "Explain"]
  },
  {
    title: "Generate test cases",
    body: "Generate tests for the function/module below, covering main branches and edge cases:",
    alias: "test",
    notes: "Useful when adding focused coverage.",
    isFavorite: false,
    tags: ["Code", "Test"]
  },
  {
    title: "Debug analysis",
    body: "Analyze likely causes from the symptoms below and give the smallest debugging steps:",
    alias: "debug",
    notes: "Useful for logs and unexpected behavior.",
    isFavorite: false,
    tags: ["Code", "Debug"]
  },
  {
    title: "Write a short reply",
    body: "Write a brief, polite reply that I can send directly:",
    alias: "reply",
    notes: "Useful for email, chat, and customer communication.",
    isFavorite: true,
    tags: ["Reply", "Work", "Common"]
  },
  {
    title: "Turn into a checklist",
    body: "Organize the following content into a clear checklist:",
    alias: "list",
    notes: "Useful for cleaning up messy information.",
    isFavorite: false,
    tags: ["Organize", "Checklist"]
  }
];

export const starterSnippetInputsByLocale: Record<StarterSnippetLocale, PromptInput[]> = {
  en: enStarterSnippetInputs,
  "zh-CN": zhCNStarterSnippetInputs
};

export function normalizeStarterSnippetLocale(locale: string | null | undefined): StarterSnippetLocale {
  return locale?.toLowerCase().startsWith("zh") ? "zh-CN" : "en";
}

export function starterSnippetInputsForLocale(locale: string | null | undefined): PromptInput[] {
  return starterSnippetInputsByLocale[normalizeStarterSnippetLocale(locale)];
}

export const STARTER_SNIPPET_COUNT = enStarterSnippetInputs.length;
