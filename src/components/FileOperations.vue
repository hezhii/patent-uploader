<template>
  <div class="space-y-6">
    <!-- 文件夹配置 -->
    <div class="card">
      <h3 class="text-lg font-semibold mb-4">文件夹配置</h3>
      
      <div class="space-y-4">
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">
            源文件夹
          </label>
          <div class="flex space-x-2">
            <input
              v-model="sourcePath"
              type="text"
              placeholder="选择包含Excel文件的文件夹"
              readonly
              class="flex-1 px-3 py-2 border border-gray-300 rounded-md bg-gray-50 cursor-pointer"
              @click="selectSourceFolder"
            />
            <button
              @click="selectSourceFolder"
              class="btn-secondary"
            >
              浏览
            </button>
          </div>
        </div>

        <div v-if="needsConversion">
          <label class="block text-sm font-medium text-gray-700 mb-1">
            目标文件夹
          </label>
          <div class="flex space-x-2">
            <input
              v-model="targetPath"
              type="text"
              placeholder="选择转换后文件的保存位置"
              readonly
              class="flex-1 px-3 py-2 border border-gray-300 rounded-md bg-gray-50 cursor-pointer"
              @click="selectTargetFolder"
            />
            <button
              @click="selectTargetFolder"
              class="btn-secondary"
            >
              浏览
            </button>
          </div>
        </div>
        
        <div v-else class="p-3 bg-blue-50 border border-blue-200 rounded-md text-sm text-blue-700">
          <p>当前未配置列映射，将直接上传原始Excel文件，无需指定目标文件夹</p>
        </div>
      </div>
    </div>

    <!-- 扫描结果 -->
    <div v-if="scanResult" class="card">
      <h3 class="text-lg font-semibold mb-4">扫描结果</h3>
      
      <div class="grid grid-cols-2 gap-4 mb-4">
        <div class="text-center p-4 bg-blue-50 rounded-lg">
          <div class="text-2xl font-bold text-blue-600">{{ scanResult.file_count }}</div>
          <div class="text-sm text-blue-500">Excel文件</div>
        </div>
        <div class="text-center p-4 bg-green-50 rounded-lg">
          <div class="text-2xl font-bold text-green-600">{{ formatFileSize(scanResult.total_size) }}</div>
          <div class="text-sm text-green-500">总大小</div>
        </div>
      </div>

      <!-- 文件列表 -->
      <div v-if="scanResult.files.length > 0" class="max-h-32 overflow-y-auto bg-gray-50 rounded-md p-3">
        <div class="text-sm text-gray-600 mb-2">文件列表:</div>
        <div 
          v-for="(file, index) in scanResult.files" 
          :key="index"
          class="text-xs text-gray-500 truncate"
          :title="file"
        >
          {{ file.split('/').pop() || file.split('\\').pop() }}
        </div>
      </div>
    </div>

    <!-- 操作按钮 -->
    <div class="card">
      <div class="flex space-x-4">
        <button
          @click="handleScanFiles"
          :disabled="!sourcePath || scanning"
          class="btn-primary flex-1"
          :class="{ 'opacity-50 cursor-not-allowed': !sourcePath || scanning }"
        >
          {{ scanning ? '扫描中...' : '扫描文件' }}
        </button>
        
        <button
          @click="handleStartConversion"
          :disabled="!canConvert || converting"
          class="btn-secondary flex-1"
          :class="{ 'opacity-50 cursor-not-allowed': !canConvert || converting }"
        >
          {{ converting ? '处理中...' : (needsConversion ? '开始转换' : '准备上传') }}
        </button>
      </div>
    </div>

    <!-- 转换进度 -->
    <div v-if="converting" class="card">
      <h3 class="text-lg font-semibold mb-4">{{ needsConversion ? '转换进度' : '处理进度' }}</h3>
      
      <div class="space-y-3">
        <div class="w-full bg-gray-200 rounded-full h-2">
          <div 
            class="bg-blue-600 h-2 rounded-full transition-all duration-300"
            :style="{ width: '50%' }"
          ></div>
        </div>
        <div class="text-sm text-gray-600 text-center">
          {{ needsConversion ? '正在转换Excel文件...' : '正在准备Excel文件...' }}
        </div>
      </div>
    </div>

    <!-- 转换结果 -->
    <div v-if="convertedFiles.length > 0" class="card">
      <h3 class="text-lg font-semibold mb-4">转换完成</h3>
      
      <div class="bg-green-50 border border-green-200 rounded-md p-4 mb-4">
        <div class="flex items-center">
          <span class="text-green-500 text-xl mr-2">✅</span>
          <span class="text-green-700">
            {{ needsConversion ? `成功转换 ${convertedFiles.length} 个文件` : `已准备 ${convertedFiles.length} 个文件` }}，可以开始上传了
          </span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useFileOperations } from '@/composables/useFileOperations';
import { formatFileSize } from '@/utils';
import type { ColumnMapping } from '@/types';

interface Props {
  columnMappings: ColumnMapping[];
}

interface Emits {
  (e: 'converted', files: File[]): void;
  (e: 'error', error: string): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

const {
  sourcePath,
  targetPath,
  scanResult,
  convertedFiles,
  scanning,
  converting,
  canConvert,
  selectSourceFolder,
  selectTargetFolder,
  scanFiles,
  startConversion,
} = useFileOperations();

// 是否需要转换（有列映射配置）
const needsConversion = computed(() => props.columnMappings.length > 0);

async function handleScanFiles() {
  try {
    await scanFiles();
  } catch (error) {
    emit('error', error instanceof Error ? error.message : '扫描文件失败');
  }
}

async function handleStartConversion() {
  try {
    const files = await startConversion(props.columnMappings);
    if (files) {
      emit('converted', files);
    }
  } catch (error) {
    emit('error', error instanceof Error ? error.message : '转换文件失败');
  }
}
</script>