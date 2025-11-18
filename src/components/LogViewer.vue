<template>
  <div class="space-y-4">
    <!-- 日志控制 -->
    <div class="card">
      <div class="flex justify-between items-center">
        <h3 class="text-lg font-semibold">操作日志</h3>
        <div class="flex space-x-2">
          <button
            @click="clearLogs"
            :disabled="logs.length === 0"
            class="btn-secondary text-sm"
            :class="{ 'opacity-50 cursor-not-allowed': logs.length === 0 }"
          >
            清空日志
          </button>
          <button
            @click="exportLogs"
            :disabled="logs.length === 0"
            class="btn-secondary text-sm"
            :class="{ 'opacity-50 cursor-not-allowed': logs.length === 0 }"
          >
            导出日志
          </button>
        </div>
      </div>
    </div>

    <!-- 日志过滤 -->
    <div class="card">
      <div class="flex space-x-4 items-center">
        <label class="text-sm font-medium text-gray-700">日志级别:</label>
        <div class="flex space-x-2">
          <button
            v-for="level in logLevels"
            :key="level"
            @click="toggleLogLevel(level)"
            class="px-3 py-1 text-xs rounded-md border transition-colors"
            :class="{
              'bg-blue-100 border-blue-300 text-blue-700': visibleLevels.includes(level),
              'bg-gray-100 border-gray-300 text-gray-600': !visibleLevels.includes(level)
            }"
          >
            {{ level.toUpperCase() }}
          </button>
        </div>
      </div>
    </div>

    <!-- 日志显示区域 -->
    <div class="card">
      <div 
        ref="logContainer"
        class="bg-gray-900 text-green-400 font-mono text-sm p-4 rounded-md h-80 overflow-y-auto"
      >
        <div v-if="filteredLogs.length === 0" class="text-gray-500 text-center py-8">
          暂无日志记录
        </div>
        <div
          v-for="log in filteredLogs"
          :key="log.id"
          class="mb-1 leading-relaxed"
          :class="getLogColor(log.level)"
        >
          <span class="text-gray-400">[{{ formatTime(log.timestamp) }}]</span>
          <span class="mx-2 font-bold">[{{ log.level.toUpperCase() }}]</span>
          <span>{{ log.message }}</span>
        </div>
      </div>
    </div>

    <!-- 日志统计 -->
    <div class="card">
      <h3 class="text-lg font-semibold mb-4">日志统计</h3>
      
      <div class="grid grid-cols-4 gap-4">
        <div 
          v-for="(count, level) in logStats"
          :key="level"
          class="text-center p-3 rounded-lg"
          :class="getStatColor(level)"
        >
          <div class="text-lg font-bold">{{ count }}</div>
          <div class="text-sm capitalize">{{ level }}</div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch, nextTick } from 'vue';
import { useLoggerStore } from '@/stores/logger';

const loggerStore = useLoggerStore();
const {
  logs,
  clearLogs: clearAllLogs,
  exportLogs: exportAllLogs,
} = loggerStore;

// 日志级别
const logLevels = ['info', 'warn', 'error', 'success'] as const;
const visibleLevels = ref<string[]>([...logLevels]);

// DOM 引用
const logContainer = ref<HTMLElement>();

// 过滤后的日志
const filteredLogs = computed(() => {
  return logs.filter(log => visibleLevels.value.includes(log.level));
});

// 日志统计
const logStats = computed(() => {
  const stats: Record<string, number> = {};
  logLevels.forEach(level => {
    stats[level] = logs.filter(log => log.level === level).length;
  });
  return stats;
});

// 切换日志级别显示
function toggleLogLevel(level: string) {
  const index = visibleLevels.value.indexOf(level);
  if (index > -1) {
    visibleLevels.value.splice(index, 1);
  } else {
    visibleLevels.value.push(level);
  }
}

// 获取日志颜色
function getLogColor(level: string): string {
  switch (level) {
    case 'error':
      return 'text-red-400';
    case 'warn':
      return 'text-yellow-400';
    case 'info':
      return 'text-blue-400';
    case 'success':
      return 'text-green-400';
    default:
      return 'text-gray-400';
  }
}

// 获取统计颜色
function getStatColor(level: string): string {
  switch (level) {
    case 'error':
      return 'bg-red-50 text-red-700';
    case 'warn':
      return 'bg-yellow-50 text-yellow-700';
    case 'info':
      return 'bg-blue-50 text-blue-700';
    case 'success':
      return 'bg-green-50 text-green-700';
    default:
      return 'bg-gray-50 text-gray-700';
  }
}

// 格式化时间
function formatTime(timestamp: number): string {
  const date = new Date(timestamp);
  return date.toLocaleTimeString('zh-CN', {
    hour12: false,
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  });
}

// 清空日志
function clearLogs() {
  clearAllLogs();
}

// 导出日志
async function exportLogs() {
  try {
    await exportAllLogs();
  } catch (error) {
    console.error('导出日志失败:', error);
  }
}

// 自动滚动到底部
watch(filteredLogs, async () => {
  await nextTick();
  if (logContainer.value) {
    logContainer.value.scrollTop = logContainer.value.scrollHeight;
  }
});
</script>