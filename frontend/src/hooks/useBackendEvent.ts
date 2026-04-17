import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef } from "react";

export function useBackendEvent<T>(eventName: string, callback: (payload: T) => void) {
  const callbackRef = useRef(callback);
  callbackRef.current = callback;

  useEffect(() => {
    const unlisten = listen<T>(eventName, (event) => {
      callbackRef.current(event.payload);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [eventName]);
}
