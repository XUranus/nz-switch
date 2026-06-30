import { useState, useRef, useCallback, useEffect } from "react";

export interface ToastState {
  text: string;
  type: "ok" | "error";
}

export function useToast(duration = 3000) {
  const [message, setMessage] = useState<ToastState | null>(null);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    return () => {
      if (timerRef.current) clearTimeout(timerRef.current);
    };
  }, []);

  const showToast = useCallback((text: string, type: "ok" | "error") => {
    if (timerRef.current) clearTimeout(timerRef.current);
    setMessage({ text, type });
    timerRef.current = setTimeout(() => setMessage(null), duration);
  }, [duration]);

  return { message, showToast };
}
