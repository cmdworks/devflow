import { useEffect, useRef, useCallback } from "react";
import { getApiBase } from "./useDevFlowApi";
import type { ServerEvent } from "../types";

export type EventHandler = (event: ServerEvent) => void;

export function useDevFlowEvents(onEvent?: EventHandler) {
  const handlerRef = useRef<EventHandler | undefined>(onEvent);
  handlerRef.current = onEvent;

  const eventSourceRef = useRef<EventSource | null>(null);

  const connect = useCallback(() => {
    if (eventSourceRef.current) {
      eventSourceRef.current.close();
    }

    const apiBase = getApiBase();
    const es = new EventSource(`${apiBase}/api/events`);

    es.onmessage = (msg) => {
      try {
        const parsed: ServerEvent = JSON.parse(msg.data);
        if (handlerRef.current) {
          handlerRef.current(parsed);
        }
      } catch (err) {
        console.warn("Failed to parse SSE event:", err);
      }
    };

    es.onerror = () => {
      // EventSource handles automatic reconnect
    };

    eventSourceRef.current = es;
  }, []);

  useEffect(() => {
    connect();
    return () => {
      if (eventSourceRef.current) {
        eventSourceRef.current.close();
        eventSourceRef.current = null;
      }
    };
  }, [connect]);

  return { reconnect: connect };
}
