import { useState } from "react";

/** `copy(texto)` põe o texto na área de transferência; depois, `copied` fica true. */
export function useCopyToClipboard() {
  const [copied, setCopied] = useState(false);

  function copy(text: string) {
    navigator.clipboard.writeText(text).then(() => setCopied(true));
  }

  return { copied, copy };
}
