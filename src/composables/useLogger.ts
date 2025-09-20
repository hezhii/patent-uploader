import { ref, computed, readonly } from 'vue';
import type { LogEntry } from '@/types';
import { generateId, formatTime } from '@/utils';

export function useLogger() {
  const logs = ref<LogEntry[]>([]);
  const logFilter = ref<'all' | 'info' | 'warn' | 'error' | 'success'>('all');
  const searchKeyword = ref('');
  const autoScroll = ref(true);

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

  function exportLogs(): string {
    const logText = logs.value.map(log => {
      const time = formatTime(log.timestamp);
      return `[${time}] [${log.level.toUpperCase()}] ${log.message}`;
    }).join('\n');

    return logText;
  }

  return {
    logs: readonly(logs),
    filteredLogs,
    logFilter,
    searchKeyword,
    autoScroll,
    addLog,
    info,
    warn,
    error,
    success,
    clearLogs,
    exportLogs,
  };
}