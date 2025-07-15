// Tauri API type declarations
declare global {
  interface Window {
    __TAURI__?: {
      invoke: (cmd: string, args?: Record<string, unknown>) => Promise<unknown>;
      [key: string]: unknown;
    };
  }
}

// Proxy status interface
export interface ProxyStatus {
  is_running: boolean;
  port: number;
  active_domains: string[];
  domains_count: number;
}

// Proxy log entry interface
export interface ProxyLogEntry {
  timestamp: string;
  original_url: string;
  redirected_url: string;
}

// Activity entry interface
export interface ActivityEntry {
  id: string;
  timestamp: string;
  actionType: string;
  fileName?: string;
  identifier?: string;
  status?: string;
  details?: string;
}

// Partial download info interface
export interface PartialDownloadInfo {
  id: string;
  file_path: string;
  downloaded_bytes: number;
  total_bytes: number;
  last_modified: string;
  checksum: string;
}

// Tauri command types
export interface TauriCommands {
  start_proxy(): Promise<string>;
  stop_proxy(): Promise<string>;
  check_proxy_status(): Promise<boolean>;
  force_stop_proxy(): Promise<string>;
  get_active_proxy_domains(): Promise<string[]>;
  get_proxy_status_with_domains(): Promise<ProxyStatus>;
  get_proxy_logs(): Promise<ProxyLogEntry[]>;
  clear_proxy_logs(): Promise<string>;
  add_proxy_domain(domain: string): Promise<string>;
  remove_proxy_domain(domain: string): Promise<string>;
  get_user_proxy_domains(): Promise<string[]>;
  get_all_proxy_domains(): Promise<string[]>;
  initialize_user_domains_if_empty(): Promise<string[]>;
  check_and_disable_windows_proxy(): Promise<string>;
  install_ssl_certificate(): Promise<string>;
  check_ssl_certificate_installed(): Promise<boolean>;
  clear_launcher_data(): Promise<string>;
  get_yuukips_data_path(): Promise<string>;
  get_app_data_path(): Promise<string>;
  get_temp_files_path(): Promise<string>;
  open_devtools(): Promise<string>;
  generate_ca_files(path: string): Promise<void>;
  launch_game(
    gameId: number,
    version: string,
    channel: number,
    gameFolderPath: string,
    deleteHoyoPass?: boolean
  ): Promise<string>;
  validate_game_directory(
    gameId: number,
    channel: number,
    gameFolderPath: string
  ): Promise<string>;
  get_game_folder_path(gameId: number, version: string): Promise<string>;
  check_game_installed(gameId: number, version: string, gameFolderPath: string): Promise<boolean>;
  open_directory(path: string): Promise<string>;
  start_game_monitor(gameId: number): Promise<string>;
  stop_game_monitor(): Promise<string>;
  is_game_monitor_active(): Promise<boolean>;
  force_stop_game_monitor(): Promise<string>;
  fetch_patch_info_command(
    gameId: number,
    version: string,
    channel: number,
    md5: string
  ): Promise<string>;
  get_available_drives(): Promise<DriveInfo[]>;
  scan_drive_for_games(
    drive: string,
    gameId: number,
    channel: number
  ): Promise<string[]>;
  get_all_game_name_codes(): Promise<[number, number, string][]>
  get_game_md5(path: string): Promise<string>
  get_hoyoplay_list_game(): Promise<Array<[string, string]>>;
  // Activity commands
  get_activities(): Promise<ActivityEntry[]>;
  clear_activities(): Promise<void>;
  add_user_interaction_activity(action: string, details?: string): Promise<void>;
  // State management commands
  save_download_state(): Promise<void>;
  load_download_state(): Promise<void>;
  resume_interrupted_downloads(): Promise<string[]>;
  get_state_version(): Promise<number>;
  set_auto_save_enabled(enabled: boolean): Promise<void>;
  get_partial_downloads(): Promise<Record<string, PartialDownloadInfo>>;
  // Speed limit commands
  get_speed_limit(): Promise<number>;
  set_speed_limit(speedLimitMbps: number): Promise<void>;
  // Divide speed commands
  get_divide_speed_enabled(): Promise<boolean>;
  set_divide_speed_enabled(enabled: boolean): Promise<void>;
}

export interface DriveInfo {
  letter: string;
  name: string;
  total_size: number;
  free_size: number;
  drive_type: string;
}

export interface ScanProgress {
  current_path: string;
  files_scanned: number;
  directories_scanned: number;
  found_paths: string[];
}

export {};