<template>
  <div class="min-h-screen bg-gray-50">
    <!-- 顶部导航 -->
    <header class="bg-white shadow-sm border-b">
      <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div class="flex justify-between items-center h-16">
          <div class="flex items-center">
            <h1 class="text-xl font-semibold text-gray-900">专利文件导入工具</h1>
            <div v-if="connectionStatus" class="ml-4 flex items-center">
              <div 
                class="w-2 h-2 rounded-full mr-2"
                :class="{
                  'bg-green-500': connectionStatus.type === 'success',
                  'bg-yellow-500': connectionStatus.type === 'warning',
                  'bg-red-500': connectionStatus.type === 'error',
                  'bg-blue-500': connectionStatus.type === 'info'
                }"
              ></div>
              <span class="text-sm text-gray-600">{{ connectionStatus.message }}</span>
            </div>
          </div>
          
          <div class="flex items-center space-x-4">
            <span class="text-sm text-gray-500">当前步骤: {{ currentStepName }}</span>
            <div class="flex space-x-1">
              <div 
                v-for="(_step, index) in steps"
                :key="index"
                class="w-2 h-2 rounded-full"
                :class="{
                  'bg-green-500': index < currentStep,
                  'bg-blue-500': index === currentStep,
                  'bg-gray-300': index > currentStep
                }"
              ></div>
            </div>
          </div>
        </div>
      </div>
    </header>

    <!-- 主内容区域 -->
    <main class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
      <div class="grid grid-cols-1 lg:grid-cols-3 gap-8">
        
        <!-- 左侧主要操作区域 -->
        <div class="lg:col-span-2 space-y-8">
          
          <!-- 步骤1: 服务器配置 -->
          <div v-show="currentStep >= 0" class="card">
            <div class="flex justify-between items-center mb-4 cursor-pointer" @click="toggleStep(0)">
              <h3 class="text-lg font-semibold">1. 服务器配置</h3>
              <span class="text-xl">{{ expandedSteps[0] ? '▼' : '▶' }}</span>
            </div>
            <div v-show="expandedSteps[0]">
              <ServerConfig
                v-model="store.serverConfig"
                @save="handleServerConfigured"
                @test="handleConnectionTest"
              />
            </div>
          </div>

          <!-- 步骤2: 列映射配置 -->
          <div v-show="currentStep >= 1" class="card">
            <div class="flex justify-between items-center mb-4 cursor-pointer" @click="toggleStep(1)">
              <h3 class="text-lg font-semibold">2. 列名映射配置（可选）</h3>
              <span class="text-xl">{{ expandedSteps[1] ? '▼' : '▶' }}</span>
            </div>
            <div v-show="expandedSteps[1]">
              <div class="mb-4 p-3 bg-blue-50 border border-blue-200 rounded-md text-sm text-blue-700">
                <p>如果不需要对Excel列进行转换，可以跳过此步骤，直接上传原始文件</p>
              </div>
              <ColumnMapping
                v-model="store.columnMappings"
                @save="handleMappingsUpdated"
              />
              <div class="mt-4">
                <button
                  @click="handleSkipMapping"
                  class="btn-secondary w-full"
                >
                  跳过列映射，直接处理文件
                </button>
              </div>
            </div>
          </div>

          <!-- 步骤3: 文件操作 -->
          <div v-show="currentStep >= 2" class="card">
            <div class="flex justify-between items-center mb-4 cursor-pointer" @click="toggleStep(2)">
              <h3 class="text-lg font-semibold">3. 文件处理</h3>
              <span class="text-xl">{{ expandedSteps[2] ? '▼' : '▶' }}</span>
            </div>
            <div v-show="expandedSteps[2]">
              <FileOperations
                :column-mappings="store.columnMappings"
                @converted="handleFilesConverted"
                @error="handleError"
              />
            </div>
          </div>

          <!-- 步骤4: 文件上传 -->
          <div v-show="currentStep >= 3" class="card">
            <div class="flex justify-between items-center mb-4 cursor-pointer" @click="toggleStep(3)">
              <h3 class="text-lg font-semibold">4. 文件上传</h3>
              <span class="text-xl">{{ expandedSteps[3] ? '▼' : '▶' }}</span>
            </div>
            <div v-show="expandedSteps[3]">
              <FileUpload
                :files="convertedFiles"
                @complete="handleUploadComplete"
                @error="handleError"
              />
            </div>
          </div>

          <!-- 完成状态 -->
          <div v-if="currentStep >= 4" class="card">
            <div class="text-center py-8">
              <div class="text-6xl mb-4">🎉</div>
              <h2 class="text-2xl font-bold text-green-600 mb-2">上传完成！</h2>
              <p class="text-gray-600 mb-4">所有文件已成功上传到服务器</p>
              <!-- <button
                @click="resetWorkflow"
                class="btn-primary"
              >
                重新开始
              </button> -->
            </div>
          </div>
        </div>

        <!-- 右侧日志和状态区域 -->
        <div class="space-y-6">
          <!-- 操作指南 -->
          <div class="card">
            <h3 class="text-lg font-semibold mb-4">操作指南</h3>
            <div class="space-y-3 text-sm">
              <div 
                v-for="(_step, index) in steps"
                :key="index"
                class="flex items-center space-x-2"
                :class="{
                  'text-green-600': index < currentStep,
                  'text-blue-600 font-medium': index === currentStep,
                  'text-gray-400': index > currentStep
                }"
              >
                <span 
                  class="w-6 h-6 rounded-full text-xs flex items-center justify-center"
                  :class="{
                    'bg-green-100 text-green-600': index < currentStep,
                    'bg-blue-100 text-blue-600': index === currentStep,
                    'bg-gray-100 text-gray-400': index > currentStep
                  }"
                >
                  {{ index + 1 }}
                </span>
                <span>{{ steps[index].name }}</span>
              </div>
            </div>
          </div>

          <!-- 日志查看器 -->
          <LogViewer />
        </div>
      </div>
    </main>

    <!-- 错误提示 -->
    <div
      v-if="errorMessage"
      class="fixed bottom-4 right-4 bg-red-500 text-white px-6 py-3 rounded-lg shadow-lg flex items-center space-x-2"
    >
      <span>❌</span>
      <span>{{ errorMessage }}</span>
      <button
        @click="errorMessage = ''"
        class="ml-2 text-white hover:text-red-200"
      >
        ✕
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { useAppStore } from '@/stores';
import { useLoggerStore } from '@/stores/logger';
import ServerConfig from '@/components/ServerConfig.vue';
import ColumnMapping from '@/components/ColumnMapping.vue';
import FileOperations from '@/components/FileOperations.vue';
import FileUpload from '@/components/FileUpload.vue';
import LogViewer from '@/components/LogViewer.vue';
import type { ConnectionStatus } from '@/types';

