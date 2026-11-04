/**
 * Filtros para a prop `filter` dos campos de texto: tiram o que o campo não
 * aceita enquanto a pessoa digita ou cola. Devolvem o texto como deve ficar.
 */

/** Só dígitos. */
export function digitsOnly(value: string) {
  return value.replace(/\D/g, "");
}

type Part = { length: number; max?: number };

/**
 * Dígitos em partes separadas por `separator` (dia/mês/ano, hora:minuto),
 * pondo o separador sozinho quando o dígito seguinte não cabe na parte:
 * ela já está cheia ("13" + "3" dá "13:3") ou o número passaria do máximo
 * ("9" + "3" dá "9:3", porque 93 não é hora). O separador digitado também
 * passa para a parte seguinte ("1/2/2026"). O resto não entra; apagar não
 * repõe nada, porque só um dígito novo põe o separador.
 */
function splitDigits(value: string, parts: Part[], separator: string) {
  let out = "";
  let index = 0;
  let current = "";
  for (const char of value) {
    const last = index === parts.length - 1;
    if (char === separator) {
      if (current && !last) {
        out += separator;
        index += 1;
        current = "";
      }
      continue;
    }
    if (!/\d/.test(char)) continue;
    const { length, max } = parts[index];
    const overflows =
      current.length === length ||
      (max !== undefined && Number(current + char) > max);
    if (current && overflows) {
      if (last) continue;
      out += separator;
      index += 1;
      current = "";
    }
    current += char;
    out += char;
  }
  return out;
}

/** Hora "hh:mm", com os dois-pontos postos sozinhos (o teclado numérico do celular não tem). */
export function timeDigits(value: string) {
  return splitDigits(value, [{ length: 2, max: 23 }, { length: 2 }], ":");
}

/** Data "dd/mm/aaaa" ("mm/dd/aaaa" com `monthFirst`), com as barras postas sozinhas. */
export function dateDigits(value: string, monthFirst: boolean) {
  const day = { length: 2, max: 31 };
  const month = { length: 2, max: 12 };
  return splitDigits(
    value,
    monthFirst ? [month, day, { length: 4 }] : [day, month, { length: 4 }],
    "/",
  );
}

/** Identificador na URL (slug): a-z, 0-9 e hífen simples, sem hífen no começo. */
export function slugChars(value: string) {
  return value
    .toLowerCase()
    .replace(/[^a-z0-9-]/g, "")
    .replace(/-{2,}/g, "-")
    .replace(/^-/, "");
}
