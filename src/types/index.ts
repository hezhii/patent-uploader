// 通用接口定义
export interface ScanResult {
  file_count: number;
  total_size: number;
  files: string[];
}

export interface ColumnMapping {
  original: string;
  mapped: string;
}

export interface ServerConfig {
  serverUrl: string;
  username: string;
  password: string;
}

// HTTP 相关接口
export interface LoginRequest {
  username: string;
  password: string;
}

export interface LoginResponse {
  success: boolean;
  data: {
    id: number;
    username: string;
    token: string;
  };
}

export interface UploadResponse {
  success: boolean;
  data: {
    modifiedCount: number;
    upsertedCount: number;
    excelCount: number;
  };
}

// 上传进度接口
export interface UploadProgress {
  fileIndex: number;
  fileName: string;
  progress: number; // 0-100
  status: 'pending' | 'uploading' | 'completed' | 'failed';
  error?: string;
  result?: UploadResponse;
}

// 日志接口
export interface LogEntry {
  id: string;
  timestamp: number;
  level: 'info' | 'warn' | 'error' | 'success';
  message: string;
}

// 连接状态
export interface ConnectionStatus {
  type: 'success' | 'warning' | 'error' | 'info';
  message: string;
}