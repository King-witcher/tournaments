/**
 * O que este navegador permite: Web Push (`supported`), só depois de
 * instalar o site na tela inicial (`install`: iPhone e iPad fora do modo
 * instalado) ou nada (`unsupported`). Só no navegador.
 */
export type PushSupport = "supported" | "install" | "unsupported";

/** iPhone e iPad (o iPad se apresenta como Mac, mas tem toque). */
function isIos() {
  const ua = navigator.userAgent;
  return (
    /iPad|iPhone|iPod/.test(ua) ||
    (ua.includes("Macintosh") && navigator.maxTouchPoints > 1)
  );
}

/** Aberto pelo ícone da tela inicial. */
function isInstalled() {
  return (
    window.matchMedia("(display-mode: standalone)").matches ||
    (navigator as Navigator & { standalone?: boolean }).standalone === true
  );
}

export function pushSupport(): PushSupport {
  if (isIos() && !isInstalled()) return "install";
  return "serviceWorker" in navigator &&
    "PushManager" in window &&
    "Notification" in window
    ? "supported"
    : "unsupported";
}
