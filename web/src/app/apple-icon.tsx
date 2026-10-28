import { ImageResponse } from "next/og";

import { BrandMark } from "@/shared/lib/brand-mark";

export const size = { width: 180, height: 180 };
export const contentType = "image/png";

/** O ícone da tela inicial do iPhone e do iPad, gerado no build. */
export default function AppleIcon() {
  return new ImageResponse(<BrandMark size={size.width} />, size);
}
