import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { save } from '@tauri-apps/plugin-dialog';
import type { LogEntry } from '@/types';
import { generateId, formatTime } from '@/utils';

export const useLoggerStore = defineStore('logger', () => {
  // 状态
  const logs = ref<LogEntry[]>([]);
  const logFilter = ref<'all' | 'info' | 'warn' | 'error' | 'success'>('all');
  const searchKeyword = ref('');
  const autoScroll = ref(true);

  // 计算属性
  const filteredLogs = computed(() => {
    let filtered = logs.value;

    // 按级别过滤
    if (logFilter.value !== 'all') {
      filtered = filtered.filter(log => log.level === logFilter.value);
    }

    // 按关键词搜索
    if (searchKeyword.value) {
      const keyword = searchKeyword.value.toLowerCase();
      filtered = filtered.filter(log => 
        log.message.toLowerCase().includes(keyword)
      );
    }

    return filtered;
  });

  // 方法
  function addLog(level: LogEntry['level'], message: string) {
    const logEntry: LogEntry = {
      id: generateId(),
      timestamp: Date.now(),
      level,
      message
    };

    logs.value.push(logEntry);

    // 限制日志数量，避免内存溢出
    if (logs.value.length > 1000) {
      logs.value = logs.value.slice(-800); // 保留最近的800条
    }
  }

  function info(message: string) {
    addLog('info', message);
  }

  function warn(message: string) {
    addLog('warn', message);
  }

  function error(message: string) {
    addLog('error', message);
  }

  function success(message: string) {
    addLog('success', message);
  }

  function clearLogs() {
    logs.value = [];
  }

  async function exportLogs(): Promise<void> {
    const logText = logs.value.map(log => {
      const time = formatTime(log.timestamp);
      return `[${time}] [${log.level.toUpperCase()}] ${log.message}`;
    }).join('\n');

    try {
      // 使用 Tauri 对话框选择保存位置
      const filePath = await save({
        title: '导出日志',
        defaultPath: `patent-upload-logs-${Date.now()}.txt`,
        filters: [{
          name: 'Text',
          extensions: ['txt']
        }]
      });

      if (filePath) {
        // 使用 Tauri 的 invoke 调用 Rust 命令保存文件
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('save_log_file', { 
          path: filePath, 
          content: logText 
        });
        addLog('success', `日志已导出到: ${filePath}`);
      }
    } catch (error) {
      const errorMsg = `导出日志失败: ${error instanceof Error ? error.message : String(error)}`;
      addLog('error', errorMsg);
      throw new Error(errorMsg);
    }
  }

  return {
    // 状态
    logs,
    logFilter,
    searchKeyword,
    autoScroll,
    // 计算属性
    filteredLogs,
    // 方法
    addLog,
    info,
    warn,
    error,
    success,
    clearLogs,
    exportLogs,
  };
});
