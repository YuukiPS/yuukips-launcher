// Tauri API type declarations
declare global {
  interface Window {
    __TAURI__?: {
      invoke: (cmd: string, args?: Record<string, unknown>) => Promise<unknown>;
      [key: string]: unknown;
    };
  }
}

// Tauri command types
export interface TauriCommands {
  start_proxy(): Promise<string>;
  stop_proxy(): Promise<string>;
  check_proxy_status(): Promise<boolean>;
  force_stop_proxy(): Promise<string>;
  check_and_disable_windows_proxy(): Promise<string>;
  check_admin_privileges(): Promise<boolean>;
  install_ssl_certificate(): Promise<string>;
  check_ssl_certificate_installed(): Promise<boolean>;
  launch_game(gameId: number, gameTitle: string): Promise<string>;
  launch_game_with_engine(
    gameId: number,
    gameTitle: string,
    engineId: number,
    engineName: string,
    version: string,
    gameFolderPath: string
  ): Promise<string>;
  get_game_folder_path(gameId: number, version: string): Promise<string>;
  show_game_folder(gameId: number): Promise<string>;
  check_game_installed(gameId: number, version: string, gameFolderPath: string): Promise<boolean>;
  open_directory(path: string): Promise<string>;
}

export {};