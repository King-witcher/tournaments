/** Navegador e sistema de um `User-Agent`, só para a pessoa reconhecer o aparelho. */
export type Device = { browser?: string; os?: string };

// Ordem importa: Edge, Opera e Samsung dizem "Chrome"; o Chrome diz "Safari".
const BROWSERS: [RegExp, string][] = [
  [/Edg(?:e|A|iOS)?\//, "Edge"],
  [/OPR\/|Opera/, "Opera"],
  [/SamsungBrowser\//, "Samsung Internet"],
  [/Firefox\/|FxiOS\//, "Firefox"],
  [/Chrome\/|CriOS\//, "Chrome"],
  [/Safari\//, "Safari"],
];

// Android antes de Linux; iOS antes de macOS (o UA do iPhone cita "like Mac OS X").
const SYSTEMS: [RegExp, string][] = [
  [/Windows/, "Windows"],
  [/Android/, "Android"],
  [/iPhone|iPad|iPod/, "iOS"],
  [/Mac OS X|Macintosh/, "macOS"],
  [/CrOS/, "ChromeOS"],
  [/Linux/, "Linux"],
];

/** Lê o navegador e o sistema; o que não reconhece fica de fora. */
export function parseUserAgent(userAgent: string | null | undefined): Device {
  if (!userAgent) return {};
  return {
    browser: BROWSERS.find(([re]) => re.test(userAgent))?.[1],
    os: SYSTEMS.find(([re]) => re.test(userAgent))?.[1],
  };
}
