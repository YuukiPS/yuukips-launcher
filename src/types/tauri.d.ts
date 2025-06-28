// Tauri API type declarations
declare global {
  interface Window {
    __TAURI__?: {
      invoke: (cmd: string, args?: Record<string, unknown>) => Promise<unknown>;
      [key: string]: unknown;
    };
  }
}

export {};