// 步骤定义
const steps = [
  { name: '配置服务器连接', description: '设置服务器地址和登录信息' },
  { name: '配置列映射', description: '设置Excel列与目标字段的映射关系' },
  { name: '处理文件', description: '扫描和转换Excel文件' },
  { name: '上传文件', description: '将转换后的文件上传到服务器' },
  { name: '完成', description: '所有操作已完成' }
];

// 使用 Pinia store
const store = useAppStore();

// 使用 logger store
const loggerStore = useLoggerStore();
const { addLog } = loggerStore;

// 状态管理
const currentStep = ref(0);
const convertedFiles = ref<File[]>([]);
const connectionStatus = ref<ConnectionStatus | null>(null);
const errorMessage = ref('');

// 步骤展开/收起状态
const expandedSteps = ref<Record<number, boolean>>({
  0: true,  // 服务器配置默认展开
  1: false,
  2: false,
  3: false
});

// Rust 日志监听器
let unlistenRustLog: UnlistenFn | null = null;

// 计算属性
const currentStepName = computed(() => {
  return steps[currentStep.value]?.name || '未知步骤';
});

// 事件处理函数
function handleServerConfigured() {
  // 保存到 store 并持久化
  store.saveServerConfig(store.serverConfig);
  addLog('success', '服务器配置已完成并保存');
  currentStep.value = Math.max(currentStep.value, 1);
  
  // 收起当前步骤，展开下一步
  expandedSteps.value[0] = false;
  expandedSteps.value[1] = true;
}

function handleConnectionTest(success: boolean) {
  if (success) {
    connectionStatus.value = { type: 'success', message: '连接成功' };
    addLog('success', '服务器连接测试成功');
  } else {
    connectionStatus.value = { type: 'error', message: '连接失败' };
    addLog('error', '服务器连接测试失败');
  }
}

function handleMappingsUpdated() {
  // 保存到 store 并持久化
  store.saveColumnMappings(store.columnMappings);
  addLog('info', `列映射配置已更新并保存，共 ${store.columnMappings.length} 个映射`);
  currentStep.value = Math.max(currentStep.value, 2);
  
  // 收起当前步骤，展开下一步
  expandedSteps.value[1] = false;
  expandedSteps.value[2] = true;
}

function handleSkipMapping() {
  // 清空列映射配置
  store.columnMappings = [];
  store.saveColumnMappings([]);
  addLog('info', '已跳过列映射配置，将直接上传原始文件');
  currentStep.value = Math.max(currentStep.value, 2);
  
  // 收起当前步骤，展开下一步
  expandedSteps.value[1] = false;
  expandedSteps.value[2] = true;
}

function toggleStep(stepIndex: number) {
  expandedSteps.value[stepIndex] = !expandedSteps.value[stepIndex];
}

function handleFilesConverted(files: File[]) {
  convertedFiles.value = files;
  addLog('success', `文件转换完成，共 ${files.length} 个文件`);
  currentStep.value = Math.max(currentStep.value, 3);
  
  // 收起当前步骤，展开下一步
  expandedSteps.value[2] = false;
  expandedSteps.value[3] = true;
}

function handleUploadComplete() {
  addLog('success', '所有文件上传完成');
  currentStep.value = 4;
}

function handleError(error: string) {
  errorMessage.value = error;
  addLog('error', error);
  
  // 3秒后自动清除错误消息
  setTimeout(() => {
    errorMessage.value = '';
  }, 3000);
}

// function resetWorkflow() {
//   currentStep.value = 0;
//   store.clearAllData();
//   convertedFiles.value = [];
//   connectionStatus.value = null;
//   errorMessage.value = '';
//   addLog('info', '工作流程已重置');
// }

// 监听 Rust 日志事件
onMounted(async () => {
  addLog('info', '系统已启动，请配置服务器连接');
  
  // 监听来自 Rust 的日志
  unlistenRustLog = await listen<{timestamp: number, level: string, message: string}>('rust-log', (event) => {
    const { level, message } = event.payload;
    addLog(level as 'info' | 'warn' | 'error' | 'success', `[Rust] ${message}`);
  });
});

onUnmounted(() => {
  if (unlistenRustLog) {
    unlistenRustLog();
  }
});
</script>