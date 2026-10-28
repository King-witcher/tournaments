// Gera um par de chaves VAPID para o Web Push, sem ferramenta global.
// Uso, em backend/: bun scripts/vapid-keys.ts
//
// Copie as duas linhas para o backend/.env (e para as variáveis do Render).
// A pública vai para o navegador; a privada assina os avisos e é segredo.
// Trocar o par invalida as inscrições dos navegadores: cada um precisa
// ativar os avisos de novo.

const pair = await crypto.subtle.generateKey(
  { name: "ECDSA", namedCurve: "P-256" },
  true,
  ["sign", "verify"],
);
// Ponto P-256 sem compressão (65 bytes), o `applicationServerKey` do navegador.
const publicKey = new Uint8Array(
  await crypto.subtle.exportKey("raw", pair.publicKey),
);
// O escalar privado (32 bytes), já em base64url no JWK.
const { d } = await crypto.subtle.exportKey("jwk", pair.privateKey);

console.log(`VAPID_PUBLIC_KEY=${Buffer.from(publicKey).toString("base64url")}`);
console.log(`VAPID_PRIVATE_KEY=${d}`);
