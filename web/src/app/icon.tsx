import { ImageResponse } from "next/og";

import { BrandMark } from "@/shared/lib/brand-mark";

export const size = { width: 512, height: 512 };
export const contentType = "image/png";

/** O ícone do site (aba, manifest, notificação), gerado no build. */
export default function Icon() {
  return new ImageResponse(<BrandMark size={size.width} />, size);
}
