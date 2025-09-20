<template>
  <div class="min-h-screen bg-gray-50">
    <!-- 顶部导航 -->
    <header class="bg-white shadow-sm border-b">
      <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div class="flex justify-between items-center h-16">
          <div class="flex items-center">
            <h1 class="text-xl font-semibold text-gray-900">专利文件上传系统</h1>
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
                v-for="(step, index) in steps"
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
          <div v-show="currentStep >= 0">
            <ServerConfig
              v-model="store.serverConfig"
              @save="handleServerConfigured"
              @test="handleConnectionTest"
            />
          </div>

          <!-- 步骤2: 列映射配置 -->
          <div v-show="currentStep >= 1">
            <ColumnMapping
              v-model="store.columnMappings"
              @save="handleMappingsUpdated"
            />
          </div>

          <!-- 步骤3: 文件操作 -->
          <div v-show="currentStep >= 2">
            <FileOperations
              :column-mappings="store.columnMappings"
              @converted="handleFilesConverted"
              @error="handleError"
            />
          </div>

          <!-- 步骤4: 文件上传 -->
          <div v-show="currentStep >= 3">
            <FileUpload
              :files="convertedFiles"
              @complete="handleUploadComplete"
              @error="handleError"
            />
          </div>

          <!-- 完成状态 -->
          <div v-if="currentStep >= 4" class="card">
            <div class="text-center py-8">
              <div class="text-6xl mb-4">🎉</div>
              <h2 class="text-2xl font-bold text-green-600 mb-2">上传完成！</h2>
              <p class="text-gray-600 mb-4">所有文件已成功上传到服务器</p>
              <button
                @click="resetWorkflow"
                class="btn-primary"
              >
                重新开始
              </button>
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
                v-for="(step, index) in steps"
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
                <span>{{ step.name }}</span>
              </div>
            </div>
          </div>

          <!-- 日志查看器 -->
          <LogViewer ref="logViewer" />
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
import { ref, computed } from 'vue';
import { useAppStore } from '@/stores';
import ServerConfig from '@/components/ServerConfig.vue';
import ColumnMapping from '@/components/ColumnMapping.vue';
import FileOperations from '@/components/FileOperations.vue';
import FileUpload from '@/components/FileUpload.vue';
import LogViewer from '@/components/LogViewer.vue';
import type { ConnectionStatus, ColumnMapping as ColumnMappingType } from '@/types';

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

// 状态管理
const currentStep = ref(0);
const convertedFiles = ref<File[]>([]);
const connectionStatus = ref<ConnectionStatus | null>(null);
const errorMessage = ref('');
const logViewer = ref();

// 计算属性
const currentStepName = computed(() => {
  return steps[currentStep.value]?.name || '未知步骤';
});

// 事件处理函数
function handleServerConfigured() {
  addLog('success', '服务器配置已完成');
  currentStep.value = Math.max(currentStep.value, 1);
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
  addLog('info', '列映射配置已更新');
  currentStep.value = Math.max(currentStep.value, 2);
}

function handleFilesConverted(files: File[]) {
  convertedFiles.value = files;
  addLog('success', `文件转换完成，共 ${files.length} 个文件`);
  currentStep.value = Math.max(currentStep.value, 3);
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

function resetWorkflow() {
  currentStep.value = 0;
  store.clearAllData();
  convertedFiles.value = [];
  connectionStatus.value = null;
  errorMessage.value = '';
  addLog('info', '工作流程已重置');
}

function addLog(level: 'info' | 'warn' | 'error' | 'success', message: string) {
  if (logViewer.value) {
    logViewer.value.addLog(level, message);
  }
}

// 初始化日志
addLog('info', '系统已启动，请配置服务器连接');
</script>