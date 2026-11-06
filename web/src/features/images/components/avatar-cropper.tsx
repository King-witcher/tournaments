"use client";

import {
  type KeyboardEvent,
  type PointerEvent,
  useEffect,
  useRef,
  useState,
} from "react";

import { type Crop, clampCrop, drawCrop, zoomTo } from "../lib/crop-image";
import styles from "./avatar-cropper.module.scss";

/** Quanto uma seta move a foto, em frações do lado (com Shift, `BIG_STEP`). */
const STEP = 0.02;
const BIG_STEP = 0.1;
/** Quanto cada tecla `+`/`-` muda o zoom. */
const ZOOM_STEP = 1.1;
/** Quanto da roda do mouse dobra o zoom (`deltaY` em pixels). */
const WHEEL_SCALE = 500;

type Props = {
  image: ImageBitmap;
  crop: Crop;
  onChange: (crop: Crop) => void;
  /** O nome acessível da área, com como mexer nela pelo teclado. */
  label: string;
};

type Point = { x: number; y: number };

/**
 * A área de recorte da foto: a imagem num quadrado, com o círculo que o site
 * mostra. Arrastar (ou as setas) move a foto; a roda do mouse, o pinçar e
 * `+`/`-` mudam o zoom; o recorte sempre cobre o quadrado (`clampCrop`).
 * Recebe o foco ao abrir o Modal (`data-modal-primary-focus`).
 */
export function AvatarCropper({ image, crop, onChange, label }: Props) {
  const canvas = useRef<HTMLCanvasElement>(null);
  const [side, setSide] = useState(0);
  // Os gestos partem do recorte mais novo: vários eventos chegam antes do
  // próximo render.
  const latest = useRef(crop);
  const pointers = useRef(new Map<number, Point>());

  useEffect(() => {
    latest.current = crop;
  }, [crop]);

  const update = (next: (current: Crop) => Crop) => {
    latest.current = next(latest.current);
    onChange(latest.current);
  };
  const updateRef = useRef(update);
  updateRef.current = update;

  // O canvas em pixels da tela (nítido em tela de alta densidade).
  useEffect(() => {
    const element = canvas.current;
    if (!element) return;
    const observer = new ResizeObserver(([entry]) => {
      const width = entry?.contentRect.width ?? 0;
      setSide(Math.round(width * window.devicePixelRatio));
    });
    observer.observe(element);
    return () => observer.disconnect();
  }, []);

  useEffect(() => {
    const element = canvas.current;
    const context = element?.getContext("2d");
    if (!element || !context || side === 0) return;
    element.width = side;
    element.height = side;
    context.clearRect(0, 0, side, side);
    drawCrop(context, image, crop, side);
  }, [image, crop, side]);

  // A roda do mouse fora do React: o ouvinte dele é passivo e não impede a
  // página de rolar.
  useEffect(() => {
    const element = canvas.current;
    if (!element) return;
    const onWheel = (event: WheelEvent) => {
      event.preventDefault();
      const factor = Math.exp(-event.deltaY / WHEEL_SCALE);
      updateRef.current((c) => zoomTo(image, c, c.zoom * factor));
    };
    element.addEventListener("wheel", onWheel, { passive: false });
    return () => element.removeEventListener("wheel", onWheel);
  }, [image]);

  const move = (dx: number, dy: number) =>
    update((c) => clampCrop(image, { ...c, x: c.x + dx, y: c.y + dy }));
  const zoomBy = (factor: number) =>
    update((c) => zoomTo(image, c, c.zoom * factor));

  const onPointerDown = (event: PointerEvent<HTMLCanvasElement>) => {
    event.currentTarget.setPointerCapture(event.pointerId);
    pointers.current.set(event.pointerId, {
      x: event.clientX,
      y: event.clientY,
    });
  };

  const onPointerMove = (event: PointerEvent<HTMLCanvasElement>) => {
    const before = pointers.current.get(event.pointerId);
    if (!before) return;
    const width = event.currentTarget.clientWidth || 1;
    const now = { x: event.clientX, y: event.clientY };
    pointers.current.set(event.pointerId, now);
    if (pointers.current.size === 1) {
      move((now.x - before.x) / width, (now.y - before.y) / width);
      return;
    }
    // Dois dedos: o zoom acompanha a distância entre eles.
    const other = [...pointers.current].find(([id]) => id !== event.pointerId);
    if (!other) return;
    const [, fixed] = other;
    const was = Math.hypot(before.x - fixed.x, before.y - fixed.y);
    const is = Math.hypot(now.x - fixed.x, now.y - fixed.y);
    if (was > 0) zoomBy(is / was);
  };

  const onPointerUp = (event: PointerEvent<HTMLCanvasElement>) => {
    pointers.current.delete(event.pointerId);
  };

  const onKeyDown = (event: KeyboardEvent<HTMLCanvasElement>) => {
    const step = event.shiftKey ? BIG_STEP : STEP;
    switch (event.key) {
      case "ArrowLeft":
        move(-step, 0);
        break;
      case "ArrowRight":
        move(step, 0);
        break;
      case "ArrowUp":
        move(0, -step);
        break;
      case "ArrowDown":
        move(0, step);
        break;
      case "+":
      case "=":
        zoomBy(ZOOM_STEP);
        break;
      case "-":
        zoomBy(1 / ZOOM_STEP);
        break;
      default:
        return;
    }
    event.preventDefault();
  };

  return (
    <div className={styles.frame}>
      <canvas
        ref={canvas}
        className={styles.canvas}
        role="img"
        aria-label={label}
        tabIndex={0}
        data-modal-primary-focus
        onPointerDown={onPointerDown}
        onPointerMove={onPointerMove}
        onPointerUp={onPointerUp}
        onPointerCancel={onPointerUp}
        onKeyDown={onKeyDown}
      />
      <div className={styles.mask} />
    </div>
  );
}
