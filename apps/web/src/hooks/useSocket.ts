import { createSignal, createEffect, onCleanup, onMount } from "solid-js";

const WS_BASE = import.meta.env.VITE_WS_BASE ?? "ws://localhost:7070";

type ConnectionStatus = "connecting" | "connected" | "disconnected" | "error";

interface SocketState {
  status: ConnectionStatus;
  lastMessage: any | null;
  error: string | null;
}

interface ClientEvent {
  type: "subscribe" | "unsubscribe" | "message" | "typing";
  chat_id?: string;
  content?: string;
  is_typing?: boolean;
}

interface ServerEvent {
  type: "hello" | "subscribed" | "unsubscribed" | "message" | "typing" | "error";
  version?: string;
  chat_id?: string;
  message_id?: string;
  user_id?: string;
  content?: string;
  timestamp?: string;
  is_typing?: boolean;
  message?: string;
}

export function useSocket() {
  const [state, setState] = createSignal<SocketState>({
    status: "disconnected",
    lastMessage: null,
    error: null,
  });

  let socket: WebSocket | null = null;
  let reconnectTimeout: number | null = null;
  let reconnectAttempts = 0;
  const maxReconnectAttempts = 5;
  const reconnectDelay = 1000; // Start with 1 second

  const connect = () => {
    if (socket?.readyState === WebSocket.OPEN) return;

    setState(prev => ({ ...prev, status: "connecting", error: null }));

    try {
      socket = new WebSocket(`${WS_BASE}/ws`);

      socket.onopen = () => {
        console.log("WebSocket connected");
        setState(prev => ({ ...prev, status: "connected", error: null }));
        reconnectAttempts = 0;
      };

      socket.onmessage = (event) => {
        try {
          const data: ServerEvent = JSON.parse(event.data);
          console.log("WebSocket message received:", data);
          setState(prev => ({ ...prev, lastMessage: data }));

          // Handle specific events
          if (data.type === "error") {
            setState(prev => ({ ...prev, error: data.message || "Unknown error" }));
          }
        } catch (error) {
          console.error("Failed to parse WebSocket message:", error);
          setState(prev => ({ ...prev, error: "Invalid message format" }));
        }
      };

      socket.onclose = (event) => {
        console.log("WebSocket disconnected:", event.code, event.reason);
        setState(prev => ({ ...prev, status: "disconnected" }));
        socket = null;

        // Attempt to reconnect if not a normal closure
        if (event.code !== 1000 && reconnectAttempts < maxReconnectAttempts) {
          scheduleReconnect();
        }
      };

      socket.onerror = (error) => {
        console.error("WebSocket error:", error);
        setState(prev => ({ ...prev, status: "error", error: "Connection failed" }));
      };

    } catch (error) {
      console.error("Failed to create WebSocket:", error);
      setState(prev => ({ ...prev, status: "error", error: "Failed to create connection" }));
    }
  };

  const disconnect = () => {
    if (reconnectTimeout) {
      clearTimeout(reconnectTimeout);
      reconnectTimeout = null;
    }

    if (socket) {
      socket.close(1000, "Client disconnect");
      socket = null;
    }

    setState(prev => ({ ...prev, status: "disconnected" }));
  };

  const send = (event: ClientEvent) => {
    if (socket?.readyState === WebSocket.OPEN) {
      socket.send(JSON.stringify(event));
    } else {
      console.warn("WebSocket not connected, cannot send message");
    }
  };

  const subscribe = (chatId: string) => {
    send({ type: "subscribe", chat_id: chatId });
  };

  const unsubscribe = (chatId: string) => {
    send({ type: "unsubscribe", chat_id: chatId });
  };

  const sendMessage = (chatId: string, content: string) => {
    send({ type: "message", chat_id: chatId, content });
  };

  const sendTyping = (chatId: string, isTyping: boolean) => {
    send({ type: "typing", chat_id: chatId, is_typing: isTyping });
  };

  const scheduleReconnect = () => {
    if (reconnectTimeout) return;

    reconnectAttempts++;
    const delay = reconnectDelay * Math.pow(2, reconnectAttempts - 1); // Exponential backoff

    console.log(`Scheduling reconnect attempt ${reconnectAttempts}/${maxReconnectAttempts} in ${delay}ms`);

    reconnectTimeout = window.setTimeout(() => {
      reconnectTimeout = null;
      connect();
    }, delay);
  };

  // Auto-connect on mount
  onMount(() => {
    connect();
  });

  // Cleanup on unmount
  onCleanup(() => {
    disconnect();
  });

  return {
    state,
    connect,
    disconnect,
    send,
    subscribe,
    unsubscribe,
    sendMessage,
    sendTyping,
  };
}