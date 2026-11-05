import type { NextFetchEvent, NextRequest } from "next/server";

import { internalHeaders } from "@/shared/api/internal";
import { API_URL } from "@/shared/api/url";

import { SESSION_COOKIE } from "../../auth/lib/session-cookie";
import { isScanner, isTrap } from "./traps";

/** A cada quanto, em cada instância, a lista é relida da API. */
const REFRESH_MS = 5 * 60 * 1000;

const TIMEOUT_MS = 3000;

const BANS_URL = `${API_URL}/internal/bans`;

/**
 * A lista de IPs banidos, numa variável do módulo: só uma cópia. A instância que
 * sobe sem ela deixa passar até a primeira leitura, em segundo plano: o proxy não
 * espera a API, para a apresentação não esperar. Um scanner pede tudo em
 * sequência e cai na instância que acabou de banir o IP dele.
 */
let banned = new Set<string>();
let refreshAfter = 0;

/**
 * O IP com a Vercel na frente, que grava este cabeçalho por cima do que o cliente
 * mandar; fora dela, ele não vem e nada é banido. Em `ipBucket`, a mesma divisão
 * de `ClientIp::bucket` da API: o IPv4 inteiro, o IPv6 pelo /64.
 */
function ipBucket(ip: string): string | null {
  const mapped = ip.match(/^::ffff:(\d+\.\d+\.\d+\.\d+)$/i);
  if (mapped) {
    return mapped[1];
  }
  if (!ip.includes(":")) {
    return ip;
  }
  const [head, tail] = ip.split("::");
  const left = head ? head.split(":") : [];
  const right = tail ? tail.split(":") : [];
  const zeros = Array(Math.max(8 - left.length - right.length, 0)).fill("0");
  const block = [...left, ...zeros, ...right]
    .slice(0, 4)
    .map((segment) => Number.parseInt(segment, 16));
  // Endereço que sobra zerado no /64 não é de ninguém em especial.
  if (block.length < 4 || block.every((segment) => !segment)) {
    return null;
  }
  return `${block.map((segment) => segment.toString(16)).join(":")}::/64`;
}

function refreshBans(event: NextFetchEvent) {
  if (Date.now() < refreshAfter) {
    return;
  }
  refreshAfter = Date.now() + REFRESH_MS;
  event.waitUntil(
    fetch(BANS_URL, {
      headers: internalHeaders(),
      signal: AbortSignal.timeout(TIMEOUT_MS),
    })
      .then(async (response) => {
        if (response.ok) {
          banned = new Set(await response.json());
        }
      })
      // Sem a API, segue com a lista que tem.
      .catch(() => {}),
  );
}

function reportBan(ip: string, event: NextFetchEvent) {
  event.waitUntil(
    fetch(BANS_URL, {
      method: "POST",
      headers: internalHeaders(),
      body: ip,
      signal: AbortSignal.timeout(TIMEOUT_MS),
    }).catch(() => {}),
  );
}

/**
 * A resposta para um scanner (404 sem nada, a de quem está banido e a do pedido
 * que o denunciou) ou `null` para seguir. Sem `INTERNAL_API_SECRET`, desligado.
 * Quem tem cookie de sessão passa, como na API: é quem divide o IP com o scanner.
 * Motivos em `docs/decisions/stack.md`, "Scanners".
 */
export function refuseScanner(
  request: NextRequest,
  event: NextFetchEvent,
): Response | null {
  const ip = request.headers.get("x-real-ip");
  const bucket = ip && ipBucket(ip);
  if (!ip || !bucket || !process.env.INTERNAL_API_SECRET) {
    return null;
  }
  refreshBans(event);
  if (request.cookies.has(SESSION_COOKIE)) {
    return null;
  }
  if (banned.has(bucket)) {
    return new Response(null, { status: 404 });
  }
  if (isTrap(request.nextUrl.pathname) && isScanner(request.headers)) {
    banned.add(bucket);
    reportBan(ip, event);
    return new Response(null, { status: 404 });
  }
  return null;
}
