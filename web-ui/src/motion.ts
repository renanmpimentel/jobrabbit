// Animation helpers (framer-motion) reused across screens.
import { useEffect, useRef, useState } from "react";
import type { Variants } from "framer-motion";

const EASE = [0.22, 1, 0.36, 1] as const;

export const fadeUp: Variants = {
  hidden: { opacity: 0, y: 14 },
  show: { opacity: 1, y: 0, transition: { duration: 0.45, ease: EASE } },
};

export const fadeIn: Variants = {
  hidden: { opacity: 0 },
  show: { opacity: 1, transition: { duration: 0.4, ease: EASE } },
};

export const stagger: Variants = {
  hidden: {},
  show: { transition: { staggerChildren: 0.06, delayChildren: 0.03 } },
};

/// Animates a number from 0 → target (cubic ease-out). Used in stat cards.
export function useCountUp(target: number, duration = 750): number {
  const [val, setVal] = useState(0);
  const from = useRef(0);

  useEffect(() => {
    const start = performance.now();
    const begin = from.current;
    let raf = 0;
    const tick = (t: number) => {
      const p = Math.min(1, (t - start) / duration);
      const eased = 1 - Math.pow(1 - p, 3);
      setVal(Math.round(begin + (target - begin) * eased));
      if (p < 1) raf = requestAnimationFrame(tick);
      else from.current = target;
    };
    raf = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(raf);
  }, [target, duration]);

  return val;
}
