import { useCallback, useLayoutEffect, useRef, useState } from "react";

type UseExpandableOverflowOptions = {
  measureKey?: unknown;
  resetKey?: unknown;
};

export function useExpandableOverflow<T extends HTMLElement>({
  measureKey,
  resetKey,
}: UseExpandableOverflowOptions = {}) {
  const contentRef = useRef<T | null>(null);
  const [expanded, setExpanded] = useState(false);
  const [overflows, setOverflows] = useState(false);

  const toggleExpanded = useCallback(() => {
    setExpanded((value) => !value);
  }, []);

  useLayoutEffect(() => {
    setExpanded(false);
  }, [resetKey]);

  useLayoutEffect(() => {
    const el = contentRef.current;
    if (!el) {
      setOverflows(false);
      return;
    }

    const measure = () => {
      if (expanded) return;
      setOverflows(el.scrollWidth > el.clientWidth + 1 || el.scrollHeight > el.clientHeight + 1);
    };

    measure();

    const observer = new ResizeObserver(measure);
    observer.observe(el);

    return () => observer.disconnect();
  }, [expanded, measureKey]);

  return { contentRef, expanded, overflows, toggleExpanded };
}
