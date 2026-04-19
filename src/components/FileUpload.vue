<template>
  <div class="space-y-6">
    <!-- 上传状态概览 -->
    <div class="card">
      <h3 class="text-lg font-semibold mb-4">文件上传</h3>
      
      <div v-if="uploadProgress.length === 0" class="text-center py-8 text-gray-500">
        请先转换Excel文件，然后开始上传
      </div>

      <div v-else class="space-y-4">
        <!-- 总体进度 -->
        <div class="bg-gray-50 rounded-lg p-4">
          <div class="flex justify-between items-center mb-2">
            <span class="text-sm font-medium">总进度</span>
            <span class="text-sm text-gray-600">
              {{ completedCount }}/{{ uploadProgress.length }} 文件
            </span>
          </div>
          <div class="w-full bg-gray-200 rounded-full h-2">
            <div 
              class="bg-blue-600 h-2 rounded-full transition-all duration-300"
              :style="{ width: `${overallProgress}%` }"
            ></div>
          </div>
        </div>

        <!-- 上传选项 -->
        <div class="mb-4">
          <label class="block text-sm font-medium text-gray-700 mb-1">
            导入模式
          </label>
          <select
            v-model="importMode"
            :disabled="uploading"
            class="w-full px-3 py-2 border border-gray-300 rounded-md bg-white focus:outline-none focus:ring-2 focus:ring-blue-500"
          >
            <option value="all">所有专利</option>
            <option value="inventionOnly">仅发明专利</option>
            <option value="validInventionOnly">仅有效发明专利</option>
          </select>
        </div>

        <!-- 控制按钮 -->
        <div class="flex space-x-4">
          <button
            @click="handleStartUpload"
            :disabled="uploading || uploadProgress.length === 0"
            class="btn-primary flex-1"
            :class="{ 'opacity-50 cursor-not-allowed': uploading || uploadProgress.length === 0 }"
          >
            {{ uploading ? '上传中...' : '开始上传' }}
          </button>
          
          <button
            @click="pauseUpload"
            :disabled="!uploading"
            class="btn-secondary"
            :class="{ 'opacity-50 cursor-not-allowed': !uploading }"
          >
            {{ uploadPaused ? '继续' : '暂停' }}
          </button>
          
          <button
            @click="clearCompleted"
            :disabled="uploading || completedCount === 0"
            class="btn-secondary"
            :class="{ 'opacity-50 cursor-not-allowed': uploading || completedCount === 0 }"
          >
            清除已完成
          </button>
        </div>
      </div>
    </div>

    <!-- 文件列表 -->
    <div v-if="uploadProgress.length > 0" class="card">
      <h3 class="text-lg font-semibold mb-4">上传列表</h3>
      
      <div class="space-y-3 max-h-96 overflow-y-auto">
        <div
          v-for="item in uploadProgress"
          :key="item.fileIndex"
          class="border rounded-lg p-3 transition-all duration-200"
          :class="{
            'border-blue-200 bg-blue-50': item.status === 'uploading',
            'border-green-200 bg-green-50': item.status === 'completed',
            'border-red-200 bg-red-50': item.status === 'failed',
            'border-gray-200 bg-white': item.status === 'pending'
          }"
        >
          <div class="flex items-center justify-between mb-2">
            <div class="flex-1 min-w-0">
              <div class="font-medium text-sm truncate" :title="item.fileName">
                {{ item.fileName }}
              </div>
              <div class="text-xs text-gray-500">
                文件 #{{ item.fileIndex + 1 }}
              </div>
            </div>
            
            <div class="flex items-center space-x-2">
              <!-- 状态图标 -->
              <span v-if="item.status === 'pending'" class="text-gray-400">⏳</span>
              <span v-else-if="item.status === 'uploading'" class="text-blue-500">📤</span>
              <span v-else-if="item.status === 'completed'" class="text-green-500">✅</span>
              <span v-else-if="item.status === 'failed'" class="text-red-500">❌</span>
              
              <!-- 重试按钮 -->
              <button
                v-if="item.status === 'failed'"
                @click="retryUpload(item.fileIndex)"
                class="text-xs px-2 py-1 bg-red-100 text-red-700 rounded hover:bg-red-200"
              >
                重试
              </button>
            </div>
          </div>

          <!-- 进度条 -->
          <div v-if="item.status === 'uploading' || item.status === 'completed'" class="mb-2">
            <div class="w-full bg-gray-200 rounded-full h-1">
              <div 
                class="h-1 rounded-full transition-all duration-300"
                :class="{
                  'bg-blue-500': item.status === 'uploading',
                  'bg-green-500': item.status === 'completed'
                }"
                :style="{ width: `${item.progress}%` }"
              ></div>
            </div>
            <div class="text-xs text-gray-500 mt-1">
              {{ item.progress }}%
            </div>
          </div>

          <!-- 错误信息 -->
          <div v-if="item.status === 'failed' && item.error" class="text-xs text-red-600 mt-1">
            {{ item.error }}
          </div>
        </div>
      </div>
    </div>

    <!-- 上传统计 -->
    <div v-if="uploadProgress.length > 0" class="card">
      <h3 class="text-lg font-semibold mb-4">上传统计</h3>
      
      <div class="grid grid-cols-3 gap-4">
        <div class="text-center p-4 bg-blue-50 rounded-lg">
          <div class="text-xl font-bold text-blue-600">{{ pendingCount }}</div>
          <div class="text-sm text-blue-500">等待中</div>
        </div>
        <div class="text-center p-4 bg-green-50 rounded-lg">
          <div class="text-xl font-bold text-green-600">{{ completedCount }}</div>
          <div class="text-sm text-green-500">已完成</div>
        </div>
        <div class="text-center p-4 bg-red-50 rounded-lg">
          <div class="text-xl font-bold text-red-600">{{ errorCount }}</div>
          <div class="text-sm text-red-500">失败</div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { useFileUpload } from '@/composables/useFileUpload';
import { useAppStore } from '@/stores';

interface Props {
  files: File[];
}

interface Emits {
  (e: 'complete'): void;
  (e: 'error', error: string): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

const store = useAppStore();

const {
  uploading,
  uploadProgress,
  overallProgress,
  completedCount,
  failedCount,
  isPaused,
  initializeUpload,
  startUpload,
  pauseUpload,
  retryUpload,
  clearUploadHistory,
  setImportMode,
} = useFileUpload();

// 从 store 加载配置
const importMode = ref(store.settings.importMode);

// 本地状态
const uploadPaused = computed(() => isPaused.value);

// 计算属性
const pendingCount = computed(() => 
  uploadProgress.value.filter((item: any) => item.status === 'pending').length
);

const errorCount = computed(() => failedCount.value);

// 简化的上传控制
async function handleStartUpload() {
  try {
    // 保存配置到 store
    store.updateSettings({ importMode: importMode.value });
    // 设置上传参数
    setImportMode(importMode.value);
    await startUpload();
  } catch (error) {
    emit('error', error instanceof Error ? error.message : '上传失败');
  }
}

function clearCompleted() {
  clearUploadHistory();
}

// 监听文件变化
watch(() => props.files, (newFiles: File[]) => {
  if (newFiles.length > 0) {
    initializeUpload(newFiles);
  }
}, { immediate: true });

// 监听上传完成
watch(completedCount, (newCount: number) => {
  if (newCount > 0 && newCount === uploadProgress.value.length) {
    emit('complete');
  }
});
</script>